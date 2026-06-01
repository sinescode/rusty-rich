# Console

The `Console` is the central rendering engine of rusty-rich. It manages terminal
detection, color system auto-detection, output dispatching, and provides all
high-level rendering methods. Every interaction with rusty-rich begins with a
`Console` instance.

## Creating a Console

### `Console::new()` -- Default Console (stdout)

The simplest way to get a Console is `Console::new()`, which writes to stdout,
auto-detects the terminal dimensions, and determines the best color system.

```rust
use rusty_rich::Console;

let mut console = Console::new();
```

### `Console::with_file()` -- File Output

When you need to render rich output to a file (or any `Write` target), use
`with_file`. The Console will disable terminal detection and use standard
ANSI codes without color degradation, assuming a 80x25 viewport.

```rust
use std::fs::File;
use rusty_rich::Console;

let file = File::create("output.ans").unwrap();
let mut console = Console::with_file(Box::new(file));
```

Any type implementing `Write + Send` can be used -- `io::stdout()`,
`io::sink()`, `io::BufWriter`, network streams, etc.

```rust
use std::io;
use rusty_rich::Console;

// Write to nowhere (useful for tests or dry runs)
let mut console = Console::with_file(Box::new(io::sink()));
```

### `get_console()` -- Global Console

For convenience, rusty-rich maintains a static global Console instance. This
is useful when you don't want to thread a Console through every function.

```rust
use rusty_rich::get_console;

fn log_message(msg: &str) {
    let mut console = get_console();
    console.print_str(&format!("[dim]{}[/dim]\n", msg));
}
```

The global console is lazily initialized on first access and protected by a
`Mutex`. Calling `get_console()` returns a `MutexGuard` that automatically
unlocks when it goes out of scope.

---

## Color System Auto-Detection

When `Console::new()` is called, it runs `detect_color_system()` which checks,
in order:

1. **`COLORTERM` environment variable** -- if set to `"truecolor"` or
   `"24bit"`, TrueColor is selected immediately.
2. **`TERM` environment variable** -- if it contains `"256color"`,
   EightBit (256-color) mode is selected. `"xterm-kitty"` gets TrueColor.
3. **`NO_COLOR` / `CLICOLOR`** -- if `NO_COLOR` is set, the system degrades
   to Standard (16-color) mode.
4. **Fallback** -- if stdout is a terminal, TrueColor is preferred;
   otherwise Standard.

The detected `ColorSystem` is stored in `console.color_system` and used by
`color_ansi()` to downgrade colors to what the terminal supports.

```rust
use rusty_rich::{Console, ColorSystem};

let console = Console::new();
match console.color_system {
    ColorSystem::TrueColor => eprintln!("Terminal supports 16.7M colors"),
    ColorSystem::EightBit  => eprintln!("Terminal supports 256 colors"),
    ColorSystem::Standard  => eprintln!("Terminal supports 16 colors"),
}
```

To force a specific color system, construct a Console and manipulate the field
directly or rely on the environment variables above.

---

## Terminal Detection

The Console detects whether output is connected to a terminal (tty) using the
`atty` crate. Access this via `is_terminal()`:

```rust
use rusty_rich::Console;

let console = Console::new();
if console.is_terminal() {
    console.print_str("[green]Connected to a terminal[/green]\n");
} else {
    // Piped output; suppress ANSI or adjust formatting
}
```

When constructed with `with_file()`, `is_terminal()` always returns `false`.

---

## Width and Height

The Console auto-detects terminal dimensions via `terminal_size`. You can
read or override them:

### Reading Dimensions

```rust
use rusty_rich::Console;

let console = Console::new();
let w = console.width();   // e.g. 80
let h = console.height();  // e.g. 24
```

Dimensions fall back to 80x25 if detection fails (e.g., piped output).

### Setting Dimensions

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.set_size(120, 40);
console.set_width(100);   // only override width
console.set_height(30);   // only override height

assert_eq!(console.width(), 100);
assert_eq!(console.height(), 30);
```

When you call `set_width()` or `set_height()`, the values are stored as
overrides. The internal `ConsoleOptions` is also updated so renderables see
the new dimensions.

---

## Print Methods

### `print()` -- Render Multiple Objects

Prints one or more renderable objects separated by `sep` and terminated by
`end`. Each object is rendered to ANSI in turn.

```rust
use rusty_rich::{Console, Panel, Style, Color};

let mut console = Console::new();

let panel = Panel::new("Hello")
    .border_style(Style::new().color(Color::parse("cyan").unwrap()));

let text = "[bold]World[/bold]";

// Print both, separated by " | ", ending with "\n"
console.print(&[&panel, &text], " | ", "");
console.print(&[&"\n"], "", "");
```

### `println()` -- Print a Single Renderable

Shorthand for printing one renderable followed by a newline.

```rust
use rusty_rich::{Console, Panel, Style, Color};

let mut console = Console::new();

let panel = Panel::new("Hello")
    .border_style(Style::new().color(Color::parse("cyan").unwrap()));
console.println(&panel);
```

### `print_str()` -- Print with Markup

Prints a plain string with console markup support (when `options.markup` is
`true`, which is the default). This is the most common way to output styled
text.

```rust
use rusty_rich::Console;

let mut console = Console::new();

// Markup is interpreted by default
console.print_str("[bold green]Success:[/bold green] Operation completed.\n");
console.print_str("[red]Error:[/red] [italic]file not found[/italic]\n");

// Escaping: double-bracket to print literal brackets
console.print_str("Print [[literal brackets]] in markup mode.\n");
```

When markup is disabled, the string is written verbatim:

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.options.markup = false;
console.print_str("[bold]This shows as-is, not bold[/bold]\n");
```

### `print_json()` -- Format and Print JSON

Pretty-prints a `serde_json::Value` with syntax highlighting using the theme's
JSON styles (keys in cyan, strings in green, numbers in bright blue, etc.).

```rust
use rusty_rich::Console;
use serde_json::json;

let mut console = Console::new();

let data = json!({
    "name": "rusty-rich",
    "version": "0.1.0",
    "features": ["tables", "trees", "panels"],
    "active": true,
    "meta": null
});

console.print_json(&data);
```

---

## Logging

### `log()` -- Timestamped Log Entry

Prints a log-style line with a timestamp and dim styling, followed by one
or more renderable objects. Equivalent to Python Rich's `Console.log()`.

```rust
use rusty_rich::Console;

let mut console = Console::new();

console.log(&[&"[bold]Starting[/bold] application"]);
// Output: [14:32:01] Starting application
//        (with dim timestamp styling)

// You can mix renderable types
use rusty_rich::{Panel, Style, Color};
let panel = Panel::new("Server listening on port 8080")
    .border_style(Style::new().color(Color::parse("green").unwrap()));
console.log(&[&"INFO", &panel]);
```

The timestamp is produced by `chrono::Local::now()` in `%H:%M:%S` format.

---

## Rules, Bell, Line, Clear

### `rule()` -- Horizontal Rule

Draws a horizontal rule (divider) with an optional title. You can customize
the characters, style, and alignment.

```rust
use rusty_rich::{Console, Style, Color, AlignMethod};

let mut console = Console::new();

// Simple rule
console.rule("", None, None, None);

// Rule with centered title
console.rule(
    "Section 1",
    None,
    Some(Style::new().color(Color::parse("yellow").unwrap())),
    None,
);

// Rule with custom characters and right alignment
console.rule(
    "End",
    Some("="),
    Some(Style::new().color(Color::parse("cyan").unwrap())),
    Some(AlignMethod::Right),
);

// Rule with left alignment
console.rule(
    "Notes",
    Some("─"),
    Some(Style::new().color(Color::parse("bright_black").unwrap())),
    Some(AlignMethod::Left),
);
```

### `bell()` -- Terminal Bell

Sends the ASCII bell character (`\x07`) to the terminal. On most terminals
this produces a short beep or flash.

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.bell();
```

### `line()` -- Blank Lines

Prints one or more blank lines.

```rust
use rusty_rich::Console;

let mut console = Console::new();

console.print_str("First line\n");
console.line(2);  // two blank lines
console.print_str("After a gap\n");
```

### `clear()` -- Clear Screen

Clears the terminal screen by writing the ANSI escape sequence `\x1b[2J\x1b[H`
(clear entire screen, move cursor to home position).

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.clear();
```

---

## Cursor Control

### `show_cursor()` / `hide_cursor()`

Shows or hides the terminal cursor. Useful for full-screen or live-updating
displays.

```rust
use rusty_rich::Console;
use std::thread::sleep;
use std::time::Duration;

let mut console = Console::new();

console.print_str("Processing...\n");
console.hide_cursor();

// Do some work
sleep(Duration::from_secs(2));

console.show_cursor();
console.print_str("Done.\n");
```

Always re-show the cursor before your program exits, or the cursor may remain
invisible in the user's terminal.

---

## Window Title

### `set_window_title()`

Sets the terminal window title using the OSC 0 escape sequence.

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.set_window_title("My Rust App v1.0");
```

The title is restored to its previous value when the terminal session ends
(most terminals automatically handle this).

---

## Quiet Mode

When `quiet` is `true`, all output methods (`print`, `println`, `print_str`,
`print_json`, `log`, `rule`, `bell`, `line`, `clear`) are silently suppressed.
This is useful for a `--quiet` / `-q` CLI flag.

```rust
use rusty_rich::Console;

let mut console = Console::new();

// Builder-style (consumes self)
let console = Console::new().quiet(true);

// Setter-style (mutable reference)
let mut console = Console::new();
console.set_quiet(true);

// Check the flag
assert!(console.quiet);

// All output is suppressed
console.print_str("[bold]This will not appear[/bold]\n");    // silent
console.println(&"Also silent");                               // silent
console.log(&[&"Silent too"]);                                 // silent

// Re-enable
console.set_quiet(false);
```

---

## Soft Wrap

When `soft_wrap` is `true`, text wraps at word boundaries rather than
character boundaries. This only takes effect when the renderable supports
word-level wrapping.

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.set_soft_wrap(true);

// With a narrow width, long lines will break at word boundaries
console.set_width(20);
console.print_str("This is a long line that should wrap at word boundaries.\n");
```

```rust
// Builder-style
let console = Console::new().soft_wrap(true);
```

---

## Theme Stack

### `push_theme()` / `pop_theme()`

The Console maintains a theme stack. `push_theme()` replaces the current theme
with a new one that inherits from the previous theme. `pop_theme()` restores
the previous theme.

```rust
use rusty_rich::{Console, Theme, Style, Color};

let mut console = Console::new();

// Create a custom theme
let mut dark_theme = Theme::new();
dark_theme.set(
    "custom.header",
    Style::new().color(Color::parse("bright_cyan").unwrap()).bold(true),
);
dark_theme.set(
    "custom.body",
    Style::new().color(Color::parse("white").unwrap()),
);

// Push it onto the stack
console.push_theme(dark_theme);

// Render with the custom theme
let header_style = console.get_style("custom.header", "");
if let Some(style) = header_style {
    console.print_str(&format!("{}Header text{}\n", style.to_ansi(), Style::new().reset_ansi()));
}

// Restore the previous theme
console.pop_theme();

// After pop, the custom style no longer resolves
let header_style = console.get_style("custom.header", "");
assert!(header_style.is_none());
```

Internally, `push_theme` wraps the old theme inside the new theme's `inherit`
field, forming a linked list. `pop_theme` unwraps one level.

---

## Input

### `input()` -- Read User Input

Reads a line of input from stdin, with an optional prompt written to the
console output.

```rust
use rusty_rich::Console;

let mut console = Console::new();

let name = console.input("Enter your name: ", false);
console.print_str(&format!("[green]Hello, {}![/green]\n", name));
```

### Password Input

When `password` is `true`, the input is masked with `*` characters using
raw terminal mode via `crossterm`. Backspace, Enter, and Ctrl+C are handled.

```rust
use rusty_rich::Console;

let mut console = Console::new();

let password = console.input("Enter password: ", true);
console.print_str("[green]Password accepted.[/green]\n");
```

Note: Password mode requires a real terminal. If raw mode cannot be enabled,
it falls back to unmasked input.

---

## Screen (Alternate Screen Buffer)

### `screen()` -- Alternate Screen Context

Enters the terminal's alternate screen buffer. The `ScreenContext` returned
automatically exits the alternate screen when dropped.

```rust
use rusty_rich::Console;
use std::thread::sleep;
use std::time::Duration;

let mut console = Console::new();

// Enter alternate screen
let ctx = console.screen();

// Update the screen content
ctx.update("[bold green]Full screen content![/bold green]");
sleep(Duration::from_secs(2));

// ctx drops here -> exits alternate screen automatically
```

### `set_alt_screen()` -- Manual Alternate Screen

If you prefer to manage the alternate screen yourself, use `set_alt_screen()`.

```rust
use rusty_rich::Console;
use std::thread::sleep;
use std::time::Duration;

let mut console = Console::new();

console.set_alt_screen(true);    // Enter alternate screen
console.print_str("[bold]Custom full-screen view[/bold]\n");
sleep(Duration::from_secs(2));
console.set_alt_screen(false);   // Exit alternate screen
```

The `ScreenContext` approach is preferred since it is panic-safe (it
implements `Drop`).

---

## Capturing Output

### `begin_capture()` / `end_capture()`

These methods provide a framework for capturing rendered output in a buffer
instead of writing to the underlying file. The current implementation is a
placeholder; the Console writes directly to its file.

```rust
use rusty_rich::Console;

let mut console = Console::new();

// Begin capture (swaps internal writer with a buffer)
console.begin_capture();

console.print_str("[bold]Captured text[/bold]\n");

// End capture and retrieve the text
let captured = console.end_capture();
// captured contains the ANSI output
```

For deterministic capture in tests, create a Console with a `Vec<u8>` buffer:

```rust
use rusty_rich::Console;

let mut buffer = Vec::new();
let mut console = Console::with_file(Box::new(&mut buffer));
console.print_str("[bold]Test output[/bold]\n");
// buffer now contains the ANSI bytes
```

---

## Render, Render Lines, and Measure

These lower-level methods give you direct access to the rendering pipeline.

### `render()` -- Recursive Rendering

Renders a renderable by recursively flattening nested items into a flat list
of `Segment`s. This handles `Group` composition and any renderable that yields
other renderables.

```rust
use rusty_rich::{
    Console, ConsoleOptions, Renderable, Group,
    Style, Color,
};

let mut console = Console::new();

// Group multiple renderables
let mut group = Group::new();
group.add("[bold]First[/bold]");
group.add("\n");
group.add("[green]Second[/green]");

let segments = console.render(&group, &console.options);
// segments is Vec<Segment> -- all nested renderables resolved

// Convert to ANSI string
let ansi: String = segments.iter().map(|s| s.to_ansi()).collect();
```

### `render_lines()` -- Segmented Lines

Renders a renderable and returns the result as `Vec<Vec<Segment>>`, where each
inner vector is one line. Optionally applies a `Style` to all segments.

```rust
use rusty_rich::{
    Console, ConsoleOptions, Style, Color,
};

let mut console = Console::new();

let text = "[bold red]Hello[/bold red] [blue]World[/blue]";

let lines = console.render_lines(
    &text,
    &console.options,
    None,      // no additional style
    true,      // pad to width
);

for (i, line) in lines.iter().enumerate() {
    let ansi: String = line.iter().map(|s| s.to_ansi()).collect();
    println!("Line {}: {}", i, ansi);
}
```

To apply a style to every segment:

```rust
let lines = console.render_lines(
    &text,
    &console.options,
    Some(&Style::new().dim(true)),  // dim everything
    true,
);
```

### `measure()` -- Width Measurement

Measures the minimum and maximum width of a renderable. This is used during
layout calculations (columns, tables, etc.) to determine how much space each
element needs.

```rust
use rusty_rich::{
    Console, ConsoleOptions, Measurement,
    Table, Column,
};

let mut console = Console::new();

let text = "Hello, World!";
let measurement: Measurement = console.measure(&text, &console.options);
println!("Min width: {}, Max width: {}", measurement.minimum, measurement.maximum);
// Output: Min width: 13, Max width: 80

// Measure a panel
use rusty_rich::Panel;
let panel = Panel::new("Short text")
    .padding(1, 1, 0, 0);
let m = console.measure(&panel, &console.options);
// The measurement accounts for padding + content
```

### `get_style()` -- Theme Style Lookup

Looks up a named style from the console's theme, with a fallback default.

```rust
use rusty_rich::Console;

let console = Console::new();

// Look up a theme style
let style = console.get_style("json.key", "");         // uses theme default
let style = console.get_style("custom.missing", "bold red"); // fallback string
```

### `render_str()` -- Styled String

Converts a plain string into a `Text` object with an optional style applied
from the theme.

```rust
use rusty_rich::Console;

let console = Console::new();
let styled_text = console.render_str("Error: disk full", "logging.level.error");
```

---

## Broken Pipe Handling

### `on_broken_pipe()`

In Python Rich, `SIGPIPE` must be handled explicitly. In Rust, broken pipes
produce `ErrorKind::BrokenPipe` from `write()` calls, which are not fatal.
All Console write operations use `let _ = write!(...)` which silently
discards all write errors including EPIPE. This method is provided for API
compatibility.

```rust
use rusty_rich::Console;

let console = Console::new();
console.on_broken_pipe();  // no-op in Rust
```

---

## `ConsoleDimensions` and `ConsoleOptions`

### `ConsoleDimensions`

Represents the size of the terminal in character cells.

```rust
use rusty_rich::ConsoleDimensions;

// Auto-detect from terminal
let dims = ConsoleDimensions::detect();
println!("{}x{}", dims.width, dims.height);

// Manual construction
let dims = ConsoleDimensions { width: 120, height: 40 };
```

### `ConsoleOptions`

Options passed to renderables during rendering. Contains terminal size,
encoding, width/height constraints, overflow handling, justification, and more.

```rust
use rusty_rich::ConsoleOptions;

let opts = ConsoleOptions::default();
println!("max_width: {}, max_height: {}", opts.max_width, opts.max_height);
```

Key fields:

| Field           | Type                        | Default          | Description                              |
|-----------------|-----------------------------|------------------|------------------------------------------|
| `size`          | `ConsoleDimensions`         | auto-detected    | Terminal size in cells                   |
| `is_terminal`   | `bool`                      | true             | Whether output is a tty                  |
| `encoding`      | `String`                    | `"utf-8"`        | Output encoding                          |
| `min_width`     | `usize`                     | 1                | Minimum render width                     |
| `max_width`     | `usize`                     | terminal width   | Maximum render width                     |
| `max_height`    | `usize`                     | terminal height  | Maximum render height                    |
| `justify`       | `Option<AlignMethod>`       | `None`           | Text justification override              |
| `overflow`      | `Option<OverflowMethod>`    | `None`           | Overflow behavior override               |
| `no_wrap`       | `bool`                      | false            | Disable text wrapping                    |
| `ascii_only`    | `bool`                      | false            | Use ASCII-only box characters            |
| `markup`        | `bool`                      | true             | Enable markup interpretation             |
| `highlight`     | `bool`                      | true             | Enable syntax highlighting of strings    |
| `height`        | `Option<usize>`             | `None`           | Fixed height override                    |
| `legacy_windows`| `bool`                      | false            | Legacy Windows console compatibility     |

Helper methods:

```rust
use rusty_rich::ConsoleOptions;

let opts = ConsoleOptions::default();

// Create a copy with a different width
let narrow = opts.update_width(40);

// Create a copy with fixed height
let fixed = opts.update_height(10);

// Shrink width by an amount (for padding)
let padded = opts.shrink_width(4);
// padded.max_width == opts.max_width - 4
```

### `OverflowMethod`

Controls how text that exceeds the available width is handled:

| Variant     | Behavior                                     |
|-------------|----------------------------------------------|
| `Fold`      | Wrap text onto the next line                 |
| `Crop`      | Truncate text at the boundary                |
| `Ellipsis`  | Truncate and append "..."                    |
| `Ignore`    | Let text overflow without clipping           |

---

## Summary

The `Console` is the central hub that ties together every rusty-rich feature.
Understanding its methods -- from `new()` and `with_file()` for construction,
through `print`, `print_str`, `print_json` and `log` for output, to `render`,
`render_lines`, and `measure` for direct rendering access -- gives you full
control over terminal output in your Rust applications.
