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
  <a href="#"><img src="https://img.shields.io/badge/tests-197%20passed-brightgreen" alt="tests"></a>
</p>

---

## ✨ Features

- 🎨 **Style** — foreground/background colors, bold, italic, underline, dim, blink, reverse, strikethrough, overline, conceal, frame, encircle, links
- 📝 **Console markup** — `[bold red]text[/bold red]` BBCode-like inline styling
- 🎯 **256 named colors** — full ANSI 256-color palette with aliases, hex, RGB, auto-downgrade
- 🎭 **170+ theme styles** — repr, json, markdown, logging, traceback, rule, bar, progress, table, tree, syntax, prompt categories

- 📊 **Table** — tabular data with headers, footers, colspan/rowspan, column alignment, sections, 17 box styles
- 🌲 **Tree** — hierarchical tree rendering with Unicode guides
- 📦 **Panel** — bordered containers with titles, subtitles
- ➖ **Rule** — horizontal dividers with optional titles
- 📐 **Padding & Align** — CSS-style padding and alignment helpers
- 📋 **Columns** — side-by-side layout
- 🗂️ **Layout** — recursive split-pane layout with ratio sizing

- ⏳ **Progress** — multi-task progress bars with 11 column types, file tracking, `track()` iterator
- 🔄 **Spinner** — 55 animated spinners with case-insensitive name lookup
- 📌 **Status** — spinner + message with in-place update
- 🔄 **Live** — auto-updating displays with alt-screen, transient mode, stdout/stderr redirect via `LiveWriter`

- 🌈 **Syntax highlighting** — powered by syntect (100+ languages)
- 📝 **Markdown** — headings, code blocks, lists, blockquotes, links, **tables**
- 📋 **JSON** — pretty-printed, syntax-highlighted JSON
- 🔍 **Logging** — Rich-formatted log records via the `log` crate

- 🖼️ **Box drawing** — 17 box styles (rounded, square, heavy, double, ASCII, etc.)
- 🖥️ **Screen / Alt-screen** — full-screen terminal applications with `ScreenContext`
- ⌨️ **Prompts** — `Prompt`, `IntPrompt`, `FloatPrompt`, `Confirm`, `Select<T>`, password mode
- 🔴 **Traceback** — rich exception rendering with locals, source code, frame suppression, panic hook
- 📤 **HTML & SVG export** — capture console output for the web
- 🧩 **Segment utilities** — simplify, split_lines, strip_styles, strip_links, align, divide, set_shape, filter_control

## 📦 Installation

```toml
[dependencies]
rusty-rich = "0.2"
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

## 📂 Module Map

```
src/
├── lib.rs              # Crate root + re-exports
├── console.rs          # Central rendering engine
├── screen.rs           # Full-screen / alt-screen / ScreenContext
├── color.rs            # TrueColor / 256 / Standard (256 names)
├── style.rs            # 13 attributes + hyperlinks + metadata
├── theme.rs            # 170+ named styles + stack
├── segment.rs          # Segment + 9 utility functions
├── text.rs             # Text with Span styling
├── cells.rs            # Unicode cell width utilities
├── measure.rs          # Width measurement protocol
├── align.rs            # Horizontal + vertical alignment
├── ratio.rs            # Proportional space distribution
├── markup.rs           # BBCode-like markup parser
├── highlighter.rs      # Regex/Repr highlighters
│
├── panel.rs            # Bordered container
├── table.rs            # Tabular data + colspan/rowspan
├── tree.rs             # Hierarchical tree
├── rule.rs             # Horizontal divider
├── padding.rs          # CSS-style padding
├── columns.rs          # Side-by-side layout
├── layout.rs           # Split-pane layout
├── box_drawing.rs      # 17 box/border styles
│
├── progress.rs         # Multi-task progress + track()
├── progress_columns.rs # 11 progress column types
├── spinner.rs          # 55 animated spinners
├── status.rs           # Spinner + message
├── live.rs             # Auto-updating display + LiveWriter
│
├── syntax.rs           # Syntax highlighting (syntect)
├── markdown.rs         # Markdown rendering + table support
├── json.rs             # Pretty-printed JSON
├── logging.rs          # log crate integration
├── prompt.rs           # 5 interactive prompt types
├── traceback.rs        # Rich exception tracebacks
└── export.rs           # HTML / SVG / text export
```

## 🔬 Compared to Python Rich

| Feature | Python Rich | rusty-rich |
|---|---|---|
| Console + markup | ✅ | ✅ |
| Text / Span / Style | ✅ | ✅ |
| 256 named colors | ✅ | ✅ |
| Table (colspan/rowspan) | ✅ | ✅ |
| Panel / Rule / Tree | ✅ | ✅ |
| Layout / Columns | ✅ | ✅ |
| Progress (11 column types) | ✅ | ✅ |
| Live / Status | ✅ | ✅ |
| Syntax highlighting | ✅ | ✅ |
| Markdown (incl. tables) | ✅ | ✅ |
| JSON / Logging | ✅ | ✅ |
| Traceback (locals, suppress) | ✅ | ✅ |
| Screen / Alt-screen | ✅ | ✅ |
| Prompts (5 types) | ✅ | ✅ |
| 55+ Spinners | 80+ | 55 |
| 17 Box styles | 20 | 17 |
| HTML / SVG export | ✅ | ✅ |
| Segment utilities | ✅ | ✅ |
| LiveWriter / redirect | ✅ | ✅ |
| 170+ Theme styles | 170+ | 170+ |
| Pretty / Inspect | ✅ | — |
| Emoji / ANSI decoder | ✅ | — |
| Jupyter support | ✅ | — |
| **Overall parity** | | **~72%** |

## 🧪 Testing

```bash
cargo test                    # 197 unit tests
cargo test --test battle_test # Integration / battle tests
```

## 📄 License

MIT — See [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Inspired by <a href="https://github.com/Textualize/rich">Textualize/rich</a> — the Python library that makes terminal output beautiful.</sub>
</p>
