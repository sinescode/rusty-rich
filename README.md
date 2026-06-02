<p align="center">
  <img src="assets/logo.svg" alt="rusty-rich logo" width="600"/>
</p>

<p align="center">
  <strong>Rich text and beautiful formatting for the terminal — a Rust port of Python's <a href="https://github.com/Textualize/rich">Rich</a> library.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/rusty-rich"><img src="https://img.shields.io/crates/v/rusty-rich?color=F74C00" alt="crates.io"></a>
  <a href="https://docs.rs/rusty-rich"><img src="https://img.shields.io/docsrs/rusty-rich?color=F74C00" alt="docs.rs"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-475%20passed-brightgreen" alt="tests"></a>
</p>

---

## ✨ Features

- 🎨 **Style** — 13 text attributes (bold, italic, underline, dim, blink, reverse, strikethrough, overline, conceal, frame, encircle, blink2, underline2), links, metadata, style chaining & combination
- 📝 **Console markup** — `[bold red]text[/bold red]` BBCode-like inline styling
- 🎯 **256 named colors** — full ANSI 256-color palette with aliases, hex, RGB, auto-downgrade, color blending
- 🎭 **170+ theme styles** — repr, json, markdown, logging, traceback, rule, bar, progress, table, tree, syntax, prompt categories with stack-based inheritance

- 📊 **Table** — tabular data with headers, footers, row/column styling, colspan/rowspan, sections, 17 box styles, `.grid()` classmethod
- 🌲 **Tree** — hierarchical tree rendering with Unicode/ASCII guides
- 📦 **Panel** — bordered containers with titles, subtitles, padding, `.fit()` for auto-sizing
- ➖ **Rule** — horizontal dividers with optional centered, left-, or right-aligned titles
- 📐 **Padding & Align** — CSS-style padding (1–4 sides) and horizontal/vertical alignment wrappers
- 📋 **Columns** — side-by-side column layout with equal-width and expand options
- 🗂️ **Layout** — recursive split-pane layout engine with ratio sizing, named regions, and pluggable splitters

- ⏳ **Progress** — multi-task progress bars with 11 column types, `track()`/`wrap_file()`/`open()` wrappers, file tracking
- 🔄 **Spinner** — 55 animated spinners with case-insensitive name lookup
- 📌 **Status** — spinner + message with in-place refresh
- 🔄 **Live** — auto-updating displays with `LiveWriter` for stdout/stderr capture

- 🌈 **Syntax highlighting** — powered by syntect (100+ languages), `.from_path()`, `.guess_lexer()`, `.stylize_range()`
- 📝 **Markdown** — headings, code blocks, lists, blockquotes, links, tables, image placeholders
- 📋 **JSON** — pretty-printed, syntax-highlighted JSON
- 🔍 **Logging** — `RichHandler` for the `log` crate + standalone `LogRender` formatter with table output

- 🖼️ **Box drawing** — 17 box styles (rounded, square, heavy, double, ASCII, markdown, etc.)
- 🖥️ **Screen / Alt-screen** — full-screen terminal applications with `ScreenContext` RAII guard
- ⌨️ **Prompts** — `Prompt`, `IntPrompt`, `FloatPrompt`, `Confirm`, `Select`, password mode
- 🔴 **Traceback** — rich exception rendering with locals, source code, frame suppression, panic hook
- 🔍 **Inspect** — structured object introspection with attribute and method tables
- 🎛️ **Control** — composable terminal escape sequences (cursor, screen, title, bell, erase)
- 📤 **HTML & SVG export** — capture console output for the web with 4 preset themes
- 😊 **Emoji** — 100+ `:shortcode:` → Unicode emoji replacement
- 🌈 **Palette** — gradient, rainbow, and monochrome color palette generation
- 📊 **Bar chart** — horizontal bar chart renderable with labels and auto-scaling
- 🖨️ **Pretty printing** — `Pretty`, `pprint()`, `pretty_repr()`, `traverse()` with `Node` tree
- 📟 **Pager** — system pager integration (`$PAGER` / `less`) with `PagerContext` RAII
- 🔤 **ANSI decoder** — parse existing ANSI escape sequences back to styled `Text`
- 🧩 **Segment utilities** — simplify, split_lines, strip_styles, strip_links, align, divide, set_shape, filter_control

## 📦 Installation

```toml
[dependencies]
rusty-rich = "0.3"
```

## 🚀 Quick Start

```rust
use rusty_rich::{
    Console, Panel, Table, Column, Rule, Tree,
    Style, Color, AlignMethod, Padding,
};

fn main() {
    let mut console = Console::new();

    // Print with markup
    console.print_str("[bold green]Hello, [red]World![/red][/bold green]");

    // Create a panel with a title
    let panel = Panel::new("Hello inside a rounded box!")
        .title("My Panel")
        .border_style(Style::new().color(Color::parse("cyan").unwrap()));
    console.println(&panel);

    // Create a table
    let mut table = Table::new();
    table.add_column(Column::new("Name").justify(AlignMethod::Left));
    table.add_column(Column::new("Age").justify(AlignMethod::Right));
    table.add_row_str("Alice", "30");
    table.add_row_str("Bob", "25");
    console.println(&table);

    // Create a tree
    let mut tree = Tree::new("Root");
    tree.add("Child 1").add("Grandchild");
    tree.add("Child 2");
    console.println(&tree);

    // Draw a rule
    console.rule("Section Break", None, None, None);
}
```

## 🎯 Colors (256 names)

```rust
use rusty_rich::{Color, Style};

// Named colors — 256 ANSI palette
let red = Color::parse("red").unwrap();
let hot_pink = Color::parse("hot_pink").unwrap();
let steel_blue = Color::parse("steel_blue").unwrap();
let grey53 = Color::parse("grey53").unwrap();

// Hex / RGB
let orange = Color::from_hex("#FF6600").unwrap();
let custom = Color::from_rgb(100, 200, 50);

// TrueColor → 8-bit → Standard auto-downgrade
let style = Style::new()
    .color(Color::parse("#FF6600").unwrap())
    .bgcolor(Color::parse("#1E1E2E").unwrap())
    .bold(true)
    .italic(true);
```

## 📊 Table with Colspan & Rowspan

```rust
use rusty_rich::{Table, Column, Cell};

let mut table = Table::new().title("User Report");
table.add_column(Column::new("Name"));
table.add_column(Column::new("Details").colspan(2));  // spans 2 columns
table.add_column(Column::new("Role"));                  // skipped by colspan above

let row = vec![
    Cell::new("Alice"),
    Cell::new("Engineer").colspan(2),
];
table.add_row(row);
```

## ⏳ Progress Bars

```rust
use rusty_rich::Progress;
use std::thread;
use std::time::Duration;

let mut progress = Progress::new();
let task_id = progress.add_task("Downloading...", Some(100.0));

for i in 0..=100 {
    progress.update(task_id, i as f64);
    print!("\r{}", progress.render(80));
    thread::sleep(Duration::from_millis(20));
}
println!();

// Or use the `track()` convenience with an iterator
let items: Vec<_> = (0..100).collect();
let tracker = progress.track(items, "Processing", None);
for item in tracker {
    // process item — progress auto-advances
}
```

## 📝 Markdown (with tables)

```rust
use rusty_rich::render_markdown;
use rusty_rich::Console;

let md = render_markdown("
# Hello

| Name  | Age |
|-------|-----|
| Alice | 30  |
| Bob   | 25  |

- list item 1
- list item 2
");

let console = Console::new();
console.println(&md);
```

## ⌨️ Interactive Prompts

```rust
use rusty_rich::{Prompt, Confirm, IntPrompt, Select};

// String input
let name = Prompt::ask_with("Enter your name").unwrap();

// Password input (masked with *)
let password = Prompt::new("Password").password(true).ask().unwrap();

// Confirmation with default
let ok = Confirm::ask_with("Continue?", true).unwrap();

// Integer with validation
let age = IntPrompt::ask_with("Enter age").unwrap();

// Pick from numbered choices
let choice = Select::new("Pick a color")
    .choice("Red", "red")
    .choice("Green", "green")
    .choice("Blue", "blue")
    .ask()
    .unwrap();
```

## 🔄 Live Display with Writer

```rust
use rusty_rich::{Console, Live, LiveWriter, Panel};
use std::io::Write;
use std::thread;
use std::time::Duration;

let mut live = Live::new(Panel::new("Starting...").title("Status"));
let mut writer = live.create_writer();
live.start().unwrap();

for i in 0..=100 {
    // Redirect writes through the live display
    writeln!(writer, "Processing item {}...", i).unwrap();
    live.update(Panel::new(format!("Progress: {}%", i)).title("Status")).unwrap();
    thread::sleep(Duration::from_millis(50));
}

live.stop().unwrap();
```

## 🔴 Rich Tracebacks

```rust
use rusty_rich::traceback;

// Install a global panic hook for rich tracebacks
traceback::install();

// Or render manually
let tb = Traceback::from_exception("MyError", "something went wrong", frames)
    .show_locals(true)
    .max_frames(5)
    .suppress(vec!["std::".into(), "core::".into()]);
```

## 🔍 Object Inspection

```rust
use rusty_rich::Inspect;

// Inspect any Debug-printable value
let value = vec![1, 2, 3];
let insp = Inspect::new(&value)
    .title("my_vec")
    .add_attr("len", "usize", "3")
    .add_attr("capacity", "usize", "3")
    .add_method("push", "fn push(&mut self, value: T)")
    .add_method("pop", "fn pop(&mut self) -> Option<T>")
    .methods(true);
```

## 🎛️ Terminal Control Sequences

```rust
use rusty_rich::control::{Control, control_clear, control_home, control_bell};

// Ring the bell
let bell = Control::bell();

// Clear screen and go home
let clear = Control::clear_home();

// Move cursor and set window title
let setup = Control::cursor_to(1, 1);
let title = Control::title("My App");

// Strip or escape control codes from strings
use rusty_rich::control::{strip_control_codes, escape_control_codes};
let clean = strip_control_codes("hello\x07world");  // "helloworld"
let escaped = escape_control_codes("\x07");           // "\\a"
```

## 📋 Log Record Formatting

```rust
use rusty_rich::LogRender;

let mut renderer = LogRender::new()
    .show_time(true)
    .show_level(true)
    .show_path(true);

// Format a single log record
let record = renderer.render_log(
    Some("2024-01-15T10:30:00"),
    "ERROR",
    "Connection refused",
    Some("src/main.rs"),
    Some(42),
);

// Or render a batch as a table
let records = vec![
    (Some("10:30:00"), "INFO",  "Server started",       Some("main.rs"), Some(10)),
    (Some("10:30:01"), "ERROR", "Connection refused",   Some("net.rs"),  Some(99)),
];
let table = renderer.render_batch(&records);
```

## 🖥️ Full-Screen Apps

```rust
use rusty_rich::{Console, Screen, Live, Panel};
use std::thread;
use std::time::Duration;

let mut console = Console::new();
let mut screen = console.screen();  // enters alternate screen
screen.enter();

let mut live = Live::new(Panel::new("Loading...").title("Status"));
live.start().unwrap();

for i in 0..=100 {
    live.update(Panel::new(format!("Progress: {}%", i)).title("Status")).unwrap();
    thread::sleep(Duration::from_millis(50));
}

live.stop().unwrap();
screen.exit();  // restores terminal
```

## 🎨 Box Styles (17 built-in)

| Style | Preview |
|---|---|
| `BOX_ROUNDED` | ╭─╮ │ │ ╰─╯ |
| `BOX_SQUARE` | ┌─┐ │ │ └─┘ |
| `BOX_HEAVY` | ┏━┓ ┃ ┃ ┗━┛ |
| `BOX_DOUBLE` | ╔═╗ ║ ║ ╚═╝ |
| `BOX_DOUBLE_EDGE` | ╔═╗ ║ │ ╚═╝ |
| `BOX_HEAVY_EDGE` | ┏━┓ ┃ │ ┗━┛ |
| `BOX_HEAVY_HEAD` | ┏━┓ ┃ ┃ └─┘ |
| `BOX_SIMPLE` | borderless with separators |
| `BOX_SIMPLE_HEAVY` | borderless with heavy separators |
| `BOX_MINIMAL` | minimal horizontal rules |
| `BOX_ASCII` | `+--+` ASCII-safe |
| `BOX_ASCII2` | `+--+` alternate ASCII |
| `BOX_MARKDOWN` | pipe-style markdown tables |
| … and 4 more |

## 🧩 Segment Utilities

```rust
use rusty_rich::segment::{self, Segment, Segments};

let segs: Segments = vec![
    Segment::styled("Hello ", Style::new().bold(true)),
    Segment::styled("World", Style::new().bold(true)),
].into();

// Combine adjacent same-styled segments
let simplified = segs.simplify();           // → one "Hello World" segment

// Split into lines
let lines = segment::split_lines(&segs.segments);

// Strip all styling
let plain = segment::strip_styles(&segs.segments);  // → "Hello World"

// Align vertically
let aligned = segment::align_middle(&lines, 80, 10, None);
```

## 📂 Module Map (48 modules)

```
src/
├── lib.rs              # Crate root + re-exports (~100 types)
├── console.rs          # Central rendering engine (148 methods)
├── screen.rs           # Full-screen / alt-screen / ScreenContext
├── color.rs            # TrueColor / 256 / Standard (256 names)
├── style.rs            # 13 attributes + links + metadata + chain/copy
├── theme.rs            # 170+ named styles + stack-based inheritance
├── segment.rs          # Segment + control codes + 15+ utility fns
├── text.rs             # Text with Span styling (99 methods)
├── cells.rs            # Unicode cell width utilities
├── measure.rs          # Width measurement protocol
├── align.rs            # Horizontal + vertical alignment
├── ratio.rs            # Proportional space distribution
├── markup.rs           # BBCode-like markup parser
├── highlighter.rs      # Regex/Repr/ISO8601/JSON/Path highlighters
│
├── panel.rs            # Bordered container + .fit()
├── table.rs            # Tabular data + Row/Cell + sections + .grid()
├── tree.rs             # Hierarchical tree with guides
├── rule.rs             # Horizontal divider with title
├── padding.rs          # CSS-style padding (1–4 sides)
├── columns.rs          # Side-by-side column layout
├── layout.rs           # Split-pane layout + 4 splitter types
├── box_drawing.rs      # 17 box/border styles
│
├── progress.rs         # Multi-task progress + track/wrap_file/open
├── progress_columns.rs # 11 progress column types
├── spinner.rs          # 55 animated spinners + get_spinner()
├── status.rs           # Animated spinner + message
├── live.rs             # Auto-updating display + LiveWriter
│
├── syntax.rs           # Syntax highlighting (syntect, 100+ langs)
├── markdown.rs         # Markdown rendering + table + image support
├── json.rs             # Pretty-printed, syntax-highlighted JSON
├── logging.rs          # RichHandler for the log crate
├── log_render.rs       # Standalone LogRender formatter (new in 0.3!)
├── prompt.rs           # 5 interactive prompt types + Select
├── traceback.rs        # Rich exception tracebacks + panic hook
│
├── inspect.rs          # Object introspection (new in 0.3!)
├── control.rs          # Terminal control sequences (new in 0.3!)
├── pretty.rs           # Pretty printing + Node tree traversal
├── emoji.rs            # 100+ :shortcode: → Unicode emoji
├── pager.rs            # System pager ($PAGER / less) + PagerContext
├── constrain.rs        # Width constraint wrapper
├── styled.rs           # Pre-styled renderable wrapper
├── bar.rs              # Horizontal bar chart renderable
├── filesize.rs         # Human-readable file size + speed formatting
├── containers.rs       # Lines + Renderables grouping
├── palette.rs          # Gradient/rainbow/monochrome palettes
├── diagnose.rs         # Error diagnostics + reporting
├── ansi.rs             # ANSI escape sequence decoder
├── scope.rs            # Variable scope rendering
├── file_proxy.rs       # Auto-refreshing file display
├── repr.rs             # RichRepr trait + auto/rich_repr
└── export.rs           # HTML / SVG / text export + 4 themes
```

## 🔬 Compared to Python Rich

| Feature | Python Rich | rusty-rich |
|---|---|---|
| Console + markup | ✅ | ✅ |
| Text / Span / Style (full API) | ✅ | ✅ |
| 256 named colors | ✅ | ✅ |
| Table (colspan/rowspan/sections/grid) | ✅ | ✅ |
| Panel / Rule / Tree | ✅ | ✅ |
| Layout with splitters | ✅ | ✅ |
| Progress (11 column types, track/wrap/open) | ✅ | ✅ |
| Live / Status | ✅ | ✅ |
| Syntax highlighting + lexer guessing | ✅ | ✅ |
| Markdown + tables + images | ✅ | ✅ |
| JSON / Logging / LogRender | ✅ | ✅ |
| Traceback (locals, suppress, panic hook) | ✅ | ✅ |
| Screen / Alt-screen | ✅ | ✅ |
| Prompts (5 types + Select) | ✅ | ✅ |
| Pretty printing + Node tree | ✅ | ✅ |
| Object inspection (Inspect) | ✅ | ✅ (new in 0.3!) |
| Terminal control sequences (Control) | ✅ | ✅ (new in 0.3!) |
| Emoji + ANSI decoder | ✅ | ✅ |
| System pager + PagerContext | ✅ | ✅ |
| Palette (gradient/rainbow) | ✅ | ✅ |
| Bar chart | ✅ | ✅ |
| FileProxy / Scope / Diagnose | ✅ | ✅ |
| Constrain / Styled / Containers | ✅ | ✅ |
| Repr protocol (RichRepr) | ✅ | ✅ |
| Color property accessors | ✅ | ✅ (new in 0.3!) |
| ISO8601/JSON/Path highlighters | ✅ | ✅ (new in 0.3!) |
| Filesize.decimal() (SI units) | ✅ | ✅ (new in 0.3!) |
| Capture / RenderHooks / ThemeContext | ✅ | ✅ |
| 55+ Spinners | 80+ | 55 |
| 17 Box styles | 20 | 17 |
| HTML / SVG / Text export | ✅ | ✅ |
| 170+ Theme styles | 170+ | 170+ |
| Markdown element tree (extensible) | ✅ | — |
| Pygments theme bridge | ✅ | — (uses syntect) |
| Jupyter support | ✅ | N/A |
| **Overall parity** | | **~88%** |

## 🧪 Testing

```bash
cargo test                    # 475 unit tests (all passing)
cargo test --test battle_test # Integration / battle tests
```

## 📄 License

MIT — See [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Inspired by <a href="https://github.com/Textualize/rich">Textualize/rich</a> — the Python library that makes terminal output beautiful.</sub>
</p>
