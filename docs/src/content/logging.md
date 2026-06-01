# Logging

`RichHandler` provides Rich-formatted log output by integrating with the `log` crate. It formats log records with colored levels, timestamps, source locations, and optional console markup, exactly like Python Rich's `RichHandler`.

```rust
use rusty_rich::RichHandler;

let mut handler = RichHandler::new();
handler.emit(&record);
```

---

## RichHandler

The core struct. It holds a `Console` instance and a set of formatting flags that control the rendered output of every log record.

### Constructor

```rust
pub fn new() -> Self
```

Creates a handler with default settings:

| Field | Default | Description |
|-------|---------|-------------|
| `show_time` | `true` | Prepend a dim `[HH:MM:SS]` timestamp. |
| `show_level` | `true` | Prepend a colored, space-padded level name (e.g. `INFO `). |
| `show_path` | `true` | Append a dim italic `[file:line]` location. |
| `enable_link_path` | `false` | Render `file:line` as a clickable terminal hyperlink. |
| `markup` | `false` | Interpret `[style]` markup tags inside log messages. |

```rust
// Default handler — most common starting point
let mut handler = RichHandler::new();
```

---

### Fields

All configuration fields are public and can be set directly after construction.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `console` | `Console` | `Console::new()` | The console used for output at emit time. |
| `show_time` | `bool` | `true` | Show a dim `[HH:MM:SS]` before each record. |
| `show_level` | `bool` | `true` | Show the log level name with level-specific colour. |
| `show_path` | `bool` | `true` | Show the source file and line number. |
| `enable_link_path` | `bool` | `false` | Emit `file:line` as an OSC-8 hyperlink. |
| `markup` | `bool` | `false` | Parse Rich markup tags in the message body. |
| `highlighter` | `ReprHighlighter` | `ReprHighlighter::new()` | Highlighter applied to the message text. |

```rust
let mut handler = RichHandler::new();
handler.show_time = false;         // suppress timestamps
handler.markup = true;             // enable console markup in messages
handler.enable_link_path = true;   // clickable file:line links
```

---

### show_time

When `true`, a dimmed timestamp is prepended to every log record in the format `[HH:MM:SS]`.

```rust
let mut handler = RichHandler::new();
handler.show_time = true;    // default

// Disable timestamps for cleaner output in pipelines
handler.show_time = false;
```

Output with `show_time: true`:

```
[14:30:01] INFO  Server listening on 0.0.0.0:8080 [src/main.rs:42]
```

The timestamp uses the local system clock via `chrono::Local::now()` and is formatted with `Style::new().dim(true)` so it recedes into the background.

---

### show_level

When `true`, the log level name is rendered in a level-specific colour, right-padded to five characters for alignment.

| Level | Colour | Style |
|-------|--------|-------|
| `ERROR` | Red | Bold |
| `WARN` | Yellow | Normal |
| `INFO` | Green | Normal |
| `DEBUG` | Blue | Normal |
| `TRACE` | Bright black | Normal |

```rust
let mut handler = RichHandler::new();
handler.show_level = true;    // default

handler.show_level = false;   // suppress level labels
```

Output showing level-coloured prefixes:

```
ERROR Server crashed: out of memory   [src/main.rs:88]
WARN  Disk usage above 90%            [src/monitor.rs:34]
INFO  Server listening on 0.0.0.0:8080 [src/main.rs:42]
DEBUG Loaded config in 12ms           [src/config.rs:15]
TRACE Entering parse_config           [src/config.rs:10]
```

The colour mapping is defined by the `style_level()` function, which returns a `Style` for each `log::Level` variant.

---

### show_path

When `true`, the source file and line number are appended in dim italic, enclosed in square brackets.

```rust
let mut handler = RichHandler::new();
handler.show_path = true;    // default

handler.show_path = false;   // suppress source location
```

Output with `show_path: true`:

```
INFO  Request processed in 4ms [src/routes/user.rs:142]
```

The location uses the `file!()` and `line!()` macros provided by the `log` crate's `Record`. When either value is missing, the location suffix is omitted entirely.

---

### enable_link_path

When `true`, the `file:line` portion is emitted as an OSC-8 terminal hyperlink, making it clickable in terminals that support hyperlinks (kitty, iTerm2, WezTerm, Windows Terminal, etc.).

```rust
let mut handler = RichHandler::new();
handler.enable_link_path = true;
```

This has no effect when `show_path` is `false`.

---

### markup

When `true`, the message body is parsed as Rich console markup, allowing inline styling inside log messages.

```rust
let mut handler = RichHandler::new();
handler.markup = true;
```

With `markup: true`, the following log call:

```rust
log::info!("[bold green]Connected[/bold green] to [underline]database[/underline]");
```

Renders the log message with bold green text for "Connected" and underlined text for "database".

**Important:** When `markup` is `false` (the default), markup tags are rendered literally as plain text — the `[]` brackets are not interpreted. Set `markup: true` only when you control the log messages and trust their content, because user-supplied data containing bracket characters could produce unexpected styling.

---

### highlighter

A `ReprHighlighter` instance that applies syntax-highlighting-style colouring to the message text. The highlighter is applied to the message before any markup parsing (if `markup` is also enabled).

```rust
use rusty_rich::{RichHandler, ReprHighlighter};

let mut handler = RichHandler::new();
handler.highlighter = ReprHighlighter::new();
```

---

## emit()

The primary method that accepts a `log::Record` and writes it to the console.

```rust
pub fn emit(&mut self, record: &log::Record)
```

```rust
handler.emit(&record);
```

`emit` calls `render()` to build the formatted string, writes it to the console's output via `writeln!`, and flushes the output handle. The output goes to `self.console.file`, which by default is `std::io::stdout`.

---

## render()

Produces the formatted string for a single log record without writing it. Useful when you need to capture or further transform the output.

```rust
pub fn render(
    &self,
    level: log::Level,
    message: &str,
    module_path: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
) -> String
```

```rust
let output = handler.render(
    log::Level::Info,
    "Server started",
    Some("my_app"),
    Some("src/main.rs"),
    Some(42),
);
assert!(output.contains("INFO"));
assert!(output.contains("Server started"));
```

The output format follows this order:

```
[HH:MM:SS] LEVEL  Message text [file:line]
```

Each section is conditionally included based on the handler's `show_time`, `show_level`, and `show_path` flags.

---

## style_level()

A standalone helper that returns the `Style` associated with a given log level.

```rust
pub fn style_level(level: log::Level) -> Style
```

```rust
use rusty_rich::logging::style_level;
use rusty_rich::Style;
use log::Level;

let err_style: Style = style_level(Level::Error);
// err_style is bold red
```

| Level | Style |
|-------|-------|
| `Error` | Bold red |
| `Warn` | Yellow |
| `Info` | Green |
| `Debug` | Blue |
| `Trace` | Bright black |

---

## Integration with the `log` crate

`RichHandler` works with Rust's standard `log` crate facade. The typical integration pattern is to create a `RichHandler`, wrap it in a `log::LevelFilter`, and install it as the global logger.

### Basic setup — manual dispatch

```rust
use log::{LevelFilter, Record};
use rusty_rich::RichHandler;

fn main() {
    let mut handler = RichHandler::new();

    // Log a record directly
    log::info!("Application started");
    log::warn!("Configuration file not found, using defaults");
    log::error!("Failed to bind socket: address in use");
}
```

### Global logger setup

For use as the application-wide logger, implement `log::Log`:

```rust
use log::{LevelFilter, Log, Record, SetLoggerError};
use rusty_rich::RichHandler;
use std::sync::Mutex;

pub struct RichLogger {
    handler: Mutex<RichHandler>,
}

impl RichLogger {
    pub fn new() -> Self {
        Self {
            handler: Mutex::new(RichHandler::new()),
        }
    }

    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        let logger = Box::new(Self::new());
        log::set_boxed_logger(logger)?;
        log::set_max_level(level);
        Ok(())
    }
}

impl Log for RichLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Ok(mut handler) = self.handler.lock() {
            handler.emit(record);
        }
    }

    fn flush(&self) {
        // Handler flushes after every write
    }
}

fn main() {
    RichLogger::init(LevelFilter::Debug)
        .expect("Failed to install logger");

    log::info!("Rich logging is ready");
    log::debug!("Debug output is enabled");
}
```

---

### Full example — customised logger

```rust
use log::{LevelFilter, Log, Record, SetLoggerError};
use rusty_rich::RichHandler;
use std::sync::Mutex;

struct RichLogger {
    handler: Mutex<RichHandler>,
}

impl RichLogger {
    fn new() -> Self {
        let mut handler = RichHandler::new();
        // Enable markup in log messages
        handler.markup = true;
        // Show clickable file:line links
        handler.enable_link_path = true;
        Self {
            handler: Mutex::new(handler),
        }
    }

    fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        let logger = Box::new(Self::new());
        log::set_boxed_logger(logger)?;
        log::set_max_level(level);
        Ok(())
    }
}

impl Log for RichLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Ok(mut handler) = self.handler.lock() {
            handler.emit(record);
        }
    }

    fn flush(&self) {}
}

fn main() {
    // Install as global logger
    RichLogger::init(LevelFilter::Info)
        .expect("Failed to install Rich logger");

    // These messages are rendered with Rich formatting
    log::info!("[bold]Server[/bold] listening on [cyan]0.0.0.0:8080[/cyan]");
    log::warn!("Disk usage at [yellow]92%[/yellow] — consider cleaning up");
    log::error!("Connection pool exhausted");
}
```

---

### Minimal example — inline handler

When a global logger is not required, a `RichHandler` can be used directly:

```rust
use log::Record;
use rusty_rich::RichHandler;

fn main() {
    let mut handler = RichHandler::new();

    // Simulate log records by calling emit directly
    handler.emit(&Record::builder()
        .args(format_args!("Hello from Rich logging"))
        .level(log::Level::Info)
        .module_path(Some("my_app"))
        .file(Some("src/main.rs"))
        .line(Some(10))
        .build());
}
```

---

## Output examples

Default settings (`show_time: true`, `show_level: true`, `show_path: true`):

```
[14:30:01] ERROR Server crashed: out of memory   [src/main.rs:88]
[14:30:01] WARN  Disk usage above 90%            [src/monitor.rs:34]
[14:30:02] INFO  Server listening on 0.0.0.0:8080 [src/main.rs:42]
[14:30:03] DEBUG Loaded config in 12ms           [src/config.rs:15]
[14:30:03] TRACE Entering parse_config           [src/config.rs:10]
```

With `show_time: false`:

```
ERROR Server crashed: out of memory   [src/main.rs:88]
WARN  Disk usage above 90%            [src/monitor.rs:34]
INFO  Server listening on 0.0.0.0:8080 [src/main.rs:42]
```

With `show_path: false`:

```
[14:30:01] ERROR Server crashed: out of memory
[14:30:01] WARN  Disk usage above 90%
[14:30:02] INFO  Server listening on 0.0.0.0:8080
```

With `show_level: false` and `show_path: false`:

```
[14:30:01] Server crashed: out of memory
[14:30:01] Disk usage above 90%
[14:30:02] Server listening on 0.0.0.0:8080
```

---

## Import paths

```rust
use rusty_rich::RichHandler;              // The handler struct
use rusty_rich::logging::style_level;     // Style helper for log levels
use rusty_rich::ReprHighlighter;          // Default highlighter
```
