# Status

`Status` displays a message alongside an animated spinner on a single terminal line. It is equivalent to Python Rich's `status.py` and provides a lightweight way to show that an operation is in progress.

Under the hood, `Status` writes to `stdout` using a carriage return (`\r`) and ANSI escape sequences to overwrite the current line, so the output stays on a single line that updates in place.

```rust
use rusty_rich::Status;
```

---

## Constructor

### `Status::new(message)`

Creates a new `Status` with the given message and the default spinner (`SPINNER_DOTS`).

| Parameter | Type | Description |
|-----------|------|-------------|
| `message` | `impl Into<String>` | The status text displayed alongside the spinner |

```rust
let status = Status::new("Working...");
```

The spinner is initially stopped (`started` is `None`). No output is written to the terminal until `start()` is called.

---

## Builder configuration

### `spinner(spinner: Spinner)`

Sets the spinner animation style. This must be called before `start()`.

```rust
use rusty_rich::Status;
use rusty_rich::spinner::{Spinner, SPINNER_LINE};

let mut status = Status::new("Loading...")
    .spinner(Spinner::new(&SPINNER_LINE));
```

See [Spinner selection](#spinner-selection) below for available spinner styles.

---

## Lifecycle methods

### `start(&mut self) -> io::Result<()>`

Starts the status display. This:
1. Records the start time (`Instant::now()`).
2. Renders the initial frame to stdout (spinner + message, preceded by a carriage return).

```rust
let mut status = Status::new("Processing...");
status.start()?;
```

After `start()`, the spinner begins animating. Call `refresh()` periodically (or use a loop with `refresh()`) to advance the animation.

### `stop(&mut self) -> io::Result<()>`

Stops the status display and clears the line from the terminal.

- Moves the cursor to the beginning of the line (`\r`).
- Clears the line with `\x1b[K` (erase to end of line).
- Sets `started` to `None`.

```rust
status.stop()?;
```

After calling `stop()`, the terminal line is empty and subsequent output starts fresh on the next line. Call `stop()` before using `println!` or other output to avoid artifacts.

### `refresh(&mut self) -> io::Result<()>`

Re-renders the current status line without changing the message. Use this to advance the spinner animation in a loop.

```rust
loop {
    status.refresh()?;
    std::thread::sleep(std::time::Duration::from_millis(80));
    if work_done { break; }
}
```

Internally, `refresh()` calls `write_status()` which computes the elapsed time, picks the current spinner frame, and overwrites the line with `\r<spinner> <message>`.

---

## Update methods

### `update(&mut self, message: impl Into<String>) -> io::Result<()>`

Changes the status message and immediately re-renders the display. The spinner continues from the same elapsed time, so there is no visual reset.

```rust
status.update("Still working...")?;
status.update("Almost done...")?;
status.update("Finishing up...")?;
```

This is useful for communicating progress stages without stopping and restarting the display.

---

## Spinner selection

The `Status` struct wraps a `Spinner` value. You can choose from 38 predefined spinner animations, all matching Python Rich's spinner set.

| Name | Frames | Interval | Preview (concept) |
|------|--------|----------|--------------------|
| `dots` (default) | 10 | 80 ms | Braille quarter-circle rotation |
| `line` | 4 | 100 ms | `- \| /` |
| `dots2` | 8 | 80 ms | Braille full-block rotation |
| `dots3`–`dots11` | 6–29 | 80 ms | Various braille patterns |
| `simpleDots` | 6 | 200 ms | `.  ` `.. ` `...` ` ..` `  .` `   ` |
| `arc` | 6 | 100 ms | `◜ ◠ ◝ ◞ ◡ ◟` |
| `arrow` | 8 | 100 ms | `← ↖ ↑ ↗ → ↘ ↓ ↙` |
| `arrow2` | 8 | 100 ms | Emoji arrows |
| `arrow3` | 6 | 100 ms | `▹ ▸` |
| `bouncingBar` | 8 | 150 ms | `[=   ]` `[==  ]` `[=== ]` ... |
| `bouncingBall` | 8 | 150 ms | `( ●    )` `(  ●   )` ... |
| `circle` | 4 | 100 ms | `◐ ◓ ◑ ◒` |
| `clock` | 12 | 100 ms | Clock face emojis |
| `moon` | 8 | 80 ms | Moon phase emojis |
| `earth` | 3 | 200 ms | Globe emojis |
| `hearts` | 12 | 120 ms | Heart color emojis |
| `smiley` | 2 | 200 ms | `😄 😝` |
| `toggle` | 2 | 200 ms | `⊶ ⊷` |
| `triangle` | 4 | 100 ms | `◢ ◣ ◤ ◥` |
| `verticalBars` | 15 | 80 ms | Vertical bar grow/shrink |
| `growHorizontal` | 14 | 80 ms | Horizontal bar grow/shrink |
| `growVertical` | 14 | 80 ms | Vertical bar segments |
| `noise` | 9 | 80 ms | `▓ ▒ ░` dither pattern |
| `pong` | 30 | 80 ms | Pong ball animation |
| `runner` | 6 | 150 ms | Walking/running emoji |
| `shark` | 6 | 150 ms | Shark emoji |
| `grenade` | 3 | 100 ms | Fuse animation |
| `monkey` | 11 | 150 ms | See-no/hear-no/speak-no emojis |
| `christmas` | 2 | 400 ms | Christmas tree emojis |
| `hamburger` | 3 | 120 ms | Trigram symbols |

Use `spinner::Spinner::new(&frames)` to create a spinner from a `SpinnerFrames` constant:

```rust
use rusty_rich::spinner::{Spinner, SPINNER_MOON, SPINNER_EARTH, SPINNER_BOUNCING_BALL};

let s1 = Status::new("Thinking...").spinner(Spinner::new(&SPINNER_MOON));
let s2 = Status::new("Searching...").spinner(Spinner::new(&SPINNER_EARTH));
```

### Runtime lookup by name

Use `spinner::get_spinner(name)` to look up a `SpinnerFrames` by name at runtime. Names are case-insensitive and also handle CamelCase, kebab-case, and space-separated forms.

```rust
use rusty_rich::spinner::get_spinner;

let frames = get_spinner("bouncingBar").unwrap();
let status = Status::new("Downloading...").spinner(Spinner::new(frames));
```

Returns `None` if the name does not match any known spinner.

---

## Speed

The spinner's animation speed is controlled by the `interval` field of the `SpinnerFrames` (in seconds per frame). The current frame is selected by:

```
frame_index = floor(elapsed_seconds / interval) % frame_count
```

Each spinner defines its own interval:

| Interval | Spinners |
|----------|----------|
| 80 ms (fast) | `dots`, `dots2`–`dots11`, `moon`, `growHorizontal`, `growVertical`, `noise`, `pong`, `verticalBars` |
| 100 ms (moderate) | `line`, `arc`, `arrow`, `arrow2`, `arrow3`, `circle`, `clock`, `grenade`, `triangle` |
| 120 ms | `hearts`, `hamburger` |
| 150 ms | `bouncingBar`, `bouncingBall`, `monkey`, `runner`, `shark` |
| 200 ms (slow) | `simpleDots`, `earth`, `smiley`, `toggle` |
| 400 ms (very slow) | `christmas` |

To render the spinner at its natural speed, call `refresh()` at least as often as the interval of the chosen spinner. For the fast `dots` spinners (80 ms), call `refresh()` every 50--80 ms. For slower spinners like `simpleDots` (200 ms), every 100--200 ms is sufficient.

The `Status` struct does **not** spawn a background thread or automatic refresh loop. You must drive the animation yourself by calling `refresh()` in a loop -- this keeps the library lightweight and gives you full control over when rendering occurs.

---

## Example: status during a long operation

```rust
use std::time::Duration;
use rusty_rich::Status;
use rusty_rich::spinner::{Spinner, SPINNER_LINE};

fn main() -> std::io::Result<()> {
    let mut status = Status::new("Initializing...")
        .spinner(Spinner::new(&SPINNER_LINE));
    status.start()?;

    // Phase 1
    std::thread::sleep(Duration::from_millis(800));
    status.update("Loading configuration...")?;
    for _ in 0..20 {
        status.refresh()?;
        std::thread::sleep(Duration::from_millis(50));
    }

    // Phase 2
    status.update("Processing data...")?;
    for _ in 0..30 {
        status.refresh()?;
        std::thread::sleep(Duration::from_millis(50));
    }

    // Phase 3
    status.update("Writing output...")?;
    for _ in 0..15 {
        status.refresh()?;
        std::thread::sleep(Duration::from_millis(50));
    }

    status.stop()?;
    println!("Done!");
    Ok(())
}
```

This produces a single line in the terminal that cycles through three stages, each with an animated line spinner:

```
\ Loading configuration...
\ Processing data...
\ Writing output...
```

After `stop()` the line is cleared and `Done!` appears on a fresh line.

### With a custom spinner

```rust
use std::time::Duration;
use rusty_rich::Status;
use rusty_rich::spinner::{Spinner, SPINNER_BOUNCING_BALL};

fn main() -> std::io::Result<()> {
    let mut status = Status::new("Downloading package...")
        .spinner(Spinner::new(&SPINNER_BOUNCING_BALL));
    status.start()?;

    let steps = 50;
    for i in 0..=steps {
        status.update(format!(
            "Downloading package... [{}/{}]",
            i, steps
        ))?;
        // refresh the spinner a few times per step to keep it smooth
        for _ in 0..4 {
            status.refresh()?;
            std::thread::sleep(Duration::from_millis(38)); // ~150ms total per step
        }
    }

    status.stop()?;
    println!("Download complete!");
    Ok(())
}
```

### Status with progress bar fallback

For longer operations, combine `Status` with a `ProgressBar` rendered to a string:

```rust
use std::time::Duration;
use rusty_rich::Status;
use rusty_rich::ProgressBar;
use rusty_rich::style::Style;
use rusty_rich::spinner::{Spinner, SPINNER_DOTS};

fn main() -> std::io::Result<()> {
    let mut status = Status::new("")
        .spinner(Spinner::new(&SPINNER_DOTS));
    status.start()?;

    let bar = ProgressBar::new()
        .total(100.0)
        .complete_style(Style::new().foreground_color("green"))
        .remaining_style(Style::new().foreground_color("bright_black"));

    for i in 0..=100 {
        let bar_str = bar.completed(i as f64).render(30);
        status.update(format!("Processing... {} {}/100", bar_str, i))?;
        std::thread::sleep(Duration::from_millis(30));
    }

    status.stop()?;
    println!("All done!");
    Ok(())
}
```

---

## Comparison with Live

| Aspect | `Status` | `Live` |
|--------|----------|--------|
| Output area | Single line | Multiple lines |
| Auto-refresh | No (manual) | Optional (background, configurable rate) |
| Renderable types | String message | Any `Renderable` |
| Alt-screen | No | Optional |
| Transient | Always (clears on stop) | Optional |
| Best for | Brief, simple operations | Complex, multi-line dashboards |

`Status` is the right choice when you need a lightweight, single-line progress indicator. For multi-line or full-screen dynamic displays, use `Live` instead.

---

## Module structure

- `rusty_rich::Status` -- status message with spinner
- `rusty_rich::spinner::Spinner` -- configurable spinner renderer
- `rusty_rich::spinner::SpinnerFrames` -- frame data for a spinner animation
- `rusty_rich::spinner::get_spinner(name)` -- runtime spinner lookup
- `rusty_rich::spinner::SPINNER_*` -- 38 predefined spinner constants

All items are re-exported from the crate root where applicable. The source files are:

- `/root/tuiproject/rust-rich/src/status.rs` -- Status struct
- `/root/tuiproject/rust-rich/src/spinner.rs` -- Spinner, SpinnerFrames, predefined spinners, and lookup
