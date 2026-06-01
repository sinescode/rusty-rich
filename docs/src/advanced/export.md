# Export (HTML / SVG / Text)

The Rust-Rich console can export rendered output to several formats beyond plain
ANSI text. You can export a styled snapshot of your console output as:
- **HTML** -- with inline CSS styles representing colours and font attributes
- **SVG** -- a vector image that mimics a terminal window, suitable for sharing
  on social media, documentation, or bug reports
- **Text** -- plain unstyled text with ANSI escape sequences stripped

---

## TerminalTheme

Before diving into export methods, it helps to understand `TerminalTheme`. This
struct describes the foreground and background colours of the terminal so that
colour calculations (blending, downgrading) can produce correct results for the
target medium.

```rust
#[derive(Debug, Clone, Copy)]
pub struct TerminalTheme {
    pub foreground_color: (u8, u8, u8),  // RGB tuple
    pub background_color: (u8, u8, u8),  // RGB tuple
}
```

The default is white-on-black (`(255, 255, 255)` foreground, `(0, 0, 0)`
background).

### Built-in themes

The following constants are available for common terminal colour schemes:

| Constant | Foreground | Background | Description |
|---|---|---|---|
| `DEFAULT_TERMINAL_THEME` | `(255, 255, 255)` | `(0, 0, 0)` | White on black |
| `MONOKAI` | `(215, 215, 215)` | `(39, 40, 34)` | Monokai Pro-like |
| `DIMMED_MONOKAI` | `(160, 160, 160)` | `(35, 35, 30)` | Dimmed monokai |
| `NIGHT_OWLISH` | `(214, 222, 235)` | `(1, 22, 39)` | Night Owl theme |
| `SVG_EXPORT_THEME` | `(215, 215, 215)` | `(39, 40, 34)` | Default theme for SVG export |

```rust
use rusty_rich::color::{TerminalTheme, DEFAULT_TERMINAL_THEME, MONOKAI};

// Use the default
let theme = DEFAULT_TERMINAL_THEME;

// Use monokai
let monokai = MONOKAI;
```

---

## Console export methods

The `Console` provides four pairs of export methods. Each pair consists of an
`export_*` method that returns a string, and a `save_*` method that writes the
result to a file.

### `export_html` / `save_html`

Renders the current console output as an HTML document. Colours are converted to
inline CSS, and font attributes (bold, italic, underline, etc.) are preserved.

```rust
pub fn export_html(&self, theme: Option<&TerminalTheme>, title: Option<&str>) -> String
pub fn save_html(&self, path: &str, theme: Option<&TerminalTheme>, title: Option<&str>) -> std::io::Result<()>
```

- `theme` -- optional `TerminalTheme` to use for background/foreground colours.
  Falls back to `DEFAULT_TERMINAL_THEME` when `None`.
- `title` -- an optional page title for the `<title>` element.

The generated HTML includes a `<pre>` block with inline CSS, plus a `<style>`
block with the base terminal colours.

### `export_svg` / `save_svg`

Renders the console output as an SVG image that looks like a terminal window
with a title bar, rounded corners, and a dark background. This is the format
commonly seen in Rich's screenshots shared on Twitter/X and GitHub.

```rust
pub fn export_svg(
    &self,
    title: &str,
    theme: Option<&TerminalTheme>,
    font_family: Option<&str>,
    font_size: Option<f64>,
) -> String

pub fn save_svg(
    &self,
    path: &str,
    title: &str,
    theme: Option<&TerminalTheme>,
    font_family: Option<&str>,
    font_size: Option<f64>,
) -> std::io::Result<()>
```

- `title` -- the text shown in the terminal window's title bar.
- `theme` -- optional `TerminalTheme`. Uses `SVG_EXPORT_THEME` when `None`.
- `font_family` -- CSS font family (default: `"Consolas, 'Courier New', monospace"`).
- `font_size` -- font size in pixels (default: `14.0`).

The SVG dimensions are calculated automatically based on the console width and
height.

### `export_text` / `save_text`

Strips all ANSI escape sequences and styles, returning only the plain text
content. Useful for search indexing, plain-text logs, or further processing.

```rust
pub fn export_text(&self) -> String
pub fn save_text(&self, path: &str) -> std::io::Result<()>
```

---

## Capture context manager

`Capture` is a context manager that redirects console output to an internal
buffer instead of the terminal. When the context exits, the captured text
(including ANSI escape sequences) is returned as a `String`.

```rust
use rusty_rich::{Console, Capture};

let mut console = Console::new();

// Enter capture mode
console.begin_capture();

// All output here goes to the buffer instead of stdout
console.print_str("[bold green]Hello[/bold green] world!");

// End capture and retrieve the buffered output
let captured = console.end_capture();
// captured now contains the ANSI-styled string
```

### Raw capture vs styled capture

By default, the captured output retains ANSI escape sequences so you can write
it to a file or a pager later. If you only need the text content, you can strip
the escapes or use `export_text` on the captured console.

### Using Capture as a wrapper struct

The `Capture` struct wraps a `Console` and swaps the output writer with a
`Vec<u8>` buffer. Calling `Capture::end()` returns the buffered content and
restores the original output target.

```rust
use rusty_rich::{Console, Capture};

let mut console = Console::new();
let mut capture = Capture::new(&mut console);

// Output is captured
capture.print_str("[bold]Captured[/bold] output");

// Retrieve the content
let result = capture.end();
```

---

## Examples

### Exporting a table to HTML

The following example creates a styled table, renders it to the console, and
then exports the result as an HTML file.

```rust
use rusty_rich::{
    Console, Table, Column, AlignMethod, Color, Style,
};
use rusty_rich::color::DEFAULT_TERMINAL_THEME;

fn main() -> std::io::Result<()> {
    let mut console = Console::new();

    // Build a table
    let mut table = Table::new()
        .title("Widget Inventory")
        .border_style(Style::new().color(Color::parse("cyan").unwrap()));

    table.add_column(
        Column::new("Item").justify(AlignMethod::Left)
    );
    table.add_column(
        Column::new("Quantity").justify(AlignMethod::Right)
    );
    table.add_column(
        Column::new("Price").justify(AlignMethod::Right)
    );

    table.add_row(vec![
        "Widget A".into(),
        "42".into(),
        "$9.99".into(),
    ]);
    table.add_row(vec![
        "Widget B".into(),
        "17".into(),
        "$24.95".into(),
    ]);
    table.add_row(vec![
        "Widget C".into(),
        "8".into(),
        "$149.00".into(),
    ]);

    // Render to the console (optional -- the table is rendered during
    // export regardless)
    console.println(&table);

    // Export to HTML
    console.save_html(
        "widget_inventory.html",
        Some(&DEFAULT_TERMINAL_THEME),
        Some("Widget Inventory Report"),
    )?;

    println!("Exported to widget_inventory.html");
    Ok(())
}
```

The resulting HTML file will contain a self-contained document with the table
rendered using inline CSS. Open it in any browser to view the styled output
without a terminal.

### Exporting to SVG

```rust
use rusty_rich::{
    Console, Panel, Style, Color,
};
use rusty_rich::color::SVG_EXPORT_THEME;

fn main() -> std::io::Result<()> {
    let mut console = Console::new();

    let panel = Panel::new("Hello, SVG export!")
        .title("Greeting")
        .border_style(Style::new().color(Color::parse("green").unwrap()));

    console.println(&panel);

    console.save_svg(
        "greeting.svg",
        "My Terminal Output",
        Some(&SVG_EXPORT_THEME),
        None,  // default font family
        None,  // default font size
    )?;

    println!("Exported to greeting.svg");
    Ok(())
}
```

### Capturing console output as a string

```rust
use rusty_rich::{Console, Capture};

fn main() {
    let mut console = Console::new();

    // Begin capturing
    console.begin_capture();

    // All output goes to the capture buffer
    console.print_str("[bold green]Success![/bold green] The operation");
    console.print_str(" completed without errors.\n");

    // End capture and retrieve the string
    let output = console.end_capture();

    // `output` contains the ANSI-styled string.
    // You can write it to a file, send it over the network, etc.
    std::fs::write("captured_output.ans", &output).expect("write failed");

    // Or strip styles for plain text:
    let plain = console.export_text();
    std::fs::write("captured_output.txt", &plain).expect("write failed");
}
```

### Saving plain text output

```rust
use rusty_rich::Console;

fn main() -> std::io::Result<()> {
    let mut console = Console::new();

    console.print_str("[bold]Section 1[/bold]\n");
    console.print_str("Some content here.\n");
    console.print_str("[bold]Section 2[/bold]\n");
    console.print_str("More content.\n");

    // Save plain text (no ANSI codes)
    console.save_text("output.txt")?;

    Ok(())
}
```

---

## Summary

| Method | Returns | Writes to file | Description |
|---|---|---|---|
| `export_html` | `String` | -- | Exports output as an HTML document with inline CSS |
| `save_html` | -- | `path` | Writes HTML export to a file |
| `export_svg` | `String` | -- | Exports output as an SVG terminal image |
| `save_svg` | -- | `path` | Writes SVG export to a file |
| `export_text` | `String` | -- | Strips ANSI codes, returns plain text |
| `save_text` | -- | `path` | Writes plain text to a file |
| `begin_capture` | -- | -- | Redirects console output to an internal buffer |
| `end_capture` | `String` | -- | Returns buffered output, restores original output target |

| Terminal theme | Foreground | Background |
|---|---|---|
| `DEFAULT_TERMINAL_THEME` | `(255, 255, 255)` | `(0, 0, 0)` |
| `MONOKAI` | `(215, 215, 215)` | `(39, 40, 34)` |
| `DIMMED_MONOKAI` | `(160, 160, 160)` | `(35, 35, 30)` |
| `NIGHT_OWLISH` | `(214, 222, 235)` | `(1, 22, 39)` |
| `SVG_EXPORT_THEME` | `(215, 215, 215)` | `(39, 40, 34)` |
