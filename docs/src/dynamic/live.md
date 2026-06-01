# Live Display

`Live` manages an auto-updating region of the terminal. It is the primary mechanism for
building real-time dynamic displays — progress indicators, live-updating dashboards,
streaming log viewers, animated clocks, and any other output that changes over time.

Under the hood, `Live` re-renders a `Renderable` on a refresh loop, using ANSI escape
sequences to overwrite the previously displayed lines rather than appending new output.
This produces the illusion of a "live" region of the terminal that updates in place.

```rust
use rusty_rich::Live;
```

---

## Constructor

### `Live::new(renderable)`

Creates a new live display that repeatedly renders the given `Renderable` value.

| Parameter | Type | Description |
|-----------|------|-------------|
| `renderable` | `impl Renderable + Send + Sync + 'static` | The content to display and re-render on each refresh cycle. |

`renderable` can be any type that implements `Renderable` — strings, `Text`, `Table`,
`Panel`, `Layout`, a custom renderable, etc.

```rust
use rusty_rich::Live;

let mut live = Live::new("Initial content");
```

---

## Builder configuration

All builder methods are called on the `Live` value returned by `new()`. They must be
chained before calling `start()`.

### `screen()`

Enables alternate-screen ("alt-screen") buffer mode. When active, the live display
switches to a separate terminal buffer that completely replaces the visible screen
while the display is running. The original screen content is restored when `stop()` is
called or the `Live` instance is dropped.

Use this for full-screen applications like dashboards, TUI editors, or interactive
monitoring tools.

```rust
let mut live = Live::new(my_dashboard).screen();
```

Internally, `screen()` causes `start()` to emit the `\x1b[?1049h` escape sequence to
enter the alternate screen, and `stop()` emits `\x1b[?1049l` to return to the main
screen. The cursor is also hidden via `\x1b[?25l` and restored via `\x1b[?25h`.

### `no_auto_refresh()`

Disables the automatic refresh loop. When this is set, the display is only updated when
you explicitly call `refresh()` or `update()`. Useful when you want precise control over
when re-rendering occurs.

```rust
let mut live = Live::new("Manual refresh only")
    .no_auto_refresh();
```

### `refresh_per_second(rate: f64)`

Sets the target refresh rate in frames per second. The default is `4.0`. Higher values
produce smoother updates but consume more CPU.

```rust
let mut live = Live::new("60 FPS live display")
    .refresh_per_second(60.0);
```

The actual frame rate depends on how fast the renderable renders and how quickly the
terminal processes the output. Values above 30–60 are typically not useful outside of
animations or very short render cycles.

### `transient()`

Makes the live display output disappear when `stop()` is called. By default the last
rendered frame remains on screen after stopping. With `transient()`, the live display
erases its lines from the terminal, leaving no trace.

```rust
let mut live = Live::new("Processing...")
    .transient();
// After live.stop(), the terminal shows nothing from this display
```

This is useful for "disposable" progress indicators that should not clutter the terminal
after they complete.

---

## Lifecycle methods

### `start(&mut self) -> io::Result<()>`

Starts the live display. This:
1. Records the start time.
2. If `screen` mode is enabled, enters the alternate screen buffer.
3. Hides the cursor.
4. Renders and outputs the initial frame.

```rust
let mut live = Live::new("Starting...");
live.start()?;
```

### `stop(&mut self) -> io::Result<()>`

Stops the live display. This:
1. If `transient` mode is enabled, erases all displayed lines.
2. If `screen` mode is enabled, exits the alternate screen buffer.
3. Restores the cursor.
4. Clears the start time.

```rust
live.stop()?;
```

`Live` also implements `Drop`, so `stop()` is called automatically when the `Live`
instance goes out of scope. However, because `stop()` can return an I/O error, you may
want to call it explicitly when error handling is important.

---

## Update methods

### `update(&mut self, renderable) -> io::Result<()>`

Replaces the current renderable and immediately re-renders. The new renderable can be a
different type from the original.

```rust
live.update("New content")?;
```

`update` accepts the same range of types as `new()` — any `impl Renderable + Send + Sync
+ 'static`.

### `refresh(&mut self) -> io::Result<()>`

Re-renders the current renderable without changing the content. This is useful when the
renderable's internal state has changed (e.g., a counter or progress value) and you want
to push the new frame to the terminal.

```rust
live.refresh()?;
```

The refresh logic:
1. Computes the number of previously displayed lines.
2. Moves the cursor up by that many lines using `\x1b[N F`.
3. Renders the current renderable to ANSI text.
4. Outputs the new text.
5. If the new output has fewer lines than the previous frame, clears the excess lines.
6. Updates the tracked line count.

This line-counting approach means that the live region can expand and contract
dynamically between frames.

---

## Auto-refresh loop

When `auto_refresh` is enabled (the default), the display continuously re-renders at the
rate specified by `refresh_per_second`. The auto-refresh is driven by the caller's own
loop — `Live` does not spawn a background thread. A typical pattern is:

```rust
let mut live = Live::new(renderable).refresh_per_second(10.0);
live.start()?;

// In your own event loop, periodically call refresh()
loop {
    // ... update application state ...
    live.refresh()?;
    std::thread::sleep(Duration::from_millis(100));
}
```

With `no_auto_refresh()`, you call `refresh()` only when something has actually changed,
which is more efficient for sporadic updates.

---

## redirect_stdout / redirect_stderr

**Not yet implemented.** In the Python Rich library, `Live` can redirect `print()` output
so that standard writes appear within the live display region rather than breaking the
layout. This Rust port does not currently support stdout/stderr redirection during a
live display. If you need to print diagnostic output while a live display is running,
consider routing messages through the renderable itself (e.g., by accumulating log lines
in a `Text` or `Table` and calling `update()`).

---

## Examples

### Live-updating clock

A real-time clock that updates every 100 milliseconds:

```rust
use std::time::{Duration, Instant};
use rusty_rich::Live;
use rusty_rich::text::Text;
use rusty_rich::style::Style;
use rusty_rich::align::AlignMethod;
use rusty_rich::panel::Panel;
use rusty_rich::box_drawing;

fn main() -> std::io::Result<()> {
    let mut live = Live::new(render_clock())
        .refresh_per_second(10.0);
    live.start()?;

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        live.update(render_clock())?;
        std::thread::sleep(Duration::from_millis(100));
    }

    live.stop()?;
    Ok(())
}

fn render_clock() -> Panel<Text> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = now.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;

    let mut text = Text::new("");
    let time_str = format!("{:02}:{:02}:{:02}", h, m, s);
    text.append_styled(
        &time_str,
        Style::new().bold(true).foreground_color("bright_green"),
    );

    Panel::new(text)
        .title(" Live Clock ")
        .title_align(AlignMethod::Center)
        .box_style(box_drawing::BOX_DOUBLE.clone())
}
```

### Progress bar in a live display

A simulated download with a live-updating progress bar:

```rust
use std::time::Duration;
use rusty_rich::Live;
use rusty_rich::text::Text;
use rusty_rich::style::Style;
use rusty_rich::progress::ProgressBar;

fn main() -> std::io::Result<()> {
    let mut live = Live::new(render_progress("Downloading...", 0.0))
        .transient();
    live.start()?;

    let steps = 50;
    for i in 0..=steps {
        let fraction = i as f64 / steps as f64;
        live.update(render_progress("Downloading...", fraction))?;
        std::thread::sleep(Duration::from_millis(50));
    }

    // Final success message
    live.update(render_success("Download complete!"))?;
    std::thread::sleep(Duration::from_secs(1));

    live.stop()?;
    Ok(())
}

fn render_progress(label: &str, fraction: f64) -> Text {
    let bar = ProgressBar::new()
        .total(1.0)
        .completed(fraction)
        .complete_style(Style::new().foreground_color("green"))
        .remaining_style(Style::new().foreground_color("bright_black"));

    let bar_rendered = bar.render(40);

    let mut text = Text::new("");
    text.append_styled(label, Style::new().bold(true));
    text.append("\n", None);
    text.append(&bar_rendered, None);
    text.append(
        &format!("  {:.1}%", fraction * 100.0),
        Some(Style::new().foreground_color("cyan")),
    );
    text
}

fn render_success(message: &str) -> Text {
    let mut text = Text::new("");
    text.append_styled(message, Style::new().bold(true).foreground_color("bright_green"));
    text
}
```

### Full-screen live app

A full-screen monitoring dashboard that uses alternate-screen mode and a `Layout` to
display multiple panels side-by-side:

```rust
use std::time::{Duration, Instant};
use rusty_rich::Live;
use rusty_rich::text::Text;
use rusty_rich::style::Style;
use rusty_rich::panel::Panel;
use rusty_rich::layout::{Layout, LayoutDivision, Size};
use rusty_rich::table::{Table, Column};
use rusty_rich::align::AlignMethod;
use rusty_rich::box_drawing;

fn main() -> std::io::Result<()> {
    let mut live = Live::new(render_dashboard(0))
        .screen()                     // Use alternate screen
        .refresh_per_second(4.0);     // Update 4 times per second
    live.start()?;

    let start = Instant::now();
    let mut counter = 0u64;
    while start.elapsed() < Duration::from_secs(15) {
        counter += 1;
        live.update(render_dashboard(counter))?;
        std::thread::sleep(Duration::from_millis(250));
    }

    live.stop()?;  // Returns to normal terminal
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

This example creates a full-screen live dashboard with three panels arranged in a layout:
a header, a metrics table, and a log view. The alternate screen (`screen()`) ensures the
dashboard takes over the whole terminal and restores the previous content on exit.

---

## Drop behavior

`Live` implements `Drop`, so the display is automatically cleaned up when the instance
goes out of scope — the cursor is restored, the alternate screen is exited if active,
and transient content is erased. However, errors during drop are silently ignored. Call
`stop()` explicitly when you need to handle I/O errors.
