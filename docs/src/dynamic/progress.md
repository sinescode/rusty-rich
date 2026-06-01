# Progress

Progress bars and task tracking -- a Rust port of Python Rich's `progress.py` and `progress_bar.py`. The progress module provides a multi-task progress display system with customizable columns, automatic refresh, file download tracking, and iterator wrapping.

---

## Progress

`Progress` is the central orchestrator. It manages a list of `Task` values and renders them as rows, with one row per task, using either a default layout or custom column definitions.

```rust
use rusty_rich::Progress;

let mut progress = Progress::new();
```

### Configuration

| Field               | Type                          | Default  | Description                                   |
|----------------------|-------------------------------|----------|-----------------------------------------------|
| `auto_refresh`       | `bool`                        | `true`   | Whether to automatically refresh the display  |
| `refresh_per_second` | `f64`                         | `4.0`    | Refresh rate in Hz                            |
| `transient`          | `bool`                        | `false`  | Remove progress display on completion         |
| `columns`            | `Option<Vec<Box<dyn ProgressColumn>>>` | `None` | Custom column layout (None = default render) |

Use `with_columns()` to set a custom column layout:

```rust
progress.with_columns(vec![
    Box::new(SpinnerColumn::new()),
    Box::new(TextColumn::new("description")),
    Box::new(BarColumn::new()),
    Box::new(TaskProgressColumn::new()),
]);
```

### add_task

Add a new task and return its unique ID.

```rust
pub fn add_task(
    &mut self,
    description: impl Into<String>,
    total: Option<f64>,
) -> usize
```

- `description` -- a human-readable label for the task.
- `total` -- the number of steps expected. `None` means indeterminate (no bar, no percentage).

```rust
let task_id = progress.add_task("Downloading...", Some(100.0));
```

Task IDs start at 1 and increment monotonically.

### advance

Advance a task by a delta (positive number of steps completed).

```rust
pub fn advance(&mut self, task_id: usize, delta: f64)
```

```rust
progress.advance(task_id, 25.0); // 25 steps closer to completion
```

If the delta pushes `completed` past `total`, `completed` is clamped to `total`.

### update

Set a task's completed value directly.

```rust
pub fn update(&mut self, task_id: usize, completed: f64)
```

```rust
progress.update(task_id, 50.0); // set to exactly 50
```

### remove_task

Remove a task from the display entirely.

```rust
pub fn remove_task(&mut self, task_id: usize)
```

```rust
progress.remove_task(task_id); // task is gone from the output
```

### advance_bytes

Advance a task by a byte count (converts `u64` to `f64` internally).

```rust
pub fn advance_bytes(&mut self, task_id: usize, bytes: u64)
```

Useful when tracking I/O progress.

### track

Wrap an iterator with progress tracking. Returns a `TrackIterator` that reports progress as items are consumed.

```rust
pub fn track<I: IntoIterator>(
    &mut self,
    sequence: I,
    description: impl Into<String>,
    total: Option<f64>,
) -> TrackIterator<I::IntoIter>
```

If `total` is `None`, the iterator's `size_hint` upper bound (or lower bound) is used as the total.

```rust
for item in progress.track(0..100, "Processing", None) {
    // item: i32
    std::thread::sleep(std::time::Duration::from_millis(10));
}
```

See [TrackIterator](#trackiterator) below for details on the wrapper.

### open

Open a file and return a `ProgressFile` with a new task registered for tracking the read progress. The file size is used as the total.

```rust
pub fn open(
    &mut self,
    path: impl AsRef<std::path::Path>,
    description: impl Into<String>,
) -> std::io::Result<ProgressFile>
```

```rust
let mut pf = progress.open("data.bin", "Downloading data.bin")?;
let mut buf = Vec::new();
pf.read_to_end(&mut buf)?;
pf.sync(&mut progress); // update progress with bytes read
```

### wrap_file

Wrap an existing `std::fs::File` with progress tracking when you already have the file handle and know the total size.

```rust
pub fn wrap_file(
    &mut self,
    file: std::fs::File,
    total: u64,
    description: impl Into<String>,
) -> ProgressFile
```

```rust
let file = std::fs::File::open("archive.tar.gz")?;
let metadata = file.metadata()?;
let pf = progress.wrap_file(file, metadata.len(), "Extracting");
```

### render

Render all visible tasks to a string. Used by the `Live` display or for manual drawing.

```rust
pub fn render(&self, width: usize) -> String
```

---

## Task

A single tracked task within a `Progress` display.

```rust
use rusty_rich::Task;
```

### Fields

| Field         | Type                       | Description                                      |
|---------------|----------------------------|--------------------------------------------------|
| `id`          | `usize`                    | Unique task identifier                           |
| `description` | `String`                   | Human-readable label                             |
| `total`       | `Option<f64>`              | Total steps (`None` = indeterminate)             |
| `completed`   | `f64`                      | Steps completed so far                           |
| `visible`     | `bool`                     | Whether the task is rendered                     |
| `start_time`  | `Instant`                  | When the task was created                        |
| `fields`      | `HashMap<String, String>`  | Arbitrary key-value metadata for custom columns  |

Tasks are not constructed directly -- they are created via `Progress::add_task()`.

### progress

Return the fraction complete (0.0 -- 1.0). Returns 0.0 for indeterminate tasks.

```rust
pub fn progress(&self) -> f64
```

### elapsed

Return the `Duration` since the task was created.

```rust
pub fn elapsed(&self) -> Duration
```

### time_remaining

Estimate the remaining duration based on the progress so far. Returns `None` if no progress has been made yet or the task is indeterminate.

```rust
pub fn time_remaining(&self) -> Option<Duration>
```

The estimate is `elapsed / progress - elapsed` -- a linear extrapolation. This is most accurate for tasks with a steady, predictable rate.

### is_finished

Return `true` if `completed >= total` (tasks without a total are never finished).

```rust
pub fn is_finished(&self) -> bool
```

---

## ProgressBar

`ProgressBar` is a standalone bar renderer. It is used both as a building block inside `BarColumn` and directly for one-off bar rendering.

```rust
use rusty_rich::ProgressBar;
```

### Fields

| Field              | Type              | Default | Description                              |
|--------------------|-------------------|---------|------------------------------------------|
| `total`            | `Option<f64>`     | `100.0` | Total steps (`None` = indeterminate)     |
| `completed`        | `f64`             | `0.0`   | Steps completed                          |
| `width`            | `Option<usize>`   | `None`  | Bar width in characters                  |
| `complete_char`    | `char`            | `'█'`   | Character for the completed portion      |
| `remaining_char`   | `char`            | `'░'`   | Character for the remaining portion      |
| `pulse`            | `bool`            | `false` | If true, render a pulsing indeterminate bar |
| `complete_style`   | `Style`           | plain   | Style for completed portion              |
| `remaining_style`  | `Style`           | plain   | Style for remaining portion              |
| `pulse_style`      | `Style`           | plain   | Style for pulse cursor                   |

### Builder methods

| Method                            | Description                                |
|-----------------------------------|--------------------------------------------|
| `total(total: f64)`               | Set total steps                            |
| `completed(completed: f64)`       | Set completed steps                        |
| `width(width: usize)`             | Set bar width                              |
| `complete_style(style: Style)`    | Style for the filled portion               |
| `remaining_style(style: Style)`   | Style for the unfilled portion             |

### percentage

Get the progress as a fraction (0.0 -- 1.0).

```rust
pub fn percentage(&self) -> f64
```

### render

Render the bar to a string at the given width.

```rust
pub fn render(&self, width: usize) -> String
```

If `pulse` is true or `total` is `None`, an indeterminate bouncing-cursor animation is rendered instead of a filled bar.

```rust
let bar = ProgressBar::new()
    .total(100.0)
    .completed(67.0)
    .complete_style(Style::new().color(Color::parse("green").unwrap()));
println!("{}", bar.render(40));
// Example output: [██████████████████████████████░░░░░░░░░░░░]
```

---

## TrackIterator

An iterator wrapper returned by `Progress::track()`. It counts items as they are consumed, but does not automatically advance the associated progress task -- the calling code must drive that externally (typically by calling `progress.advance()` in a live render loop, or by letting `Live` redraw on each tick).

```rust
pub struct TrackIterator<I: Iterator> {
    pub progress_id: usize,
    // ...
}
```

| Method             | Return Type   | Description                        |
|--------------------|---------------|------------------------------------|
| `count()`          | `usize`       | Number of items yielded so far     |
| `total()`          | `f64`         | Expected total from size_hint      |

```rust
let mut progress = Progress::new();
let iter = progress.track(0..50, "Counting", Some(50.0));
for item in iter {
    // ...
}
```

---

## ProgressFile

A `std::fs::File` wrapper that counts bytes read so the progress can be synced back to a `Progress` instance.

```rust
use rusty_rich::ProgressFile;
```

### Methods

| Method                  | Return Type              | Description                                  |
|-------------------------|--------------------------|----------------------------------------------|
| `bytes_read()`          | `u64`                    | Bytes read so far                            |
| `total()`               | `u64`                    | Total file size                              |
| `task_id()`             | `usize`                  | The task ID associated with this file        |
| `sync(&self, &mut Progress)` | --                 | Update the progress task's `completed` field |
| `inner()`               | `&std::fs::File`         | Reference to the inner file                  |
| `inner_mut()`           | `&mut std::fs::File`     | Mutable reference to the inner file          |
| `into_inner(self)`      | `std::fs::File`          | Consume and return the inner file            |

It implements `std::io::Read`, so you can pass it to any function expecting `Read`.

```rust
let mut pf = progress.open("archive.iso", "Downloading")?;
let mut chunk = [0u8; 8192];
loop {
    let n = pf.read(&mut chunk)?;
    if n == 0 { break; }
    pf.sync(&mut progress); // update the bar
    // render or sleep as needed
}
```

---

## ProgressColumn trait

All column types implement the `ProgressColumn` trait:

```rust
pub trait ProgressColumn: std::fmt::Debug {
    fn render(&self, task: &Task, width: usize, elapsed: Duration) -> String;
}
```

- `task` -- the current task being rendered.
- `width` -- available character width for this column (can be ignored).
- `elapsed` -- time since the task started.

---

## Column types

There are 11 column types.

### BarColumn

Renders the actual progress bar.

```rust
use rusty_rich::BarColumn;

let mut col = BarColumn::new();
col = col.width(30);
col = col.complete_style(Style::new().color(Color::parse("cyan").unwrap()));
col = col.finished_style(Style::new().color(Color::parse("green").unwrap()));
```

| Builder method              | Description                         |
|-----------------------------|-------------------------------------|
| `width(w: usize)`           | Override bar width                  |
| `complete_style(s: Style)`  | Style for the filled bar portion    |
| `finished_style(s: Style)`  | Style for the remaining bar portion |

### SpinnerColumn

Shows an animated spinner while the task is running. When the task is finished, displays a checkmark ("✓") in green.

```rust
use rusty_rich::SpinnerColumn;

let col = SpinnerColumn::new();
```

| Builder method                    | Description                          |
|-----------------------------------|--------------------------------------|
| `style(s: Style)`                 | Style for the spinner frame          |
| `finished_style(s: Style)`        | Style for the completion indicator   |

The spinner uses the default `SPINNER_DOTS` animation (braille dots). The completion text defaults to "✓" styled in green.

### TextColumn

Displays a value from `task.fields` using a given format.

```rust
use rusty_rich::TextColumn;

let col = TextColumn::new("description")
    .format("{:>20}")
    .style(Style::new().bold(true));
```

| Builder method             | Description                              |
|----------------------------|------------------------------------------|
| `format(fmt: &str)`        | Format string (currently applied simply) |
| `style(s: Style)`          | Style for the rendered text              |

The column looks up `task.fields["key"]` and renders its value.

### TimeElapsedColumn

Shows how long the task has been running.

```rust
use rusty_rich::TimeElapsedColumn;

let col = TimeElapsedColumn::new();
```

Formats as `0:05` (minutes:seconds) or `1:02:30` (hours:minutes:seconds) as appropriate.

### TimeRemainingColumn

Shows estimated time remaining based on current progress.

```rust
use rusty_rich::TimeRemainingColumn;

let col = TimeRemainingColumn::new();
```

| Field                  | Type    | Default | Description                                      |
|------------------------|---------|---------|--------------------------------------------------|
| `elapsed_when_finished`| `bool`  | `false` | Show elapsed time instead of empty when finished |

Shows `?` when insufficient data is available to estimate. Uses the same linear extrapolation as `Task::time_remaining()`.

### TaskProgressColumn

Shows the percentage complete as text (e.g. " 42%").

```rust
use rusty_rich::TaskProgressColumn;

let col = TaskProgressColumn::new()
    .style(Style::new().color(Color::parse("yellow").unwrap()));
```

For indeterminate tasks (no total), renders nothing.

### MofNCompleteColumn

Shows "completed / total" as a pair of integers (e.g. "3/10").

```rust
use rusty_rich::MofNCompleteColumn;

let col = MofNCompleteColumn::new();
col.separator = " of ".to_string(); // customize separator
```

| Field       | Type     | Default | Description            |
|-------------|----------|---------|------------------------|
| `separator` | `String` | `"/"`   | Text between the counts |

For indeterminate tasks, shows only the completed count.

### FileSizeColumn

Shows the completed count formatted as a human-readable size (e.g. "1.5 MB").

```rust
use rusty_rich::FileSizeColumn;

let col = FileSizeColumn::new()
    .style(Style::new().color(Color::parse("blue").unwrap()));
```

Uses decimal (1000-based) units: B, KB, MB, GB, TB, PB.

### TotalFileSizeColumn

Shows the total size formatted as a human-readable size (e.g. "250.0 MB").

```rust
use rusty_rich::TotalFileSizeColumn;

let col = TotalFileSizeColumn::new();
```

For indeterminate tasks, renders nothing.

### DownloadColumn

Shows "completed / total" formatted as file sizes (e.g. "500.0 KB / 1.5 MB").

```rust
use rusty_rich::DownloadColumn;

let col = DownloadColumn::new()
    .style(Style::new().color(Color::parse("cyan").unwrap()))
    .separator(" of ");
```

| Field       | Type     | Default | Description            |
|-------------|----------|---------|------------------------|
| `separator` | `String` | `"/"`   | Text between the sizes |

### TransferSpeedColumn

Shows transfer speed in human-readable format (e.g. "1.5 MB/s").

```rust
use rusty_rich::TransferSpeedColumn;

let col = TransferSpeedColumn::new()
    .style(Style::new().color(Color::parse("green").unwrap()));
```

Speed is calculated as `completed / elapsed_seconds`. Shows "0 B/s" at the start.

---

## Helper functions

### format_size

Format a byte count into a human-readable string using decimal (1000-based) units.

```rust
pub fn format_size(bytes: f64) -> String
```

```rust
use rusty_rich::progress_columns::format_size;

assert_eq!(format_size(0.0), "0 B");
assert_eq!(format_size(500.0), "500 B");
assert_eq!(format_size(1500.0), "1.5 KB");
assert_eq!(format_size(2_500_000.0), "2.5 MB");
```

### format_speed

Format a transfer speed (bytes per second) into a human-readable string.

```rust
pub fn format_speed(bytes_per_sec: f64) -> String
```

```rust
use rusty_rich::progress_columns::format_speed;

assert_eq!(format_speed(0.0), "0 B/s");
assert_eq!(format_speed(1500.0), "1.5 KB/s");
```

---

## Examples

### Basic single-task progress

```rust
use rusty_rich::{Progress, ProgressBar, Style, Color};

let mut progress = Progress::new();
let task_id = progress.add_task("Processing items", Some(50.0));

for i in 0..50 {
    // Do some work
    std::thread::sleep(std::time::Duration::from_millis(20));
    progress.advance(task_id, 1.0);

    // Render the current state
    print!("\r{}", progress.render(60));
    std::io::Write::flush(&mut std::io::stdout()).ok();
}
println!();
```

### Multi-task progress

```rust
use rusty_rich::Progress;

let mut progress = Progress::new();
let download = progress.add_task("Downloading", Some(100.0));
let install  = progress.add_task("Installing",  Some(50.0));
let verify   = progress.add_task("Verifying",   Some(20.0));

for step in 0..100 {
    progress.advance(download, 1.0);
    if step >= 50 {
        progress.advance(install, 1.0);
    }
    if step >= 80 {
        progress.advance(verify, 1.0);
    }
    print!("\r{}", progress.render(60));
    std::io::Write::flush(&mut std::io::stdout()).ok();
    std::thread::sleep(std::time::Duration::from_millis(15));
}
// Remove finished tasks
progress.remove_task(download);
println!("\nDone!");
```

### File download with progress

```rust
use rusty_rich::Progress;
use std::io::Read;

let mut progress = Progress::new();
let mut pf = progress
    .open("large-file.iso", "Downloading large-file.iso")
    .expect("Could not open file");

let mut buffer = [0u8; 4096];
loop {
    let n = pf.read(&mut buffer).expect("Read error");
    if n == 0 {
        break;
    }
    pf.sync(&mut progress);
    print!("\r{}", progress.render(70));
    std::io::Write::flush(&mut std::io::stdout()).ok();
    std::thread::sleep(std::time::Duration::from_millis(5));
}
println!();
```

### Custom columns

```rust
use rusty_rich::{
    Progress, Style, Color,
    progress_columns::*,
};

let mut progress = Progress::new();
progress.with_columns(vec![
    Box::new(SpinnerColumn::new()
        .style(Style::new().color(Color::parse("cyan").unwrap()))),
    Box::new(TextColumn::new("description")
        .format("{:>20}")
        .style(Style::new().bold(true))),
    Box::new(BarColumn::new()
        .width(40)
        .complete_style(Style::new().color(Color::parse("bright_green").unwrap()))),
    Box::new(TaskProgressColumn::new()
        .style(Style::new().color(Color::parse("yellow").unwrap()))),
    Box::new(TimeElapsedColumn::new()),
    Box::new(TimeRemainingColumn::new()),
    Box::new(DownloadColumn::new()
        .style(Style::new().color(Color::parse("blue").unwrap()))),
    Box::new(TransferSpeedColumn::new()
        .style(Style::new().color(Color::parse("green").unwrap()))),
]);

let task = progress.add_task("Custom download", Some(200.0));

// In a real application, update task.fields if TextColumn references fields:
if let Some(t) = progress.tasks.iter_mut().find(|t| t.id == task) {
    t.fields.insert("description".into(), "Custom download".into());
}

for i in 0..200 {
    progress.advance(task, 1.0);
    print!("\r{}", progress.render(100));
    std::io::Write::flush(&mut std::io::stdout()).ok();
    std::thread::sleep(std::time::Duration::from_millis(10));
}
println!();
```

### Using ProgressBar standalone

```rust
use rusty_rich::{ProgressBar, Style, Color};

let bar = ProgressBar::new()
    .total(1.0)
    .completed(0.67)
    .complete_style(Style::new().color(Color::parse("green").unwrap()))
    .remaining_style(Style::new().color(Color::parse("bright_black").unwrap()));

println!("Compiling... {}", bar.render(40));
```

---

## Module structure

- `rusty_rich::Progress` -- multi-task progress display
- `rusty_rich::Task` -- a single tracked task
- `rusty_rich::ProgressBar` -- standalone bar renderer
- `rusty_rich::TrackIterator` -- iterator with progress tracking
- `rusty_rich::ProgressFile` -- file read with progress tracking
- `rusty_rich::progress_columns::{BarColumn, SpinnerColumn, TextColumn, ...}` -- column types
- `rusty_rich::progress_columns::{format_size, format_speed}` -- formatting helpers

All items are re-exported from the crate root. The module files are:

- `/root/tuiproject/rust-rich/src/progress.rs` -- Progress, Task, ProgressBar, TrackIterator, ProgressFile
- `/root/tuiproject/rust-rich/src/progress_columns.rs` -- all column types and formatting helpers
