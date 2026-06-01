# Spinners

Animated spinners for showing progress, loading states, or ongoing activity -- a Rust port of Python Rich's `spinner.py`. Spinners are lightweight: each one is a list of string frames plus a frame interval, with no threading or rendering engine of its own. They integrate naturally with `Live`, `Progress`, and `Status`.

```rust
use rusty_rich::Spinner;
```

---

## SpinnerFrames

`SpinnerFrames` is the raw data for a spinner animation: the sequence of frame strings and the time between frames.

```rust
pub struct SpinnerFrames {
    pub frames: &'static [&'static str],
    pub interval: f64, // seconds per frame
}
```

| Field      | Type                      | Description                            |
|------------|---------------------------|----------------------------------------|
| `frames`   | `&'static [&'static str]` | The sequence of frame strings.         |
| `interval` | `f64`                     | Seconds per frame (duration between frame transitions). |

The `frames` array is indexed by time; the `interval` controls the animation speed. A smaller interval means faster animation.

All predefined spinners are `const` values of type `SpinnerFrames`.

---

## Predefined spinners

There are **38** built-in spinners. They are registered in the `SPINNERS` lookup list and can be retrieved by name via `get_spinner()`.

### Classic / line spinner

| Constant            | Name    | Frames                       | Interval | Preview                                                       |
|---------------------|---------|------------------------------|----------|---------------------------------------------------------------|
| `SPINNER_LINE`      | `line`  | `- \ | /` (4)                | 0.10 s   | A rotating line: `-` `\` `|` `/`. Simple and widely supported. |

### Braille dots (dots1 through dots11)

| Constant             | Name      | Frames count | Interval | Preview                                                              |
|----------------------|-----------|-------------:|----------|----------------------------------------------------------------------|
| `SPINNER_DOTS`       | `dots`    | 10           | 0.08 s   | Braille dots rotating clockwise: `⠋` `⠙` `⠹` `⠸` `⠼` `⠴` `⠦` `⠧` `⠇` `⠏`. This is also the **default** spinner. |
| `SPINNER_DOTS2`      | `dots2`   | 8            | 0.08 s   | Braille block rotation: `⣾` `⣽` `⣻` `⢿` `⡿` `⣟` `⣯` `⣷`. |
| `SPINNER_DOTS3`      | `dots3`   | 10           | 0.08 s   | Braille wave: `⠋` `⠙` `⠚` `⠞` `⠖` `⠦` `⠴` `⠲` `⠳` `⠓`. |
| `SPINNER_DOTS4`      | `dots4`   | 14           | 0.08 s   | Braille pulse out-and-back: `⠄` `⠆` `⠇` `⠋` `⠙` `⠸` `⠰` `⠠` `⠰` `⠸` `⠙` `⠋` `⠇` `⠆`. |
| `SPINNER_DOTS5`      | `dots5`   | 17           | 0.08 s   | Braille dot with bounce: `⠋` `⠙` `⠚` `⠒` `⠂` `⠂` `⠒` `⠲` `⠴` `⠦` `⠖` `⠒` `⠐` `⠐` `⠒` `⠓` `⠋`. |
| `SPINNER_DOTS6`      | `dots6`   | 24           | 0.08 s   | Braille full rotation: 24-frame clockwise fill cycle. |
| `SPINNER_DOTS7`      | `dots7`   | 24           | 0.08 s   | Braille offset rotation: 24-frame counter-clockwise fill cycle. |
| `SPINNER_DOTS8`      | `dots8`   | 29           | 0.08 s   | Braille bounce: 29-frame long bounce cycle. |
| `SPINNER_DOTS9`      | `dots9`   | 8            | 0.08 s   | Braille corner sweep: `⢹` `⢺` `⢼` `⣸` `⣇` `⡧` `⡗` `⡏`. |
| `SPINNER_DOTS10`     | `dots10`  | 7            | 0.08 s   | Braille growing bar: `⢄` `⢂` `⢁` `⡁` `⡈` `⡐` `⡠`. |
| `SPINNER_DOTS11`     | `dots11`  | 8            | 0.10 s   | Braille single dot: `⠁` `⠂` `⠄` `⡀` `⢀` `⠠` `⠐` `⠈`. |

### Arrow spinners

| Constant            | Name     | Frames count | Interval | Preview                                                        |
|---------------------|----------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_ARROW`     | `arrow`  | 8            | 0.10 s   | Rotating arrows: `←` `↖` `↑` `↗` `→` `↘` `↓` `↙`.            |
| `SPINNER_ARROW2`    | `arrow2` | 8            | 0.10 s   | Emoji arrows: `⬆️` `↗️` `➡️` `↘️` `⬇️` `↙️` `⬅️` `↖️`.        |
| `SPINNER_ARROW3`    | `arrow3` | 6            | 0.10 s   | Thin triangle alternation: `▹` `▸` `▹` `▸` `▹` `▸`.          |

### Geometric spinners

| Constant              | Name           | Frames count | Interval | Preview                                                        |
|-----------------------|----------------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_ARC`         | `arc`          | 6            | 0.10 s   | Rotating arc segments: `◜` `◠` `◝` `◞` `◡` `◟`.              |
| `SPINNER_CIRCLE`      | `circle`       | 4            | 0.10 s   | Quarter-circle rotation: `◐` `◓` `◑` `◒`.                     |
| `SPINNER_TRIANGLE`    | `triangle`     | 4            | 0.10 s   | Rotating triangles: `◢` `◣` `◤` `◥`.                          |
| `SPINNER_TOGGLE`      | `toggle`       | 2            | 0.20 s   | Toggle switch: `⊶` `⊷`.                                        |
| `SPINNER_HAMBURGER`   | `hamburger`    | 3            | 0.12 s   | Trigram rotation: `☱` `☲` `☴`.                                |

### Growth and bar spinners

| Constant                    | Name              | Frames count | Interval | Preview                                                        |
|-----------------------------|-------------------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_GROW_HORIZONTAL`   | `growHorizontal`  | 14           | 0.08 s   | A horizontal bar that grows and shrinks: `▏` `▎` `▍` `▌` `▋` `▊` `▉` then back. |
| `SPINNER_GROW_VERTICAL`     | `growVertical`    | 14           | 0.08 s   | A vertical bar that grows and shrinks: `▁` `▂` `▃` `▄` `▅` `▆` `▇` `█` then back. |
| `SPINNER_VERTICAL_BARS`     | `verticalBars`    | 15           | 0.08 s   | A single column rising and falling: `▁` through `█` and back.  |
| `SPINNER_BOUNCING_BAR`      | `bouncingBar`     | 8            | 0.15 s   | A bar bouncing inside brackets: `[    ]` `[=   ]` `[==  ]` `[=== ]` `[ ===]` `[  ==]` `[   =]` `[    ]`. |
| `SPINNER_BOUNCING_BALL`     | `bouncingBall`    | 8            | 0.15 s   | A ball bouncing in parentheses: `( ●    )` `(  ●   )` `(   ●  )` etc. |
| `SPINNER_NOISE`             | `noise`           | 9            | 0.08 s   | Dithering noise: `▓` `▒` `░` `▓` `▒` `░` `▓` `▒` `░`.        |

### Pong spinner

| Constant            | Name    | Frames count | Interval | Preview                                                        |
|---------------------|---------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_PONG`      | `pong`  | 30           | 0.08 s   | A ball bouncing left and right between two walls: `▐⠂       ▌` through `▐       ⠂▌` and back. The largest built-in spinner at 30 frames. |

### Nature and celestial spinners

| Constant              | Name       | Frames count | Interval | Preview                                                        |
|-----------------------|------------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_MOON`        | `moon`     | 8            | 0.08 s   | Lunar phases: `🌑` `🌒` `🌓` `🌔` `🌕` `🌖` `🌗` `🌘`.       |
| `SPINNER_EARTH`       | `earth`    | 3            | 0.20 s   | Rotating Earth globes: `🌍` `🌎` `🌏`.                         |

### Emoji and icon spinners

| Constant               | Name          | Frames count | Interval | Preview                                                        |
|------------------------|---------------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_CLOCK`        | `clock`       | 12           | 0.10 s   | Clock faces showing each hour: `🕐` through `🕛`.              |
| `SPINNER_HEARTS`       | `hearts`      | 12           | 0.12 s   | Cycling heart colors: `🩷` `❤️` `🧡` `💛` `💚` `💙` `🩵` `💜` `🤎` `🖤` `🩶` `🤍`. |
| `SPINNER_SMILEY`       | `smiley`      | 2            | 0.20 s   | Grinning and squinting: `😄` `😝`.                              |
| `SPINNER_MONKEY`       | `monkey`      | 11           | 0.15 s   | Monkey see-no-evil sequence: `🐒` with `🙈` `🙉` `🙊` interludes. |
| `SPINNER_RUNNER`       | `runner`      | 6            | 0.15 s   | Walking and running: `🚶` `🏃` `🏃` `🏃` `🚶` `🚶`.          |
| `SPINNER_SHARK`        | `shark`       | 6            | 0.15 s   | Shark with splash effects: `🦈` `🌀` `🦈` `🌀` `🦈` `🌀`.    |
| `SPINNER_CHRISTMAS`    | `christmas`   | 2            | 0.40 s   | Pine tree and decorated tree: `🌲` `🎄`.                        |
| `SPINNER_GRENADE`      | `grenade`     | 3            | 0.10 s   | Fuse sparking: `،  💣  ` `۔  💣  ` `﹒ 💣  `.                 |

### Utility spinners

| Constant                  | Name          | Frames count | Interval | Preview                                                        |
|---------------------------|---------------|-------------:|----------|----------------------------------------------------------------|
| `SPINNER_SIMPLE_DOTS`     | `simpleDots`  | 6            | 0.20 s   | ASCII dots marching: `.  ` `.. ` `...` ` ..` `  .` `   `. Safe for any terminal. |

### Default spinner

```rust
pub const DEFAULT_SPINNER: SpinnerFrames = SPINNER_DOTS;
```

`DEFAULT_SPINNER` is the braille `dots` spinner (10 frames, 0.08 s interval). It is used when constructing a `Spinner` with `Spinner::default()` or the `SpinnerColumn` default.

---

## SPINNERS list

All predefined spinners are registered in a constant name-to-frame mapping:

```rust
pub const SPINNERS: &[(&str, &SpinnerFrames)] = &[
    ("arc", &SPINNER_ARC),
    ("arrow", &SPINNER_ARROW),
    ("arrow2", &SPINNER_ARROW2),
    // ... all 38 spinners ...
];
```

This list is used by `get_spinner()` for runtime name-based lookup. You can iterate over it to enumerate all available spinners:

```rust
use rusty_rich::SPINNERS;

for (name, frames) in SPINNERS {
    println!("{} ({} frames, {:.0} ms/frame)",
        name, frames.frames.len(), frames.interval * 1000.0);
}
```

---

## get_spinner()

Look up a spinner by name (case-insensitive, whitespace-tolerant).

```rust
pub fn get_spinner(name: &str) -> Option<&'static SpinnerFrames>
```

Returns `None` if no spinner with the given name exists.

**Name matching rules:**

- Direct case-insensitive match is tried first.
- If that fails, spaces and hyphens are stripped from both the input and each registered name before comparing (lowercased).
- Examples of equivalent lookups: `"bouncingBar"`, `"BOUNCINGBAR"`, `"bouncing_bar"`, `"bouncing bar"` all resolve to the same spinner.

```rust
use rusty_rich::get_spinner;

// Direct name
let arc = get_spinner("arc").unwrap();
assert_eq!(arc.frames.len(), 6);

// Case-insensitive
let same = get_spinner("ARC").unwrap();
assert_eq!(arc.frames, same.frames);

// Normalized lookup
let bb = get_spinner("bouncing_bar").unwrap();
assert_eq!(bb.frames.len(), 8);
```

---

## Spinner

`Spinner` is an animated spinner renderable with optional text and styling. It combines a `SpinnerFrames` definition with runtime state (text, style) and provides `frame_at()` and `render()` methods keyed on elapsed time.

```rust
use rusty_rich::Spinner;
```

### Fields

| Field      | Type                        | Description                             |
|------------|-----------------------------|-----------------------------------------|
| `frames`   | `&'static [&'static str]`   | The animation frames.                   |
| `interval` | `f64`                       | Seconds per frame.                      |
| `text`     | `String`                    | Text displayed alongside the spinner.   |
| `style`    | `Style`                     | ANSI style applied to the spinner frame. |

### new

Create a `Spinner` from any `&'static SpinnerFrames`.

```rust
pub fn new(spinner: &'static SpinnerFrames) -> Self
```

```rust
use rusty_rich::{Spinner, SPINNER_ARC, SPINNER_MOON};

let s = Spinner::new(&SPINNER_ARC);
let moon = Spinner::new(&SPINNER_MOON);
```
If you call `Spinner::default()`, it uses the `DEFAULT_SPINNER` (braille dots).

### builder: text

Set the text that appears after the spinner frame.

```rust
pub fn text(mut self, text: impl Into<String>) -> Self
```

```rust
let s = Spinner::new(&SPINNER_DOTS).text("Loading...");
```

### builder: style

Set the ANSI style for the spinner frame.

```rust
pub fn style(mut self, style: crate::style::Style) -> Self
```

```rust
use rusty_rich::{Spinner, SPINNER_DOTS, Style};

let s = Spinner::new(&SPINNER_DOTS)
    .text("Working...")
    .style(Style::new().foreground_color("cyan"));
```

### frame_at()

Get the frame string for a given elapsed duration. Selects the frame by cycling through `frames` based on `elapsed / interval`.

```rust
pub fn frame_at(&self, elapsed: Duration) -> &'static str
```

```rust
use std::time::Duration;
use rusty_rich::{Spinner, SPINNER_LINE};

let s = Spinner::new(&SPINNER_LINE);

// At 0 ms      -> "-"
// At 100 ms    -> "\"
// At 200 ms    -> "|"
// At 300 ms    -> "/"
// At 400 ms    -> "-" (wraps around)
println!("{}", s.frame_at(Duration::from_millis(200)));
```

The frame index is computed as:

```rust
let idx = (elapsed.as_secs_f64() / self.interval) as usize % self.frames.len();
```

### render()

Render the spinner and optional text as a single formatted string.

```rust
pub fn render(&self, elapsed: Duration) -> String
```

```rust
use std::time::Duration;
use rusty_rich::{Spinner, SPINNER_DOTS};

let s = Spinner::new(&SPINNER_DOTS).text("Installing...");

// Outputs something like: "⠙ Installing..."
println!("{}", s.render(Duration::from_millis(160)));
```

- If the spinner has a `style` set, the frame is wrapped in ANSI style codes and followed by a reset.
- If `text` is non-empty, it is appended after a space: `"{frame} {text}"`.

---

## Custom spinners

You can define your own spinner by constructing a `SpinnerFrames` value with any sequence of strings and an interval.

```rust
use rusty_rich::{Spinner, SpinnerFrames, Style};

// A custom 3-frame pulsing dot
const PULSE: SpinnerFrames = SpinnerFrames {
    frames: &[".", "o", "O"],
    interval: 0.15,
};

let spinner = Spinner::new(&PULSE)
    .text("Pulsing...")
    .style(Style::new().foreground_color("bright_magenta"));
```

Considerations for custom spinners:

- All frames should be the same display width so the animation does not jump.
- The `interval` controls animation speed: 0.08--0.15 s is typical for smooth animation, 0.15--0.4 s for more deliberate pacing.
- Frames can contain any Unicode text, including emoji, ANSI art, or braille patterns.
- `SpinnerFrames` stores a `&'static [&'static str]`, so static data (constants) work best. For runtime-generated frames you would need a different container (not currently supported).

To make a custom spinner available via `get_spinner()`, add it to a copy of the `SPINNERS` list:

```rust
use rusty_rich::SpinnerFrames;

const MY_CUSTOM: SpinnerFrames = SpinnerFrames {
    frames: &["▖", "▘", "▝", "▗"],
    interval: 0.10,
};

// Lookup is not automatically registered -- callers must reference the constant directly.
```

---

## Usage with Live display

`Spinner` pairs naturally with `Live` for animated output:

```rust
use std::time::{Duration, Instant};
use rusty_rich::{Live, Spinner, SPINNER_DOTS, Style};

fn main() -> std::io::Result<()> {
    let spinner = Spinner::new(&SPINNER_DOTS)
        .text("Processing...")
        .style(Style::new().foreground_color("cyan"));

    let mut live = Live::new(spinner.render(Duration::ZERO))
        .refresh_per_second(12.0)
        .transient();

    live.start()?;

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        let s = spinner.render(start.elapsed());
        live.update(s)?;
        std::thread::sleep(Duration::from_millis(10));
    }

    live.stop()?;
    Ok(())
}
```

---

## Usage with Status

`Status` wraps a spinner with a message and manages its own render loop:

```rust
use rusty_rich::Status;
use std::time::Duration;

let mut status = Status::new("Downloading...");
status.start()?;

// Do work...
std::thread::sleep(Duration::from_secs(2));

status.update("Verifying...")?;
std::thread::sleep(Duration::from_secs(2));

status.stop()?;
```

---

## Usage with Progress SpinnerColumn

Inside a `Progress` display, `SpinnerColumn` uses the default `DEFAULT_SPINNER` (braille dots) and shows a green checkmark when the task is finished:

```rust
use rusty_rich::{
    Progress, Style, Color,
    progress_columns::{SpinnerColumn, TextColumn, BarColumn, TaskProgressColumn},
};

let mut progress = Progress::new();
progress.with_columns(vec![
    Box::new(SpinnerColumn::new()
        .style(Style::new().color(Color::parse("cyan").unwrap()))),
    Box::new(TextColumn::new("description")),
    Box::new(BarColumn::new().width(30)),
    Box::new(TaskProgressColumn::new()),
]);

let task = progress.add_task("Spinning", Some(100));
for i in 0..100 {
    progress.advance(task, 1.0);
    // In a real app, update fields and render via Live
    std::thread::sleep(std::time::Duration::from_millis(20));
}
```

---

## Spinner categories at a glance

| Category              | Spinners                                                              |
|-----------------------|-----------------------------------------------------------------------|
| Braille dots          | `dots`, `dots2`--`dots11` (11 spinners)                              |
| Arrows                | `arrow`, `arrow2`, `arrow3`                                          |
| Geometric             | `arc`, `circle`, `triangle`, `toggle`, `hamburger`                    |
| Growth / bars         | `growHorizontal`, `growVertical`, `verticalBars`, `bouncingBar`, `bouncingBall`, `noise` |
| Pong                  | `pong`                                                                |
| Nature / celestial    | `moon`, `earth`                                                       |
| Emoji / icons         | `christmas`, `clock`, `hearts`, `monkey`, `runner`, `shark`, `smiley`, `grenade` |
| Classic / utility     | `line`, `simpleDots`                                                  |

---

## Module structure

All spinner items are re-exported from the crate root.

- `rusty_rich::Spinner` -- animated spinner with text and style
- `rusty_rich::SpinnerFrames` -- frame data
- `rusty_rich::get_spinner()` -- runtime name-based lookup
- `rusty_rich::SPINNERS` -- all registered spinner entries
- `rusty_rich::SPINNER_*` -- individual spinner constants (e.g. `SPINNER_DOTS`, `SPINNER_ARC`)
- `rusty_rich::DEFAULT_SPINNER` -- the default spinner (braille dots)

Source file: `/root/tuiproject/rust-rich/src/spinner.rs`
