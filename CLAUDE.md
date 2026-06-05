# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`rusty-rich` is a high-fidelity Rust port of Python's [Rich](https://github.com/Textualize/rich) library — terminal text formatting, styling, and rendering. Single crate, 51 modules, ~25.5K lines, published on crates.io.

## Build, Test, Lint

```bash
# Build (all features)
cargo build

# Build without optional deps (fastest compile)
cargo build --no-default-features

# Run all tests (742+)
cargo test

# Run a specific test file
cargo test --test battle_test
cargo test --test box_table_exhaustive

# Run a single test by name substring
cargo test --test battle_test test_name_substring

# Run inline unit tests in a specific module
cargo test --lib progress

# Show output during tests (for debugging)
cargo test -- --nocapture

# Format
cargo fmt --all

# Lint (warnings-as-errors, enforced in CI)
cargo clippy --all-targets -- -D warnings

# Build docs (no warnings)
cargo doc --no-deps --document-private-items

# Security audit
cargo deny check advisories
cargo deny check bans licenses sources

# Run the visual demo rendering all box styles, panels, tables
cargo run --example view_all
```

## Architecture: Rendering Pipeline

The core rendering flow: **Console** → **Renderable trait** → **RenderResult** (lines of Segments) → **ANSI string** → stdout.

```
User calls console.print(obj) or console.println(obj)
  → Console::render() calls obj.render(&options) → RenderResult
  → Recursively flattens nested RenderItems via flatten_items()
  → Converts Segments to ANSI via Segment::to_ansi()
  → Writes ANSI bytes to self.file (stdout or capture buffer)
```

### Key types in `src/console.rs`

- **`Renderable` trait** — universal output protocol: `fn render(&self, options: &ConsoleOptions) -> RenderResult`. Everything printable implements this (`&str`, `String`, `Text`, `Panel`, `Table`, `DynRenderable`, `Group`, etc.).
- **`Segment`** — atomic output unit: text + optional `Style` + optional `ControlCode`. Defined in `src/segment.rs`.
- **`RenderResult`** — has `lines: Vec<Vec<Segment>>` (flat) and `items: Vec<RenderItem>` (for recursive flattening of nested renderables). Use `from_items()` when your renderable nests others; the Console resolves the tree.
- **`RenderItem`** — enum: `Segment(Segment)` or `Nested(DynRenderable)`. Put sub-renderables here so they're recursively resolved.
- **`DynRenderable`** — type-erased `Arc<dyn Renderable + Send + Sync>` wrapper with `Clone + Debug`. Use this to store renderables in collections (Panel children, Table cells, Group items, etc.).
- **`ConsoleOptions`** — rendering context: terminal size, color system, width/height constraints, markup/highlight flags, `ascii_only`, justification/overflow overrides. Most renderables should obey `options.max_width` and `options.ascii_only`.
- **`Group`** — renders children sequentially (vertical stacking). Equivalent to Python Rich's `Group`.

### Measurement trait (`src/measure.rs`)

- **`Measurable` trait** — `fn measure(&self, options: &ConsoleOptions) -> Measurement`. Returns `{minimum, maximum}` widths for layout negotiation.
- **`Renderable::measure()`** — optional override; returns `Option<Measurement>`. If `None`, the Console falls back to measuring the rendered output.

## Style & Color System

- **`Style`** in `src/style.rs` — consuming builder pattern: all setters take `self` and return `Self`. 13 text attributes (bold, italic, underline, dim, blink, reverse, strike, overline, conceal, frame, encircle, blink2, underline2) plus link support. `Style::combine()` merges two styles (other takes precedence).
- **`Color`** in `src/color.rs` — three representations: `TrueColor` (24-bit RGB), `Color8Bit` (256-color palette), `Standard` (16 ANSI colors). `Color::parse("name")` handles ~256 ANSI names + ~140 CSS/X11 names + hex `#RRGGBB`. Auto-downgrade via `ColorSystem` to match terminal capabilities.
- **`Theme`** in `src/theme.rs` — 170+ named styles across 10 categories. Stack-based inheritance via `console.push_theme()` / `console.pop_theme()` (RAII `ThemeContext`).

## Box Drawing (`src/box_drawing.rs`)

17 box styles: `ROUNDED`, `SQUARE`, `HEAVY`, `HEAVY_EDGE`, `HEAVY_HEAD`, `DOUBLE`, `DOUBLE_EDGE`, `SIMPLE`, `SIMPLE_HEAVY`, `MINIMAL`, `MINIMAL_HEAVY`, `ASCII`, `ASCII2`, `SQUARE_DOUBLE_HEAD`, `MINIMAL_DOUBLE_HEAD`, `SIMPLE_HEAD`, `ASCII_DOUBLE_HEAD`, `MARKDOWN`. Definitions must match Python Rich 14.x exactly — validate with `compare_rich.py`.

## Module Organization

| Layer | Modules |
|---|---|
| **Styling engine** | `style`, `color`, `theme`, `text`, `segment`, `markup` |
| **Console & I/O** | `console`, `screen`, `control`, `export`, `emoji`, `ansi` |
| **Measurement & layout** | `cells`, `measure`, `align`, `constrain`, `ratio`, `padding`, `box_drawing` |
| **Renderables** | `panel`, `table`, `tree`, `rule`, `columns`, `layout`, `pretty`, `bar`, `progress`, `progress_columns`, `spinner`, `status`, `live`, `pager`, `palette`, `containers`, `styled`, `filesize` |
| **Content rendering** | `syntax`, `markdown`, `json`, `logging`, `log_render`, `traceback` |
| **Interactive** | `prompt`, `inspect`, `diagnose`, `file_proxy`, `scope`, `repr` |

## Project-Specific Conventions

- **Consuming builders** — all setter/builder methods take `self` and return `Self`. Never use `&mut self` builders.
- **Match Python Rich behavior** — when in doubt, check Python Rich 14.x output. Use `compare_rich.py` to diff against the Python reference.
- **ANSI stripped from width measurement** — `cells::cell_len()` excludes ANSI escape sequences before measuring Unicode display width. Always use it, never raw `str::len()`.
- **Theme fallbacks, never panics** — never index into a `HashMap` where a missing key would panic. Use `.get()` with a sensible fallback.
- **Terminal width** — max render width = `terminal_width - 1` to prevent edge-of-screen line wrapping.
- **`PhantomData<*const ()>`** — used on types that hold raw pointers (e.g. `ThemeContext`) to explicitly block `Send + Sync` auto-derive.

## Feature Flags

```toml
default = ["syntax-highlighting", "markdown"]
syntax-highlighting  # pulls in syntect (100+ languages)
markdown             # pulls in pulldown-cmark
minimal              # no optional deps
```

Always cfg-gate feature-dependent code and doc examples:
```rust
#[cfg(feature = "markdown")]
pub fn render_markdown(md: &str) -> Markdown { ... }

/// ```rust,no_run
/// # #[cfg(feature = "markdown")] {
/// use rusty_rich::render_markdown;
/// let md = render_markdown("# Hello");
/// # }
/// ```
```

## Testing

- **Unit tests** — inline in `src/*.rs` under `#[cfg(test)] mod tests { ... }`
- **Integration tests** — `tests/battle_test.rs` (103 tests) and `tests/box_table_exhaustive.rs` (150 tests covering all 18 box styles × features)
- Render to string and assert on key substrings (border characters, alignment, ANSI codes, visible text).

## Python Parity Comparison

`compare_rich.py` renders all box styles, panels, and tables using Python Rich 14.x. Pipe output to a file and diff against the Rust `view_all` example output to verify rendering fidelity.
