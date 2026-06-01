# Quick Start

This guide walks through the essentials of rusty-rich. By the end you will know how to create a `Console`, print styled text, and build all the core renderables: Panel, Table, Tree, Rule, and Progress bars.

Add rusty-rich to your `Cargo.toml`:

```toml
[dependencies]
rusty-rich = "0.1"
```

Then import what you need:

```rust
use rusty_rich::*;
```

---

## Console

`Console` is the central rendering engine. It detects terminal capabilities (color depth, width, height) and dispatches renderables to produce styled output.

```rust
use rusty_rich::Console;

let mut console = Console::new();
```

`Console::new()` writes to stdout and auto-detects color support and terminal dimensions. To write elsewhere, use `Console::with_file(...)`.

For quick scripts that do not need a local instance, the free functions `rusty_rich::print_str()` and `rusty_rich::print()` operate on a global console singleton:

```rust
use rusty_rich::print_str;

print_str("[bold cyan]Hello from the global console![/]\n");
```

---

## Printing with Markup

### `console.print_str()`

`print_str()` takes a plain `&str` and interprets **console markup** — a BBCode-like syntax for inline styling. Markup tags set styles until a closing `[/tag]` or `[/]` (close all) is encountered.

Basic tags:

| Tag | Effect |
|-----|--------|
| `[bold]` | Bold text |
| `[dim]` | Dim text |
| `[italic]` | Italic text |
| `[underline]` | Underline text |
| `[blink]` | Blinking text (rarely supported) |
| `[reverse]` | Reverse foreground/background |
| `[strike]` | Strikethrough text |

Color tags use standard names (`red`, `green`, `blue`, `yellow`, `magenta`, `cyan`, `white`, `black`, and their `bright_*` variants). Background colors use `[on <color>]`.

Compose any number of tags in one style:

```rust
// Run this example:
console.print_str("[bold]bold[/bold]\n");
console.print_str("[dim]dim[/dim]\n");
console.print_str("[italic]italic[/italic]\n");
console.print_str("[underline]underline[/underline]\n");
console.print_str("[strike]strike[/strike]\n");
console.print_str("[red]red[/red]\n");
console.print_str("[green on bright_black]green on bright black[/]\n");
console.print_str("[bold yellow on blue]combined[/]\n");
```

Escape a literal `[` with `[[`:

```rust
console.print_str("[[not markup]]\n");
```

### `console.println()`

`println()` takes any **renderable** (a type that implements the `Renderable` trait), renders it, and appends a newline.

```rust
use rusty_rich::Rule;

let rule = Rule::new().title("Section");
console.println(&rule);
```

### Console markup reference

Markup strings are parsed by `rusty_rich::markup::render()` into a `Text` object — a plain string plus a list of styled `Span`s. `print_str()` calls this internally when `options.markup` is `true` (the default).

```rust
// Manual markup rendering:
use rusty_rich::markup;

let text = markup::render("[bold green]Styled[/]");
// text is a rusty_rich::Text — can be passed to any renderable
console.println(&text);
```

---

## Panel

`Panel` draws a bordered container around any content. It supports titles, subtitles, 17 box styles, padding, and border styling.

```rust
use rusty_rich::{Panel, Style, Color, AlignMethod};
use rusty_rich::box_drawing::BOX_ROUNDED;

let panel = Panel::new("Hello, rusty-rich!")
    .title("Greeting")
    .title_align(AlignMethod::Center)
    .border_style(
        Style::new().color(Color::parse("cyan").unwrap()),
    )
    .box_style(BOX_ROUNDED.clone())
    .padding(1, 2, 1, 2);

console.println(&panel);
```

**Builder methods:**

| Method | Description |
|--------|-------------|
| `.title("...")` | Title text in the top border |
| `.subtitle("...")` | Subtitle text in the bottom border |
| `.title_align(AlignMethod::Center)` | Left, Center, Right, or Full |
| `.border_style(Style)` | Style for the border characters |
| `.box_style(BoxStyle)` | Box-drawing character set |
| `.style(Style)` | Default style for content |
| `.padding(top, right, bottom, left)` | Inner padding (default `0, 1, 0, 1`) |
| `.width(n)` | Fixed width |
| `.height(n)` | Fixed height |
| `.fit()` | Do not expand to fill terminal width |

Content can be any `Renderable` — a `&str`, `Text`, `Table`, another `Panel`, etc.

```rust
use rusty_rich::box_drawing::BOX_DOUBLE;

let inner = Panel::new("Nested panels work too")
    .box_style(BOX_DOUBLE.clone())
    .border_style(Style::new().color(Color::parse("yellow").unwrap()))
    .padding(0, 1, 0, 1);

let outer = Panel::new(inner)
    .title("Outer")
    .border_style(Style::new().color(Color::parse("magenta").unwrap()));

console.println(&outer);
```

---

## Table

`Table` renders tabular data with headers, footers, sections, and configurable borders.

```rust
use rusty_rich::{Table, Column, Cell, Style, Color, AlignMethod};

let mut table = Table::new();

table.add_column(
    Column::new("Name")
        .justify(AlignMethod::Left)
        .header_style(Style::new().bold(true).color(Color::parse("cyan").unwrap())),
);
table.add_column(
    Column::new("Age")
        .justify(AlignMethod::Right),
);
table.add_column(
    Column::new("City")
        .justify(AlignMethod::Left),
);

table.add_row(vec![
    "Alice".into(),
    "30".into(),
    "New York".into(),
]);
table.add_row(vec![
    "Bob".into(),
    "25".into(),
    "London".into(),
]);

let table = table
    .title("People")
    .border_style(Style::new().color(Color::parse("bright_black").unwrap()));

console.println(&table);
```

**Column builder methods:**

| Method | Description |
|--------|-------------|
| `.justify(AlignMethod)` | Left, Right, Center, or Full |
| `.header_style(Style)` | Style for the header cell |
| `.style(Style)` | Default cell style for the column |
| `.width(n)` | Fixed column width |
| `.min_width(n)` | Minimum column width |
| `.max_width(n)` | Maximum column width |
| `.ratio(n)` | Proportional width weight |
| `.overflow(OverflowMethod)` | Fold, Crop, Ellipsis, or Ignore |

**Table builder methods:**

| Method | Description |
|--------|-------------|
| `.title("...")` | Title above the table |
| `.caption("...")` | Caption below the table |
| `.border_style(Style)` | Border styling |
| `.box_style(BoxStyle)` | Box-drawing character set |
| `.show_lines()` | Show lines between every row |
| `.hide_header()` | Omit the header row |
| `.leading(n)` | Blank lines between rows |
| `.column(Column)` | Builder-style column addition |
| `.row(row)` | Builder-style row addition |
| `.row_str(vec)` | Builder-style row from `Vec<String>` |

**Grid tables** (no edge, no header):

```rust
let mut grid = Table::grid();
grid.add_column(Column::new("A"));
grid.add_column(Column::new("B"));
grid.add_row_str(vec!["1".into(), "2".into()]);
console.println(&grid);
```

**Sections and colspan/rowspan:**

```rust
use rusty_rich::Cell;

let mut table = Table::new();
table.add_column(Column::new("Item"));
table.add_column(Column::new("Value"));

table.add_row(vec![
    Cell::new("Covers two rows").rowspan(2),
    Cell::new("Row 1"),
]);
table.add_row_str(vec!["Row 2".into()]);

table.add_section(); // horizontal separator before the next row

table.add_row_str(vec!["Done".into(), "✓".into()]);

console.println(&table);
```

---

## Tree

`Tree` renders a hierarchical tree structure with guide lines.

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("root");

// add() returns a &mut Tree for the new child,
// so you can nest immediately:
let src = tree.add("src/");
src.add("main.rs");
src.add("lib.rs");

let tests = tree.add("tests/");
tests.add("integration.rs");

tree.add("Cargo.toml");
tree.add("README.md");

console.println(&tree);
```

Output (conceptually):

```
root
├── src/
│   ├── main.rs
│   └── lib.rs
├── tests/
│   └── integration.rs
├── Cargo.toml
└── README.md
```

**Builder methods:**

| Method | Description |
|--------|-------------|
| `.style(Style)` | Style for the node label |
| `.guide_style(Style)` | Style for the guide lines |
| `.hide_root()` | Omit the root label from output |

```rust
let mut tree = Tree::new("hidden").hide_root();
tree.add("Only children appear");
console.println(&tree);
```

---

## Rule

`Rule` draws a horizontal divider across the terminal, optionally with a centered, left-aligned, or right-aligned title.

```rust
use rusty_rich::{Rule, Style, Color, AlignMethod};

// Plain rule
let rule = Rule::new();
console.println(&rule);

// Rule with centered title
let rule = Rule::new()
    .title(" Section ");
console.println(&rule);

// Styled and aligned
let rule = Rule::new()
    .title("Left")
    .align(AlignMethod::Left)
    .style(Style::new().color(Color::parse("bright_black").unwrap()));
console.println(&rule);

// Custom characters
let rule = Rule::new()
    .title("Heavy")
    .characters("━")
    .style(Style::new().color(Color::parse("cyan").unwrap()));
console.println(&rule);
```

**Builder methods:**

| Method | Description |
|--------|-------------|
| `.title("...")` | Title text embedded in the rule |
| `.characters("...")` | Character(s) to draw the line (default `─`) |
| `.style(Style)` | Style for the rule |
| `.align(AlignMethod)` | Title alignment: Left, Center (default), Right, Full |

---

## Progress Bar

rusty-rich provides two approaches for progress: a standalone `ProgressBar` renderable for simple use, and a `Progress` display for multi-task scenarios with columns.

### Standalone ProgressBar

`ProgressBar` renders a single bar as a string. It is not a `Renderable` (it is drawn directly with its `render(width)` method).

```rust
use rusty_rich::{ProgressBar, Style, Color};

let bar = ProgressBar::new()
    .total(100.0)
    .completed(42.0)
    .complete_style(Style::new().color(Color::parse("green").unwrap()))
    .remaining_style(Style::new().color(Color::parse("bright_black").unwrap()));

let rendered = bar.render(40);
println!("Downloading... {rendered}");
```

**Builder methods:**

| Method | Description |
|--------|-------------|
| `.total(f64)` | Total steps (default 100) |
| `.completed(f64)` | Completed steps (default 0) |
| `.width(usize)` | Bar width in characters |
| `.complete_style(Style)` | Style for the filled portion |
| `.remaining_style(Style)` | Style for the unfilled portion |

### Multi-task Progress with columns

`Progress` manages multiple `Task` instances and renders them with customizable columns.

```rust
use rusty_rich::*;
use std::thread::sleep;
use std::time::Duration;

let mut progress = Progress::new()
    .with_columns(vec![
        Box::new(SpinnerColumn::new()),
        Box::new(TextColumn::new("description")),
        Box::new(BarColumn::new().complete_style(
            Style::new().color(Color::parse("green").unwrap()),
        )),
        Box::new(TaskProgressColumn::new()),
        Box::new(TimeElapsedColumn::new()),
    ]);

let task_id = progress.add_task("Downloading...", Some(100.0));

for i in 0..=100 {
    progress.update(task_id, i as f64);
    print!("\r{}", progress.render(80));
    sleep(Duration::from_millis(50));
}
// Clear the progress line
print!("\r\x1b[K");
```

**Available column types** (in `rusty_rich::progress_columns`):

| Column | Description |
|--------|-------------|
| `SpinnerColumn` | Spinner when running, checkmark when done |
| `TextColumn` | Custom text from `task.fields` |
| `BarColumn` | The progress bar graphic |
| `TaskProgressColumn` | Percentage (e.g. " 42%") |
| `TimeElapsedColumn` | Elapsed time |
| `TimeRemainingColumn` | Estimated time remaining |
| `MofNCompleteColumn` | "completed/total" text |
| `FileSizeColumn` | Completed size in human-readable form |
| `TotalFileSizeColumn` | Total size in human-readable form |
| `DownloadColumn` | "completed/total" with file sizes |
| `TransferSpeedColumn` | Transfer speed (e.g. "1.5 MB/s") |

### TrackIterator

`Progress::track()` wraps an iterator and provides a `TrackIterator` that counts items consumed:

```rust
let mut progress = Progress::new();
let track = progress.track(0..10, "Counting", Some(10.0));

for item in track {
    println!("Got {item}");
    // progress.update(track.progress_id, ...) must be called externally
}
```

### File tracking

`Progress::open()` opens a file and returns a `ProgressFile` that tracks bytes read:

```rust
let mut progress = Progress::new();
let mut pf = progress.open("/path/to/file", "Reading").unwrap();
let mut buf = Vec::new();
pf.read_to_end(&mut buf).unwrap();
pf.sync(&mut progress);
```

---

## Putting It All Together

Here is a complete program that uses every renderable covered in this guide:

```rust
use rusty_rich::*;
use rusty_rich::box_drawing::BOX_ROUNDED;

fn main() {
    let mut console = Console::new();

    // 1. Console markup
    console.print_str("[bold green]rusty-rich Quick Start[/bold green]\n\n");

    // 2. Panel
    let panel = Panel::new("Content inside a panel")
        .title("Panel")
        .box_style(BOX_ROUNDED.clone())
        .border_style(Style::new().color(Color::parse("cyan").unwrap()))
        .padding(1, 2, 1, 2);
    console.println(&panel);

    // 3. Table
    let mut table = Table::new();
    table.add_column(Column::new("Item").justify(AlignMethod::Left));
    table.add_column(Column::new("Qty").justify(AlignMethod::Right));
    table.add_row_str(vec!["Apples".into(), "3".into()]);
    table.add_row_str(vec!["Bananas".into(), "7".into()]);
    let table = table.border_style(Style::new().color(Color::parse("bright_black").unwrap()));
    console.println(&table);

    // 4. Tree
    let mut tree = Tree::new("Project");
    tree.add("src/").add("main.rs");
    tree.add("Cargo.toml");
    console.println(&tree);

    // 5. Rule
    let rule = Rule::new()
        .title("End")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    // 6. Progress bar
    let bar = ProgressBar::new()
        .total(100.0)
        .completed(75.0)
        .complete_style(Style::new().color(Color::parse("green").unwrap()))
        .remaining_style(Style::new().color(Color::parse("bright_black").unwrap()));
    println!("Progress: {}", bar.render(30));
}
```

## Next Steps

- [Style & Color](../core-concepts/style-and-color.md) — full color and text attribute reference
- [Text & Markup](../core-concepts/text-and-markup.md) — the `Text` type and markup syntax details
- [Renderable Protocol](../core-concepts/renderable-protocol.md) — implement `Renderable` for your own types
- [Installation](installation.md) — compatibility requirements
