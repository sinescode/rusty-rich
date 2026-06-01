# Tracebacks

Traceback rendering provides Rich-formatted exception tracebacks, complete with
source code context, local variable tables, and styled box-drawn borders. It
mirrors Python Rich's `rich.traceback.Traceback` renderable.

---

## Data types

Three data types capture the structure of a traceback: `Frame`, `Stack`, and
`Trace`.

### Frame

A single frame in a traceback, corresponding to one call on the call stack.

```rust
#[derive(Debug, Clone)]
pub struct Frame {
    pub filename: String,
    pub lineno: usize,
    pub name: String,
    pub line: Option<String>,
    pub locals: Option<HashMap<String, String>>,
    pub last_instruction: Option<String>,
}
```

Fields:

| Field | Type | Description |
|---|---|---|
| `filename` | `String` | Path to the source file |
| `lineno` | `usize` | 1-based line number |
| `name` | `String` | Function or method name |
| `line` | `Option<String>` | The source line text (used when the file is unavailable) |
| `locals` | `Option<HashMap<String, String>>` | Local variable names and their string representations |
| `last_instruction` | `Option<String>` | (reserved) Last instruction before the error |

```rust
use rusty_rich::traceback::Frame;

let frame = Frame::new("src/main.rs", 42, "parse_config")
    .line("let config = parse_file(path);");
```

### Stack

A single exception level in a traceback, containing the exception type, value,
and an ordered list of frames.

```rust
#[derive(Debug, Clone)]
pub struct Stack {
    pub exc_type: Option<String>,
    pub exc_value: Option<String>,
    pub syntax_error: Option<String>,
    pub is_cause: bool,
    pub frames: Vec<Frame>,
    pub notes: Vec<String>,
    pub is_group: bool,
    pub exceptions: Vec<Stack>,
}
```

Builder methods:

```rust
let stack = Stack::new()
    .exc_type("ValueError")
    .exc_value("invalid input")
    .add_frame(Frame::new("lib.rs", 10, "validate"));
```

### Trace

A complete trace, holding one or more `Stack` entries. Multiple stacks represent
chained exceptions (`raise ... from ...` in Python terms).

```rust
#[derive(Debug, Clone)]
pub struct Trace {
    pub stacks: Vec<Stack>,
}
```

```rust
let trace = Trace::from_stack(stack);
// or
let trace = Trace::new(); // empty
```

---

## The Traceback renderable

`Traceback` is the main renderable that produces the formatted output. It
combines `Trace` data with layout configuration.

```rust
pub struct Traceback { /* private fields */ }
```

### Constructors

#### `Traceback::new(trace)`

Create from a `Trace` value:

```rust
use rusty_rich::traceback::{Traceback, Trace, Stack, Frame};

let stack = Stack::new()
    .exc_type("FileNotFoundError")
    .exc_value("config.toml not found")
    .add_frame(Frame::new("src/main.rs", 15, "load_config"));

let traceback = Traceback::new(Trace::from_stack(stack));
```

#### `Traceback::from_exception(exc_type, exc_value, frames)`

Convenience constructor that builds a single-stack traceback directly from an
exception type, value, and frame list:

```rust
let tb = Traceback::from_exception(
    "ArithmeticError",
    "division by zero",
    vec![
        Frame::new("src/calc.rs", 22, "divide"),
        Frame::new("src/main.rs",  8, "main"),
    ],
);
```

This is equivalent to manually constructing a `Stack`, wrapping it in a `Trace`,
and calling `Traceback::new(...)`.

### Builder methods

All builder methods consume and return `Self`, enabling a chained API.

| Method | Default | Description |
|---|---|---|
| `width(n)` | console max (capped at 120) | Total output width in characters |
| `code_width(n)` | derived from width | Width reserved for source code lines |
| `extra_lines(n)` | `3` | Number of context lines above and below the error line |
| `theme(name)` | `None` (default theme) | Named theme to use for styling |
| `word_wrap(bool)` | `false` | Whether to wrap long source lines |
| `show_locals(bool)` | `false` | Show local variables at each frame |
| `indent_guides(bool)` | `false` | Show indentation guides in source code |
| `locals_max_length(n)` | `10` | Maximum number of locals to show per frame |
| `locals_max_string(n)` | `80` | Maximum length of a local value string |
| `locals_max_depth(n)` | `5` | Maximum depth for nested local values |
| `locals_hide_dunder(bool)` | `true` | Hide `__dunder__`-style local variables |
| `locals_hide_sunder(bool)` | `false` | Hide `_sunder`-style local variables |
| `suppress(paths)` | `[]` | Hide frames whose filename starts with or contains any of these strings |
| `max_frames(n)` | `None` (unlimited) | Maximum number of frames to display |

```rust
let tb = Traceback::from_exception("Error", "something broke", frames)
    .width(100)
    .extra_lines(5)
    .theme("monokai")
    .word_wrap(true)
    .show_locals(true)
    .locals_max_length(20)
    .locals_max_string(120)
    .locals_hide_sunder(true)
    .suppress(vec!["/rustc".to_string(), "/library/std".to_string()])
    .max_frames(8);
```

#### suppress

The `suppress` method accepts a list of string patterns. A frame is hidden if
its `filename` starts with or contains any of the patterns. This is useful for
hiding frames from standard library or dependency code, similar to Python Rich's
`suppress` parameter.

```rust
let tb = Traceback::from_exception("Error", "msg", frames)
    .suppress(vec![
        "/rustc/".to_string(),
        "/library/std".to_string(),
        "vendor/".to_string(),
    ]);
```

When frames are hidden, a summary line like `"  ... 3 frames hidden ..."` is
displayed at the end of the frame list.

#### max_frames

Limits the number of frames rendered. When set, only the first `n` frames
(after suppression) are shown. Any remaining frames are reported in the hidden
frames summary.

---

## Theme keys

Traceback rendering uses the following theme keys. Customise them via a
`Theme` to change colours and styles.

| Constant | Key | Default style |
|---|---|---|
| `TRACEBACK_BORDER` | `"traceback.border"` | red |
| `TRACEBACK_TITLE` | `"traceback.title"` | bold |
| `TRACEBACK_ERROR` | `"traceback.error"` | bold bright_red |
| `TRACEBACK_ERROR_MARK` | `"traceback.error_mark"` | bold bright_red |
| `TRACEBACK_FILENAME` | `"traceback.filename"` | cyan |
| `TRACEBACK_LINE_NO` | `"traceback.line_no"` | bright_black |
| `TRACEBACK_LOCALS_HEADER` | `"traceback.locals_header"` | bold |

```rust
use rusty_rich::theme::{default_theme, names};

let mut theme = default_theme();
theme.set(names::TRACEBACK_BORDER, Style::from_str("bright_yellow"));
theme.set(names::TRACEBACK_FILENAME, Style::from_str("bold cyan"));
// ... pass theme to Console or Traceback
```

---

## `install()` -- global panic hook

`install()` registers a global Rust panic hook that renders Rich-formatted
tracebacks to stderr whenever a panic occurs. This replaces the default panic
handler.

```rust
use rusty_rich::traceback;

traceback::install();

// Any panic from this point will produce a styled traceback
panic!("something unexpected happened");
```

The hook captures:

- The panic message (via `downcast_ref`)
- The file, line, and column from `panic_info.location()`
- A frame constructed from the panic location

It constructs a `Traceback` with `extra_lines(0)` (no source context since the
source file may not be accessible) and renders it to stderr via ANSI escape
codes.

You can remove the hook later with `std::panic::take_hook()`:

```rust
let _ = std::panic::take_hook();
```

---

## Rendering

Like all Rich renderables, `Traceback` implements the `Renderable` trait. Pass
it to `Console::print()` or call `render()` directly.

### Via Console

```rust
use rusty_rich::{Console, traceback::{Traceback, Trace, Stack, Frame}};

let mut console = Console::new();

let tb = Traceback::from_exception(
    "RuntimeError",
    "something went wrong",
    vec![Frame::new("src/main.rs", 10, "main")],
)
.width(80);

console.print(tb);
```

### Direct rendering

```rust
use rusty_rich::console::ConsoleOptions;
use rusty_rich::traceback::{Traceback, Trace, Stack, Frame};

let tb = Traceback::from_exception("Error", "msg", vec![
    Frame::new("test.rs", 5, "func"),
])
.width(80);

let opts = ConsoleOptions {
    max_width: 80,
    ..ConsoleOptions::default()
};

let result = tb.render(&opts);
let ansi = result.to_ansi();
println!("{}", ansi);
```

---

## Examples

### Example 1: Captured error with source context

When a source file is available on disk, the traceback reads it and displays the
offending line with surrounding context.

```rust
use rusty_rich::{Console, traceback::{Traceback, Frame}};

let mut console = Console::new();

// Simulate an error in a real file
let tb = Traceback::from_exception(
    "ParseError",
    "unexpected token at line 5, column 12",
    vec![
        Frame::new("src/config.rs", 15, "parse_token"),
        Frame::new("src/config.rs", 42, "parse_file"),
        Frame::new("src/main.rs",   8, "main"),
    ],
)
.width(100)
.extra_lines(4);

console.print(tb);
```

Output (with source files present):

```
╭─ Traceback (most recent call last) ──────────────────────────────────────╮
│                                                                          │
│   src/config.rs:15 in parse_token                                        │
│   ╭────────────────────────────────────────────────────────────────────╮  │
│   │ 11 │ fn parse_token(input: &str) -> Token {                        │  │
│   │ 12 │     let first = input.chars().next();                         │  │
│   │ 13 │     match first {                                             │  │
│   │ 14 │         Some(ch) => {                                         │  │
│   │  ❱ 15 │             Token::from_char(ch)                           │  │
│   │ 16 │         }                                                     │  │
│   │ 17 │         None => return Token::Eof,                            │  │
│   │ 18 │     }                                                         │  │
│   │ 19 │ }                                                             │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│                                                                          │
│   src/config.rs:42 in parse_file                                         │
│   ╭────────────────────────────────────────────────────────────────────╮  │
│   │ 38 │ fn parse_file(path: &str) -> Config {                         │  │
│   │ 39 │     let content = read_file(path);                            │  │
│   │ 40 │     let mut tokens = Vec::new();                              │  │
│   │ 41 │     for line in content.lines() {                             │  │
│   │  ❱ 42 │         tokens.push(parse_token(line));                    │  │
│   │ 43 │     }                                                         │  │
│   │ 44 │     Config::from_tokens(tokens)                               │  │
│   │ 45 │ }                                                             │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│                                                                          │
│   src/main.rs:8 in main                                                  │
│   ╭────────────────────────────────────────────────────────────────────╮  │
│   │ 4  │ fn main() {                                                   │  │
│   │ 5  │     let args = parse_args();                                  │  │
│   │ 6  │     if let Some(path) = args.config {                         │  │
│   │ 7  │         println!("Loading config...");                        │  │
│   │  ❱ 8  │         let config = parse_file(&path);                    │  │
│   │ 9  │         println!("Config loaded");                            │  │
│   │ 10 │     }                                                         │  │
│   │ 11 │ }                                                             │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│                                                                          │
│   ParseError: unexpected token at line 5, column 12                      │
│                                                                          │
╰──────────────────────────────────────────────────────────────────────────╯
```

The `❱` marker highlights the exact line where the error occurred. Line numbers
are shown in a dim style to keep focus on the code.

### Example 2: Custom traceback with locals

Enable `show_locals(true)` to display local variables at each frame. Locals
appear in a nested sub-box below the source context.

```rust
use std::collections::HashMap;
use rusty_rich::{Console, traceback::{Traceback, Frame}};

let mut console = Console::new();

let mut locals_main = HashMap::new();
locals_main.insert("path".to_string(), "String(\"./config.toml\")".to_string());
locals_main.insert("args".to_string(), "Args { verbose: false, config: Some(\"./config.toml\") }".to_string());
locals_main.insert("debug".to_string(), "false".to_string());

let mut locals_parse = HashMap::new();
locals_parse.insert("content".to_string(), "String(len=2048)".to_string());
locals_parse.insert("tokens".to_string(), "Vec<Token>(len=42)".to_string());
locals_parse.insert("line_no".to_string(), "15".to_string());

let tb = Traceback::from_exception(
    "ParseError",
    "unexpected token at line 15",
    vec![
        Frame::new("src/parser.rs", 32, "parse_file")
            .locals(locals_parse),
        Frame::new("src/main.rs", 10, "main")
            .locals(locals_main),
    ],
)
.width(100)
.show_locals(true)
.locals_max_length(10)
.locals_max_string(80)
.locals_hide_dunder(true);

console.print(tb);
```

Output (approximate):

```
╭─ Traceback (most recent call last) ──────────────────────────────────────╮
│                                                                          │
│   src/parser.rs:32 in parse_file                                         │
│   ╭────────────────────────────────────────────────────────────────────╮  │
│   │  ❱ 32 │     let token = parse_token(line, line_no);               │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│   ╭─ locals ───────────────────────────────────────────────────────────╮  │
│   │ content = String(len=2048)                                         │  │
│   │ line_no = 15                                                       │  │
│   │ tokens = Vec<Token>(len=42)                                        │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│                                                                          │
│   src/main.rs:10 in main                                                 │
│   ╭────────────────────────────────────────────────────────────────────╮  │
│   │  ❱ 10 │     let cfg = parse_file(&path);                          │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│   ╭─ locals ───────────────────────────────────────────────────────────╮  │
│   │ args = Args { verbose: false, config: Some(\"./config.toml\") }    │  │
│   │ debug = false                                                      │  │
│   │ path = String(\"./config.toml\")                                   │  │
│   ╰────────────────────────────────────────────────────────────────────╯  │
│                                                                          │
│   ParseError: unexpected token at line 15                                │
│                                                                          │
╰──────────────────────────────────────────────────────────────────────────╯
```

### Example 3: Chained exceptions

Multiple stacks in a `Trace` represent chained exceptions. Each stack is
rendered with its own frames and exception message.

```rust
use rusty_rich::traceback::{Traceback, Trace, Stack, Frame};

let inner_stack = Stack::new()
    .exc_type("IOError")
    .exc_value("No such file or directory")
    .add_frame(Frame::new("src/io.rs", 22, "read_file"));

let outer_stack = Stack::new()
    .exc_type("ConfigError")
    .exc_value("Failed to load configuration")
    .is_cause(true)
    .add_frame(Frame::new("src/main.rs", 15, "load_config"));

let trace = Trace {
    stacks: vec![inner_stack, outer_stack],
};

let tb = Traceback::new(trace).width(80);
```

### Example 4: Suppressing internal frames

Hide standard library and framework frames to focus on application code:

```rust
let tb = Traceback::from_exception(
    "NullPointerException",
    "dereference of None value",
    all_frames,
)
.width(100)
.suppress(vec![
    "/rustc/".to_string(),
    "/library/".to_string(),
    "framework/src/".to_string(),
]);
```

Frames matching any suppress pattern are omitted from the output. A summary
line shows how many were hidden:

```
  ... 4 frames hidden ...
```

---

## Rendering behaviour

### Source file lookup

When a frame has a `filename` that exists on disk, the traceback reads the
file and renders `extra_lines` of context above and below the error line. If
the file cannot be read (e.g. it was compiled from a different machine, or
the source has been deleted), the frame falls back to displaying the `line`
field as plain text.

### Locals filtering

When `show_locals` is enabled, local variables are filtered and truncated:

- `locals_hide_dunder` (default `true`): hides variables named like `__foo__`.
- `locals_hide_sunder` (default `false`): hides variables starting with `_`.
- `locals_max_length` limits the number of visible variables (oldest first).
- `locals_max_string` truncates string representations longer than this limit,
  appending `...`.
- `locals_max_depth` limits recursion depth when rendering nested values.

### Word wrapping

When `word_wrap` is enabled, source lines longer than the available code width
are wrapped. When disabled (the default), lines are truncated to fit.

---

## Summary

| Type / Function | Purpose |
|---|---|
| `Frame` | A single call frame (file, line, function, optional locals) |
| `Stack` | One exception level (exception type, value, frames) |
| `Trace` | A collection of stacks (supports chained exceptions) |
| `Traceback` | The renderable that produces formatted output |
| `Traceback::new(trace)` | Create from a `Trace` |
| `Traceback::from_exception(type, value, frames)` | Convenience constructor for simple cases |
| Builder methods (`.width()`, `.show_locals()`, etc.) | Configure layout and filtering |
| `install()` | Register a Rich-formatted global panic hook |
| Theme keys (`traceback.border`, etc.) | Customise colours via a `Theme` |
