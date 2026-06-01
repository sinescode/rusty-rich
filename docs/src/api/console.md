# Console API Reference

The `Console` struct is the central rendering engine. It manages terminal
detection, color system support, and dispatching renderables to produce
styled output.

```rust
use rusty_rich::Console;
```

---

## Construction

### `Console::new() -> Self`

Create a new Console writing to `io::stdout()`. Auto-detects the terminal size,
color system (TrueColor / 256 / 16 / standard), and whether output is a
terminal.

```rust
let mut console = Console::new();
```

### `Console::with_file(file: Box<dyn Write + Send>) -> Self`

Create a Console that writes to an arbitrary `Write` target (file, buffer,
`sink`, etc.). Color system is set to `Standard`, terminal size defaults to
80x25, and `is_terminal` is always `false`.

```rust
use std::fs::File;
let file = File::create("output.txt").unwrap();
let mut console = Console::with_file(Box::new(file));
```

### `impl Default for Console`

Delegates to `Console::new()`.

```rust
let mut console = Console::default();   // equivalent to Console::new()
```

---

## Output

### `fn print(&mut self, objects: &[&dyn Renderable], sep: &str, end: &str)`

Print one or more renderable objects, separated by `sep`, ending with `end`.
Honours the `quiet` flag — returns immediately when `self.quiet` is `true`.

```rust
let panel = Panel::new("Content");
console.print(&[&panel], " ", "\n");
```

### `fn println(&mut self, renderable: &dyn Renderable)`

Print a single renderable followed by a newline. Equivalent to
`self.print(&[renderable], " ", "\n")`.

```rust
let table = Table::new();
console.println(&table);
```

### `fn print_str(&mut self, text: &str)`

Print a plain string. When `self.options.markup` is `true` (the default),
the string is parsed as console markup (`[bold red]...[/]`).

```rust
console.print_str("[bold green]Hello, [red]World![/red][/bold green]");
```

### `fn print_json(&mut self, data: &serde_json::Value)`

Pretty-print a JSON value with syntax highlighting (coloured keys, strings,
numbers, booleans, and braces from the theme).

```rust
let value = serde_json::json!({"name": "Alice", "age": 30});
console.print_json(&value);
```

### `fn log(&mut self, objects: &[&dyn Renderable])`

Output a log entry with a dimmed timestamp (`[HH:MM:SS]`) prefix.

```rust
let text = Text::new("Server started");
console.log(&[&text]);
// [14:30:01] Server started
```

### `fn clear(&mut self)`

Clear the entire terminal screen (escape sequence `\\x1b[2J\\x1b[H`).
No-op when `quiet` is `true`.

```rust
console.clear();
```

### `fn line(&mut self, count: usize)`

Output `count` blank lines (newlines).

```rust
console.line(3);    // three blank lines
```

### `fn bell(&mut self)`

Output a terminal bell character (`\\x07`).

```rust
console.bell();
```

---

## Styling

### `fn get_style(&self, name: &str, default: &str) -> Option<Style>`

Look up a style by name from the current theme. If `name` is not found and
`default` is non-empty, parses `default` as a style string and returns it.

```rust
let style = console.get_style("repr.number", "bold cyan");
```

### `fn render_str(&self, text: &str, style: &str) -> Text`

Create a `Text` object from `text` styled with the resolved style name (via
`get_style`). If the style is not found, the text is returned unstyled.

```rust
let text = console.render_str("42", "repr.number");
console.println(&text);
```

### `fn color_ansi(&self, color: &Color) -> String`

Get the ANSI escape sequence for a `Color` as supported by the console's
detected `ColorSystem`. The colour is automatically downgraded
(TrueColor -> 256 -> 16 -> standard) when necessary.

```rust
let ansi = console.color_ansi(&Color::parse("deep_sky_blue1").unwrap());
write!(io::stdout(), "{ansi}text\x1b[0m");
```

### `fn push_theme(&mut self, theme: Theme)`

Push a theme onto the theme stack. The current theme becomes the "inherited"
parent of the new theme, so style lookups fall through.

```rust
let mut custom = Theme::new();
custom.set("my.key", Style::new().bold(true));
console.push_theme(custom);
```

### `fn pop_theme(&mut self)`

Pop the current theme, restoring the previously inherited theme. No-op if
the current theme has no parent.

```rust
console.pop_theme();
```

---

## Layout

### `fn rule(&mut self, title: impl Into<String>, characters: Option<&str>, style: Option<Style>, align: Option<AlignMethod>)`

Draw a horizontal rule (divider line) with an optional title in the middle.
All parameters except `title` are optional.

```rust
// Simple rule
console.rule("", None, None, None);

// Styled rule with custom character
console.rule("Section 1", Some("="), Some(Style::new().color(Color::Cyan)), None);

// Right-aligned title
use rusty_rich::AlignMethod;
console.rule("Summary", None, None, Some(AlignMethod::Right));
```

### `fn set_width(&mut self, width: usize)`

Override the console width. Also updates `self.options.max_width`.

```rust
console.set_width(100);
```

### `fn set_height(&mut self, height: usize)`

Override the console height. Also updates `self.options.max_height`.

```rust
console.set_height(40);
```

### `fn width(&self) -> usize`

Get the effective width: the overridden width if set, otherwise the
auto-detected terminal width.

```rust
let w = console.width();
```

### `fn height(&self) -> usize`

Get the effective height: the overridden height if set, otherwise the
auto-detected terminal height.

```rust
let h = console.height();
```

### `fn set_size(&mut self, width: usize, height: usize)`

Set both width and height at once. Updates `width`, `height`, `max_width`,
`max_height`, and `options.size`.

```rust
console.set_size(120, 30);
```

### `fn render_lines(&self, renderable: &dyn Renderable, options: &ConsoleOptions, style: Option<&Style>, pad: bool) -> Vec<Vec<Segment>>`

Render a renderable into a vector of lines (each line is a vector of
`Segment`). When `style` is `Some`, each segment's style is combined with
the given style. The `pad` parameter is accepted for API compatibility with
Python Rich but currently unused.

```rust
let opts = ConsoleOptions::default();
let lines = console.render_lines(&panel, &opts, None, true);
```

---

## Interactive

### `fn input(&mut self, prompt: &str, password: bool) -> String`

Write `prompt` to the console and read a line from stdin. When `password` is
`true`, input is masked with `*` characters using raw terminal mode
(via `crossterm`). Handles backspace and Ctrl+C.

```rust
let name = console.input("Enter your name: ", false);
let pass = console.input("Enter password: ", true);
```

### `fn show_cursor(&mut self)`

Show the cursor (`\\x1b[?25h`).

```rust
console.show_cursor();
```

### `fn hide_cursor(&mut self)`

Hide the cursor (`\\x1b[?25l`).

```rust
console.hide_cursor();
```

### `fn set_window_title(&mut self, title: &str)`

Set the terminal window title.

```rust
console.set_window_title("My Application");
```

---

## Export

### `fn render(&self, renderable: &dyn Renderable, options: &ConsoleOptions) -> Vec<Segment>`

Recursively render any `Renderable` into a flat `Vec<Segment>`. This is the
core rendering entry point — it handles nested renderables (via
`RenderItem::Nested`), `Group` composition, and any renderable that yields
other renderables.

```rust
let opts = ConsoleOptions::default();
let segments = console.render(&tree, &opts);
for seg in &segments {
    print!("{}", seg.to_ansi());
}
```

### `fn measure(&self, renderable: &dyn Renderable, options: &ConsoleOptions) -> Measurement`

Measure a renderable's width constraints. First checks if the renderable
provides a custom `measure()` implementation; if not, renders the renderable
and measures the maximum segment cell length. Returns a `Measurement` with
`minimum` and `maximum` widths.

```rust
let opts = ConsoleOptions::default();
let m = console.measure(&panel, &opts);
// m.minimum .. m.maximum
```

---

## Screen

### `fn screen(&mut self) -> ScreenContext`

Create a `ScreenContext` that enters the alternate screen buffer. The
context automatically exits the alternate screen when dropped (RAII).

```rust
let ctx = console.screen();
ctx.update("Hello from alternate screen!");
std::thread::sleep(Duration::from_secs(2));
// ctx drops -> exits alt screen
```

### `fn set_alt_screen(&mut self, enable: bool)`

Manually enter or exit the alternate screen buffer by writing the escape
sequences `\\x1b[?1049h` / `\\x1b[?1049l`.

```rust
console.set_alt_screen(true);   // enter alt screen
// ... render full-screen content ...
console.set_alt_screen(false);  // exit alt screen
```

---

## Utility

### `fn is_terminal(&self) -> bool`

Returns `true` if the console is attached to a terminal (TTY).

```rust
if console.is_terminal() {
    console.hide_cursor();
}
```

### `fn quiet(mut self, quiet: bool) -> Self`

Builder-style setter for the `quiet` flag. When `true`, all output methods
(`print`, `println`, `print_str`, `print_json`, `log`, `clear`, `rule`,
`bell`, `line`) return immediately.

```rust
let console = Console::new().quiet(true);
```

### `fn set_quiet(&mut self, quiet: bool)`

Set the `quiet` flag on an existing Console (mutating setter).

```rust
let mut console = Console::new();
console.set_quiet(true);
```

### `fn soft_wrap(mut self, soft_wrap: bool) -> Self`

Builder-style setter for the `soft_wrap` flag. When `true`, text wraps at
word boundaries.

```rust
let console = Console::new().soft_wrap(true);
```

### `fn set_soft_wrap(&mut self, soft_wrap: bool)`

Set the `soft_wrap` flag on an existing Console (mutating setter).

```rust
let mut console = Console::new();
console.set_soft_wrap(true);
```

### `fn on_broken_pipe(&self)`

A documentation point and API compatibility shim with Python Rich. In Rust,
`write()` returns `ErrorKind::BrokenPipe` instead of raising `SIGPIPE`, and
the Console already discards all write errors (including EPIPE) silently
using `let _ = write!(...)`. This method is a no-op.

```rust
console.on_broken_pipe();
```

### `fn begin_capture(&mut self)`

Enter a capture context. In the current implementation this is a placeholder
(no-op). A full implementation would swap `self.file` with a buffer so that
output is captured instead of written.

```rust
console.begin_capture();
// ... output ...
let captured = console.end_capture();
```

### `fn end_capture(&mut self) -> String`

End a capture and return the captured text. Currently returns an empty
`String` (placeholder).

```rust
let text = console.end_capture();
```

---

## Free Functions

### `get_console() -> MutexGuard<'static, Console>`

Get a reference to the global `Console` instance (lazily initialized,
behind a `Mutex`).

```rust
use rusty_rich::get_console;
let mut console = get_console();
console.print_str("[bold]Hello from global console![/]");
```

### `print(objects: &[&dyn Renderable])`

Convenience: print renderable objects using the global console
(aliased as `rusty_rich::print`).

```rust
use rusty_rich::print;
print(&[&Panel::new("Hello")]);
```

### `print_str(text: &str)`

Convenience: print a string (with markup support) using the global console.

```rust
use rusty_rich::print_str;
print_str("[bold cyan]Hello![/]");
```

### `print_json(data: &serde_json::Value)`

Convenience: print formatted JSON using the global console.

```rust
use rusty_rich::print_json;
let data = serde_json::json!({"key": "value"});
print_json(&data);
```

---

## Supporting Types

The Console API relies on the following types (all re-exported from
`rusty_rich`):

| Type | Description |
|------|-------------|
| `ConsoleOptions` | Options passed to renderables during rendering (size, colour, overflow, wrapping, etc.) |
| `ConsoleDimensions` | Terminal dimensions `{ width, height }` |
| `OverflowMethod` | Enum: `Fold`, `Crop`, `Ellipsis`, `Ignore` |
| `ColorSystem` | Enum: `TrueColor`, `EightBit`, `Standard`, `NoColor` |
| `Renderable` | Trait for anything that can be rendered |
| `RenderResult` | Result of rendering: lines of segments plus optional nested items |
| `RenderItem` | A `Segment` or a nested `DynRenderable` |
| `DynRenderable` | Thread-safe, cloneable trait-object wrapper for `Renderable` |
| `Group` | A collection of renderables rendered sequentially |
| `Segment` | A text string with optional `Style` and optional `ControlCode` |
| `Measurement` | Minimum/maximum width constraints for layout |
| `ScreenContext` | Alternate screen buffer RAII context |
| `Theme` | Named style map with optional inheritance |

