# Contributing to rusty-rich

Thanks for your interest in helping out! rusty-rich is a Rust port of Python's [Rich](https://github.com/Textualize/rich) library — it brings beautiful terminal formatting to Rust. We welcome bug reports, feature requests, documentation improvements, and code contributions.

## Table of contents

- [Getting started](#getting-started)
- [Development workflow](#development-workflow)
- [Code style](#code-style)
- [Testing](#testing)
- [Pull request process](#pull-request-process)
- [Reporting bugs](#reporting-bugs)
- [Feature requests](#feature-requests)
- [Architecture](#architecture)
- [Release process](#release-process)

## Getting started

### Prerequisites

- **Rust** — stable toolchain (1.70+). Install with [rustup](https://rustup.rs).
- **Git** — to clone and branch.

### Setup

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/rusty-rich.git
cd rusty-rich

# 2. Build
cargo build

# 3. Run the test suite
cargo test

# 4. Run the visual demo (renders all box styles, panels, tables)
cargo run --example view_all
```

### Tools you'll want

| Tool | Install | Used for |
|---|---|---|
| `rustfmt` | `rustup component add rustfmt` | Code formatting |
| `clippy` | `rustup component add clippy` | Linting |
| `cargo-audit` | `cargo install cargo-audit` | Security advisory check |
| `cargo-deny` | `cargo install cargo-deny` | License/duplicate/source audit |

## Development workflow

```bash
# 1. Create a feature branch
git checkout -b feat/my-feature

# 2. Make changes, build, test
cargo build
cargo test

# 3. Format and lint before committing
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# 4. Commit (short, imperative, explain "why")
git commit -m "fix: clamp end index in Text::stylize to prevent OOB"

# 5. Push and open a PR against master
git push origin feat/my-feature
```

### Commit style

We use conventional-ish, imperative commits:

- `fix:` — bug fixes (patch release)
- `feat:` — new features (minor release)
- `docs:` — documentation only
- `refactor:` — no behavioral change
- `test:` — test additions/changes
- `chore:` — deps, CI, build tooling

Keep the subject line under 72 characters. Explain *why* in the body, not *what* — the diff already shows what changed.

## Code style

### Rust conventions

- Follow `rustfmt` defaults (enforced by CI).
- Follow `clippy` warnings-as-errors (enforced by CI).
- Use `///` doc comments on all public items — we publish to docs.rs.
- Prefer `&str` over `&String`, `impl Trait` over explicit generics where idiomatic.

### Project-specific conventions

- **Builder methods consume `self`**. Follow the pattern used by `Style`, `Panel`, `Table`, etc. — all `.setter(value)` methods take `self` and return `Self`:
  ```rust
  // ✅ Right — consuming builder
  pub fn bold(self, value: bool) -> Self { Self { bold: Some(value), ..self } }

  // ❌ Wrong — &mut builder (inconsistent with the rest of the crate)
  pub fn bold(&mut self, value: bool) -> &mut Self { ... }
  ```

- **Match Python Rich behavior**. When in doubt, check what Python Rich 14.x does. The `compare_rich.py` script helps verify output against the Python version.

- **ANSI escape sequences must be excluded from `cell_length()`**. Use `cells::cell_len()` which strips ANSI before measuring Unicode display width.

- **Box styles are defined in `box_drawing.rs`**. If adding a new style, follow the exact corner/edge naming scheme from Python Rich.

- **Theme fallbacks are required**. Never index into a `HashMap` where a missing key would panic — use `.get()` with a sensible fallback.

## Testing

### Running tests

```bash
# All tests
cargo test

# Specific test files
cargo test --test battle_test           # 103 integration tests
cargo test --test box_table_exhaustive  # 150 exhaustive box/table tests

# Run with output shown (for debugging)
cargo test -- --nocapture

# Run a single test
cargo test --test battle_test test_name_substring
```

### Writing tests

1. **Unit tests** go inline in `src/*.rs` under `#[cfg(test)] mod tests { ... }`.
2. **Integration tests** go in `tests/*.rs`.
3. **Exhaustive tests** for rendering features — cover all box styles × feature combinations.

When fixing a rendering bug, include a test that:
- Renders the affected component to a string
- Asserts the exact expected output (or key substrings like border characters, alignment, etc.)

### CI will run

All PRs trigger [CI](.github/workflows/ci.yml) (test, clippy, fmt, doc) across Ubuntu/macOS/Windows on stable + beta, plus a [security audit](.github/workflows/security-audit.yml). Make sure your branch passes locally before opening a PR:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps --document-private-items
```

## Pull request process

1. **Open an issue first** for features — discuss the approach before writing code.
2. **Keep PRs focused** — one feature or fix per PR. Small PRs get reviewed faster.
3. **Update the CHANGELOG** under an `## Unreleased` section with your change.
4. **Add tests** that verify your change works and won't regress.
5. **Update docs** — if you added/changed a public API, document it with `///` comments and a `# Examples` section.
6. **Link the issue** in the PR description: `Closes #123`.

### Review checklist

Before marking a PR ready for review:

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test` passes (all 778+ tests)
- [ ] `cargo doc --no-deps` has no warnings
- [ ] New public items have `///` doc comments
- [ ] CHANGELOG entry added
- [ ] PR targets the `master` branch

## Reporting bugs

Include as much of this as you can:

1. **rusty-rich version** — `cargo tree | grep rusty-rich`
2. **Rust version** — `rustc --version`
3. **Terminal** — which emulator, $TERM, does it support true color?
4. **Minimal reproduction** — a short `fn main()` that shows the bug
5. **Expected vs actual output** — screenshots help for rendering issues

Open a [GitHub Issue](https://github.com/sinescode/rusty-rich/issues/new) with the **bug report** template.

## Feature requests

Check the [Python Rich feature parity table](README.md#-compared-to-python-rich) first — the goal is closing the ~12% gap. Features that bring rusty-rich closer to Python Rich behavior get priority.

For new features beyond parity:
1. Open an issue describing the use case
2. Mention which Python Rich feature/API it corresponds to (if any)
3. Propose the public API (module path, types, method signatures)

## Architecture

rusty-rich has 48 modules organized into these layers:

```
User-facing renderables    Table, Panel, Tree, Rule, Progress, Bar, Columns, Layout, Syntax, Markdown, JSON, Traceback, Inspect, Pretty, LogRender, Prompt, Spinner, Status, Live, Pager

Styling engine             Style, Color, Theme, Text, Segment, Markup

Terminal I/O               Console, Screen, Control, Export, Emoji, ANSI

Measurement & layout       Cells, Measure, Align, Constrain, Ratio, Padding, BoxDrawing

Internals                  Highlighter, Palette, Repr, FileProxy, Scope, Diagnose, Containers, Styled, Filesize
```

### Key design points

- **Console** is the central rendering engine — everything flows through it. It measures, renders to segments, and writes ANSI to the terminal.
- **Segment** is the atomic unit of output — a string slice + optional Style + optional ControlCode.
- **Style** uses a consuming builder pattern — every setter takes `self` and returns `Self`, so you chain or reassign.
- **Color** supports three representations: TrueColor (24-bit RGB), 8-bit (256-color palette), and Standard (16 ANSI colors), with automatic downgrade via `ColorSystem`.
- **Theme** provides 170+ named styles with stack-based inheritance — `console.push_theme()` / `console.pop_theme()`.
- **Renderable** trait is the universal output protocol — `fn render(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment>`.

### Where to find things

| What | Where |
|---|---|
| Rendering protocol | `src/console.rs` — `Renderable` trait + `Console::render()` |
| Box/border definitions | `src/box_drawing.rs` — 17 `BoxStyle` constants |
| Theme presets | `src/theme.rs` — `Theme::default_light()`, `Theme::default_dark()` |
| Color names | `src/color.rs` — `COLOR_NAMES` HashMap (256 entries) |
| ANSI escape generation | `src/style.rs` — `Style::to_ansi()` |
| Segment manipulation | `src/segment.rs` — 15+ utility functions |
| Markup parser | `src/markup.rs` — `[bold red]text[/bold red]` → styled Text |
| Table rendering | `src/table.rs` — `Table::render()` with colspan/rowspan/sections |
| Progress columns | `src/progress_columns.rs` — 11 column types |
| Syntax themes | `src/syntax.rs` — `Syntax::render()` (syntect integration) |
| Markdown | `src/markdown.rs` — pulldown-cmark → renderables |
| HTML/SVG export | `src/export.rs` — 4 preset export themes |
| Interactive prompts | `src/prompt.rs` — Prompt, IntPrompt, FloatPrompt, Confirm, Select |
| Full-screen apps | `src/screen.rs` — `ScreenContext` RAII guard (alt-screen) |

## Release process

Maintainers only — this documents how releases are cut:

```bash
# 1. Bump version in Cargo.toml
# 2. Update CHANGELOG.md — move Unreleased → version section
# 3. Commit and tag
git add Cargo.toml CHANGELOG.md Cargo.lock
git commit -m "vX.Y.Z: <summary>"
git push origin master

# 4. Publish to crates.io
cargo publish
```

Versioning follows [SemVer](https://semver.org):
- **Patch** (0.4.x) — bug fixes, safety fixes
- **Minor** (0.x.0) — new features, new renderables, parity improvements
- **Major** (1.0.0) — reserved for API stability / 100% Python Rich parity

---

Questions? Open a [discussion](https://github.com/sinescode/rusty-rich/discussions) or ask in an issue.
