<p align="center">
  <img src="assets/logo.svg" alt="rusty-rich" width="600"/>
</p>

<p align="center">
  <strong>Rich Text &amp; Beautiful Terminal Formatting — A High-Fidelity Rust Port of Python's <a href="https://github.com/Textualize/rich">Rich</a></strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/rusty-rich"><img src="https://img.shields.io/crates/v/rusty-rich?color=F74C00" alt="crates.io"></a>
  <a href="https://docs.rs/rusty-rich"><img src="https://img.shields.io/docsrs/rusty-rich?color=F74C00" alt="docs.rs"></a>
  <a href="https://github.com/sinescode/rusty-rich/actions/workflows/ci.yml"><img src="https://img.shields.io/badge/CI-passing-brightgreen" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-742%2B-brightgreen" alt="tests"></a>
  <a href="#"><img src="https://img.shields.io/badge/parity-~86%25-orange" alt="parity"></a>
</p>

---

`rusty-rich` is a high-performance Rust library for terminal text formatting, styling, and rendering. It ports Python's beloved [Rich](https://github.com/Textualize/rich) library to Rust — delivering the same expressive API with zero-cost abstractions, thread safety, and native performance.

- **51 modules** · **~25,500 lines** · **742+ tests** · **~86% feature parity** with Python Rich 14.x
- **0 dependencies deprecated** · **0 Clippy warnings** · **CI all green** (Linux, macOS, Windows)
- **7 security vulnerabilities fixed** · **10 bugs resolved** · **3 dependencies upgraded**

---

## Installation

```toml
[dependencies]
rusty-rich = "0.4"
```

**Minimum Rust version**: 1.80+ (for `std::sync::LazyLock` and `std::io::IsTerminal`).

**Feature flags**:
- `default` = `["syntax-highlighting", "markdown"]`
- `syntax-highlighting` — pulls in `syntect` (100+ languages)
- `markdown` — pulls in `pulldown-cmark`
- `minimal` — no optional deps, fastest compile

---

## Quick Start

```rust
use rusty_rich::{Console, Panel, Table, Column, Rule, Style, Color};

let mut console = Console::new();

// Inline markup
console.print_str("[bold green]Hello, [red]World![/red][/bold green]");

// Bordered panel
let panel = Panel::new("Hello inside a rounded box!")
    .title("My Panel")
    .border_style(Style::new().color(Color::parse("cyan").unwrap()));
console.println(&panel);

// Table with columns
let mut table = Table::new();
table.add_column(Column::new("Name"));
table.add_column(Column::new("Age"));
table.add_row_str("Alice", "30");
table.add_row_str("Bob", "25");
console.println(&table);

// Horizontal rule
console.rule("Section Break", None, None, None);
```

---

## Features

| Category | Highlights |
|---|---|
| **Style** | 13 attributes (bold, italic, underline, dim, blink, reverse, strike, overline, conceal, frame, encircle, blink2, underline2), links, metadata, chaining |
| **Color** | 256 ANSI names, 148 CSS/X11 names, hex, RGB, TrueColor → 8-bit auto-downgrade, blending |
| **Markup** | `[bold red on blue]text[/]` BBCode-like inline styling with proper tag matching |
| **Theme** | 170+ named styles across 10 categories with stack-based inheritance |
| **Table** | Headers, footers, colspan, rowspan, sections, 17 box styles, `.grid()` |
| **Panel** | Bordered containers with titles, subtitles, padding, auto-fit |
| **Tree** | Hierarchical rendering with Unicode/ASCII guides |
| **Layout** | Recursive split-pane layout with ratio sizing and named regions |
| **Progress** | Multi-task bars, 11 column types, `track()` / `wrap_file()` / `open()` |
| **Live** | Auto-updating displays, `LiveWriter` for stdout/stderr capture, thread-safe |
| **Syntax** | 100+ languages via `syntect`, lexer guessing, `.stylize_range()` |
| **Markdown** | Headings, code blocks, lists, blockquotes, links, tables |
| **Prompts** | `Prompt`, `IntPrompt`, `FloatPrompt`, `Confirm`, `Select`, password masking |
| **Traceback** | Rich exception rendering with source context, frame suppression, panic hook |
| **Inspect** | Structured object introspection with attribute/method tables |
| **Export** | HTML & SVG with 4 preset themes (Monokai, Dimmed Monokai, Night Owl, Light) |
| **Control** | Composable terminal escape sequences via `Control` type |
| **Box Drawing** | 17 built-in box styles (rounded, square, heavy, double, ASCII, markdown…) |
| **Spinner** | 55 animated spinner frames with case-insensitive lookup |
| **JSON** | Pretty-printed, syntax-highlighted JSON rendering |
| **Logging** | `RichHandler` for the `log` crate + standalone `LogRender` formatter |
| **Pager** | System pager integration (`$PAGER` / `less`) with RAII `PagerContext` |
| **Screen** | Alternate screen buffer with `ScreenContext` RAII guard |

---

## Security

10 vulnerabilities audited — **7 fixed**, **3 mitigated**. Zero open HIGH or CRITICAL issues. See `Full_audit.md` for the complete security analysis.

---

## Python Rich Parity

| Dimension | Score | Status |
|---|---|---|
| Console & Rendering | 90% | Print, log, rule, input, capture, pager, screen |
| Style & Color | 82% | 13 attrs, 256+140 colors, blending, downgrade |
| Layout & Renderables | 90% | Panel, Table, Tree, Rule, Columns, Layout, Padding |
| Progress & Live | 80% | Multi-task, 11 columns, track, Status, LiveWriter |
| Content Rendering | 82% | Syntax, Markdown, JSON, Logging, Traceback |
| Export | 92% | HTML, SVG, Text, 4 themes, ANSI decode |
| Interactive | 86% | 5 prompt types, Inspect, Control, Pager |
| **Overall** | **~86%** | 51 modules, full API compatibility |

---

## Contributing

See `RAW_URLS_AND_AI_PROMPTS.md` for AI-ready analysis prompts covering parity comparison, security audit, architecture review, performance analysis, and upgrade roadmap.

CI runs on every push: build (×3 OS), test (all + no-default features), lint (fmt + clippy), docs (warnings as errors), and security audit (cargo-deny).

---

## License

MIT — See [LICENSE](LICENSE).

<p align="center">
  <sub>Inspired by <a href="https://github.com/Textualize/rich">Textualize/rich</a> — the Python library that made terminal output beautiful.</sub>
</p>
