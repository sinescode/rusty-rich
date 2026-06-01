# Screen

`Screen` provides full-screen rendering and alternate screen buffer management. It encompasses three components:

- **`Screen`** — a renderable that crops or pads its child content to exactly fill the terminal dimensions.
- **`ScreenContext`** — a RAII guard that enters the alternate screen buffer and automatically exits it when dropped.
- **`ScreenUpdate`** — a type used to replace the content shown in a `Screen` or `ScreenContext` without constructing a new screen.

```rust
use rusty_rich::Screen;
use rusty_rich::ScreenContext;
use rusty_rich::ScreenUpdate;
```

All three types are re-exported from the crate root.

---

## Screen renderable

`Screen` is a `Renderable` that wraps a child renderable and ensures the output fills the entire terminal. If the child content is smaller than the terminal, the remaining area is padded with blank lines. If the child content is larger, it is cropped.

This makes `Screen` the correct choice for any full-screen layout: the rendering pipeline always produces exactly `width` columns by `height` rows of output, with no gaps or overflow.

```rust
use rusty_rich::Screen;

let screen = Screen::new("Hello from full-screen!");
```

### Constructor

#### `Screen::new(renderable)`

Creates a new `Screen` wrapping the given renderable.

| Parameter | Type | Description |
|-----------|------|-------------|
| `renderable` | `impl Renderable + Send + Sync + 'static` | The content to display full-screen |

```rust
let screen = Screen::new(my_panel);
```

### Builder methods

#### `.style(style: Style)`

Sets an optional background / padding style. When set, every blank cell used for padding inherits this style, and content segments have this style combined on top of their own styles.

```rust
use rusty_rich::Style;

let screen = Screen::new("Content")
    .style(Style::new().bg("navy_blue"));
```

#### `.application_mode(mode: bool)`

When `true`, line endings use `\n\r` (CR+LF) instead of the default `\n` (LF). This is useful for raw terminal modes that require carriage returns.

```rust
let screen = Screen::new("Raw content")
    .application_mode(true);
```

### Methods

#### `.update(update)`

Replaces the child renderable with a new one. Accepts anything that implements `Into<ScreenUpdate>` — any `Renderable + Send + Sync + 'static`, or a `ScreenUpdate` value.

```rust
let mut screen = Screen::new("Initial content");
screen.update("Replaced content");
```

### Render behavior

When `Screen::render()` is called, it:

1. Determines the terminal size from `ConsoleOptions`.
2. Renders the child renderable at that size.
3. Applies the optional style to all content segments.
4. Crops or pads each line to exactly `width` columns (using space characters).
5. Crops or pads the total number of lines to exactly `height` rows.
6. Inserts newline segments between lines (not after the last).

The result is a `RenderResult` that, when converted to ANSI, produces a block of text that fills the visible terminal area exactly.

---

## ScreenContext via Console::screen()

`ScreenContext` is a RAII guard that manages the terminal's alternate screen buffer. It is created by calling `Console::screen()`.

```rust
use rusty_rich::Console;

let mut console = Console::new();
let ctx = console.screen();
// Alternate screen is now active
ctx.update("Hello from the alternate screen!");
std::thread::sleep(std::time::Duration::from_secs(2));
// ctx drops here → alternate screen is exited automatically
```

### Constructor

`ScreenContext` is not typically constructed directly. Instead, call `Console::screen()`:

```rust
pub fn screen(&mut self) -> ScreenContext
```

`Console::screen()` creates a `ScreenContext` and immediately calls `enter()` to activate the alternate screen buffer. The returned context is ready to use.

You can also construct one manually with `ScreenContext::new()`, but it starts inactive — you must call `enter()` explicitly.

### Methods

#### `.enter()`

Enters the alternate screen buffer by writing `\x1b[?1049h` to stdout and flushing. Safe to call multiple times — subsequent calls are no-ops if already active.

```rust
let mut ctx = ScreenContext::new();
ctx.enter();
```

#### `.exit()`

Exits the alternate screen buffer by writing `\x1b[?1049l` to stdout and flushing, restoring the original screen content. Safe to call multiple times — subsequent calls are no-ops if already inactive.

```rust
ctx.exit();
```

#### `.update(update) -> io::Result<()>`

Renders the given content in the alternate screen. If the alternate screen is not yet active, `enter()` is called first. The content is wrapped in a `Screen` renderable and written to stdout.

Accepts anything that implements `Into<ScreenUpdate>` — any `Renderable + Send + Sync + 'static`, or a `ScreenUpdate`.

```rust
ctx.update("New content")?;
```

Returns `std::io::Result<()>` — the write or flush can fail.

#### `.is_active() -> bool`

Returns `true` if the alternate screen buffer is currently active.

```rust
if ctx.is_active() {
    // We are in the alternate screen
}
```

#### `.style(style: Style) -> Self`

Builder: sets a style that will be applied to all content rendered via `update()`.

```rust
let ctx = ScreenContext::new()
    .style(Style::new().bg("black"));
```

### Escape sequences

The alternate screen is controlled by two standard DEC private mode sequences:

| Action | Sequence |
|--------|----------|
| Enter alternate screen | `\x1b[?1049h` |
| Exit alternate screen | `\x1b[?1049l` |

`Console::set_alt_screen(enable)` is also available if you need lower-level control without a `ScreenContext`:

```rust
console.set_alt_screen(true);   // Enter alternate screen
// ... render manually ...
console.set_alt_screen(false);  // Exit alternate screen
```

---

## Auto-exit on Drop

`ScreenContext` implements `Drop`. When the context goes out of scope, `exit()` is called automatically, restoring the original terminal content.

```rust
fn show_splash() {
    let mut console = Console::new();
    let ctx = console.screen();
    ctx.update("Splash screen").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    // ctx drops here → exit is automatic
}
```

This means you can use `ScreenContext` with scoping to guarantee cleanup. There is no way to accidentally leave the user in the alternate screen — even if a panic occurs, the `Drop` implementation runs.

Note that `exit()` is infallible during `Drop` — any I/O errors are silently ignored. If error handling is critical, call `ctx.exit()` explicitly before the context drops.

---

## ScreenUpdate

`ScreenUpdate` is a wrapper type that represents a new renderable to display inside a `Screen` or `ScreenContext`. It is used by `Screen::update()` and `ScreenContext::update()`.

```rust
pub struct ScreenUpdate {
    pub renderable: DynRenderable,
}
```

### Constructor

#### `ScreenUpdate::new(renderable)`

```rust
let update = ScreenUpdate::new("Updated content");
```

### From impl

Any type that implements `Renderable + Send + Sync + 'static` can be converted into a `ScreenUpdate` via `Into`/`From`:

```rust
let update: ScreenUpdate = "Hello".into();
let update: ScreenUpdate = my_table.into();
let update: ScreenUpdate = my_custom_renderable.into();
```

This is what allows the `update()` methods to accept plain strings, `Panel`, `Text`, `Layout`, etc. directly.

---

## Integration with Live

The `Live` display system has built-in support for the alternate screen via its `.screen()` builder method.

```rust
use rusty_rich::Live;

let mut live = Live::new(my_dashboard)
    .screen()                    // Enter alternate screen
    .refresh_per_second(4.0);
live.start()?;
```

When `.screen()` is set, `Live::start()` writes `\x1b[?1049h` to enter the alternate screen, and `Live::stop()` (or the `Drop` impl) writes `\x1b[?1049l` to exit it. The cursor is also hidden during the display (`\x1b[?25l`) and restored on stop (`\x1b[?25h`).

This integration means you do not need to manage a `ScreenContext` separately when using `Live` — the live display handles the alternate screen lifecycle for you. See the [Live documentation](live.md) for full details.

For finer control, you can combine `ScreenContext` with manual rendering loops instead of `Live`.

---

## Full-screen app example

The following example demonstrates a full-screen TUI application that uses `ScreenContext` directly (without `Live`). It shows a dashboard with a header, a metrics table, and a log panel, all arranged in a `Layout`. The application runs for 15 seconds, updating every 250 milliseconds, then exits cleanly.

```rust
use std::time::{Duration, Instant};
use rusty_rich::Console;
use rusty_rich::text::Text;
use rusty_rich::style::Style;
use rusty_rich::panel::Panel;
use rusty_rich::layout::{Layout, LayoutDivision, Size};
use rusty_rich::table::{Table, Column};
use rusty_rich::align::AlignMethod;
use rusty_rich::box_drawing;

fn main() -> std::io::Result<()> {
    let mut console = Console::new();

    // Enter the alternate screen buffer
    let mut ctx = console.screen();

    let start = Instant::now();
    let mut counter = 0u64;

    while start.elapsed() < Duration::from_secs(15) {
        counter += 1;
        ctx.update(render_dashboard(counter))?;
        std::thread::sleep(Duration::from_millis(250));
    }

    // ctx drops here → alternate screen is exited, original content restored
    Ok(())
}

fn render_dashboard(iteration: u64) -> Layout {
    let elapsed = iteration as f64 * 0.25;

    // ── Header panel ───────────────────────────────────────────────
    let mut header_text = Text::new("");
    header_text.append_styled(
        &format!("System Monitor — Iteration {}", iteration),
        Style::new().bold(true).foreground_color("bright_cyan"),
    );
    let header = Panel::new(header_text)
        .title(" Dashboard ")
        .title_align(AlignMethod::Center)
        .box_style(box_drawing::BOX_HEAVY_EDGE.clone());

    // ── Metrics table ──────────────────────────────────────────────
    let mut table = Table::new().show_lines();
    table.add_column(Column::new("Metric").justify(AlignMethod::Left));
    table.add_column(Column::new("Value").justify(AlignMethod::Right));
    table.add_column(Column::new("Status").justify(AlignMethod::Center));

    table.add_row_str(vec![
        "Uptime".into(),
        format!("{:.1}s", elapsed),
        status_badge(elapsed < 10.0),
    ]);
    table.add_row_str(vec![
        "Requests".into(),
        format!("{}", iteration * 42),
        status_badge(true),
    ]);
    table.add_row_str(vec![
        "Errors".into(),
        format!("{}", iteration % 7),
        status_badge(iteration % 7 < 3),
    ]);
    table.add_row_str(vec![
        "Memory".into(),
        format!("{:.1} MB", 128.0 + (iteration % 50) as f64 * 1.5),
        status_badge((iteration % 50) < 40),
    ]);

    let metrics = Panel::new(table)
        .title(" Metrics ")
        .box_style(box_drawing::BOX_SQUARE.clone());

    // ── Log panel ──────────────────────────────────────────────────
    let logs = [
        format!("[INFO]  Iteration {} started", iteration),
        format!("[DEBUG] Processing batch #{}", iteration % 10),
        format!("[INFO]  Memory OK"),
        if iteration % 3 == 0 {
            format!("[WARN]  Retry attempt #{}", iteration / 3)
        } else {
            format!("[INFO]  All systems nominal")
        },
    ];
    let mut log_text = Text::new("");
    for line in &logs {
        if line.starts_with("[WARN]") {
            log_text.append_styled(line, Style::new().foreground_color("yellow"));
        } else if line.starts_with("[ERROR]") {
            log_text.append_styled(line, Style::new().foreground_color("red"));
        } else {
            log_text.append_styled(line, Style::new().foreground_color("bright_black"));
        }
        log_text.append("\n", None);
    }
    let log_panel = Panel::new(log_text)
        .title(" Log ")
        .box_style(box_drawing::BOX_SQUARE.clone());

    // ── Layout: header on top, metrics + logs side by side ────────
    let mut layout = Layout::new();
    let top = layout.add(LayoutDivision::new(Size::Fixed(3)));
    let bottom = layout.add(LayoutDivision::new(Size::Ratio(1)));

    let bottom_split = bottom.split(
        LayoutDivision::new(Size::Ratio(1)),
        LayoutDivision::new(Size::Ratio(1)),
    );

    layout.set(&top, header).unwrap();
    layout.set(&bottom_split.0, metrics).unwrap();
    layout.set(&bottom_split.1, log_panel).unwrap();

    layout
}

fn status_badge(healthy: bool) -> String {
    if healthy {
        let style = Style::new()
            .bold(true)
            .foreground_color("bright_green")
            .to_ansi();
        format!("{style}OK{}", Style::new().reset_ansi())
    } else {
        let style = Style::new()
            .bold(true)
            .foreground_color("bright_red")
            .to_ansi();
        format!("{style}ALERT{}", Style::new().reset_ansi())
    }
}
```

This example demonstrates:

- **Alternate screen management** — `Console::screen()` creates a `ScreenContext` that enters the alternate screen immediately. The context lives for the duration of the `while` loop and auto-exits when dropped at the end of `main()`.
- **Content updates** — `ctx.update(render_dashboard(counter))` replaces the full-screen content on each iteration. The `update()` method wraps the layout in a `Screen` renderable, so it always fills the terminal exactly.
- **Scoped cleanup** — no explicit `exit()` call is needed. The `ScreenContext::drop` implementation restores the original terminal content automatically.

---

## Module structure

- `rusty_rich::Screen` — full-screen renderable (crops/pads child content)
- `rusty_rich::ScreenContext` — RAII guard for the alternate screen buffer
- `rusty_rich::ScreenUpdate` — wrapper for updating screen content

All items are re-exported from the crate root. The module file is:

- `/root/tuiproject/rust-rich/src/screen.rs` — Screen, ScreenContext, ScreenUpdate
