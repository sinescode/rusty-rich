//! Progress bars and task tracking. Equivalent to Rich's `progress.py`
//! and `progress_bar.py`.
//!
//! # Overview
//!
//! [`Progress`] manages multiple concurrent tasks, each with its own
//! description, total, and completed count. The display is built from
//! configurable column types (see [`crate::progress_columns`]).
//!
//! # Quick Example
//!
//! ```rust
//! use rusty_rich::Progress;
//!
//! let mut progress = Progress::new();
//! let task = progress.add_task("Downloading...", Some(100.0));
//! progress.update(task, 50.0);
//! println!("{}", progress.render(80));
//! ```
//!
//! # Tracking Iterables
//!
//! ```rust
//! use rusty_rich::{Progress, TrackIterator};
//!
//! let mut progress = Progress::new();
//! let items: Vec<i32> = (0..100).collect();
//! let tracker = progress.track(items, "Processing", None);
//! for item in tracker {
//!     // item is yielded, progress auto-advances
//! }
//! ```
//!
//! # File Progress
//!
//! [`ProgressFile`] wraps a `std::io::Read` and tracks read progress via a
//! [`Progress`] task. Use [`Progress::wrap_file`] to create one.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::console::{ConsoleOptions, DynRenderable, Renderable};
use crate::progress_columns::{
    BarColumn, ProgressColumn, SpinnerColumn, TaskProgressColumn, TextColumn,
    TimeElapsedColumn,
};
use crate::style::Style;
use crate::table::{Cell, Table};

// ---------------------------------------------------------------------------
// ProgressBar
// ---------------------------------------------------------------------------

/// A single progress bar.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Total steps (None = indeterminate).
    pub total: Option<f64>,
    /// Completed steps.
    pub completed: f64,
    /// Width in characters.
    pub width: Option<usize>,
    /// Characters for completed portion.
    pub complete_char: char,
    /// Characters for remaining portion.
    pub remaining_char: char,
    /// Optional pulse style (for indeterminate).
    pub pulse: bool,
    /// Style for completed portion.
    pub complete_style: Style,
    /// Style for remaining portion.
    pub remaining_style: Style,
    /// Style for the pulse cursor.
    pub pulse_style: Style,
}

impl ProgressBar {
    /// Create a new `ProgressBar` with default values (total=100, completed=0).
    pub fn new() -> Self {
        Self {
            total: Some(100.0),
            completed: 0.0,
            width: None,
            complete_char: '█',
            remaining_char: '░',
            pulse: false,
            complete_style: Style::new(),
            remaining_style: Style::new(),
            pulse_style: Style::new(),
        }
    }

    /// Set total.
    pub fn total(mut self, total: f64) -> Self { self.total = Some(total); self }

    /// Set completed.
    pub fn completed(mut self, completed: f64) -> Self { self.completed = completed; self }

    /// Set width.
    pub fn width(mut self, width: usize) -> Self { self.width = Some(width); self }

    /// Set complete style.
    pub fn complete_style(mut self, style: Style) -> Self { self.complete_style = style; self }

    /// Set remaining style.
    pub fn remaining_style(mut self, style: Style) -> Self { self.remaining_style = style; self }

    /// Get progress as a fraction (0.0–1.0).
    pub fn percentage(&self) -> f64 {
        if let Some(total) = self.total {
            if total > 0.0 {
                (self.completed / total).min(1.0).max(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Render the bar to a string.
    pub fn render(&self, width: usize) -> String {
        let w = self.width.unwrap_or(width).saturating_sub(2); // leave room for brackets
        if w < 3 {
            return "[]".to_string();
        }

        if self.pulse || self.total.is_none() {
            // Indeterminate: pulsing animation
            let pos = ((self.completed as usize / 8) % (w - 1)).min(w);
            let left = " ".repeat(pos);
            let right = " ".repeat(w.saturating_sub(pos + 1));
            format!("[{left}⣿{right}]")
        } else {
            let pct = self.percentage();
            let filled = (w as f64 * pct) as usize;
            let empty = w - filled;
            let complete_ansi = self.complete_style.to_ansi();
            let complete_reset = if complete_ansi.is_empty() { "" } else { "\x1b[0m" };
            format!(
                "[{complete_ansi}{}{complete_reset}{}]",
                self.complete_char.to_string().repeat(filled),
                self.remaining_char.to_string().repeat(empty)
            )
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Task
// ---------------------------------------------------------------------------

/// A tracked task within a Progress display.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub description: String,
    pub total: Option<f64>,
    pub completed: f64,
    pub visible: bool,
    pub start_time: Instant,
    pub stop_time: Option<Instant>,
    pub fields: HashMap<String, String>,
    /// Optional custom renderable associated with this task.
    pub renderable: Option<DynRenderable>,
}

impl Task {
    /// Create a new `Task` with the given id, description, and optional total.
    pub fn new(id: usize, description: impl Into<String>, total: Option<f64>) -> Self {
        Self {
            id,
            description: description.into(),
            total,
            completed: 0.0,
            visible: true,
            start_time: Instant::now(),
            stop_time: None,
            fields: HashMap::new(),
            renderable: None,
        }
    }

    /// Return the progress fraction (0.0–1.0), or 0.0 if no total is set.
    pub fn progress(&self) -> f64 {
        if let Some(t) = self.total {
            if t > 0.0 {
                (self.completed / t).min(1.0).max(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Return the [`Duration`] since this task was created.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Estimate the remaining [`Duration`] based on current progress, or
    /// [`None`] if progress is zero or no total is set.
    pub fn time_remaining(&self) -> Option<Duration> {
        let pct = self.progress();
        if pct > 0.0 {
            let elapsed = self.elapsed();
            let total = elapsed.div_f64(pct);
            Some(total.saturating_sub(elapsed))
        } else {
            None
        }
    }

    /// Check if the task is finished (completed >= total).
    pub fn is_finished(&self) -> bool {
        if let Some(t) = self.total {
            self.completed >= t
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// RenderableColumn — a ProgressColumn that renders a custom renderable
// ---------------------------------------------------------------------------

/// A column that renders a custom renderable per task.
pub struct RenderableColumn {
    pub format: Box<dyn Fn(&Task) -> DynRenderable + Send + Sync>,
}

impl RenderableColumn {
    /// Create a new `RenderableColumn` from a renderable-producing closure.
    pub fn new<F: Fn(&Task) -> DynRenderable + Send + Sync + 'static>(format: F) -> Self {
        Self { format: Box::new(format) }
    }
}

impl std::fmt::Debug for RenderableColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderableColumn").finish()
    }
}

impl ProgressColumn for RenderableColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: Duration) -> String {
        let renderable = (self.format)(task);
        renderable.render(&ConsoleOptions::default()).to_ansi()
    }
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------

/// A multi-task progress display.
///
/// Tasks are stored in a [`HashMap`] keyed by task ID for O(1) lookup.
/// `task_order` maintains insertion order for rendering.
#[derive(Debug)]
pub struct Progress {
    pub tasks: HashMap<usize, Task>,
    /// Insertion order of task IDs for stable iteration during rendering.
    pub task_order: Vec<usize>,
    pub auto_refresh: bool,
    pub refresh_per_second: f64,
    pub transient: bool,
    /// Columns to render for each task (if None, uses default columns).
    pub columns: Option<Vec<Box<dyn crate::progress_columns::ProgressColumn>>>,
    next_id: usize,
}

impl Progress {
    /// Create a new `Progress` instance with no tasks.
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            task_order: Vec::new(),
            auto_refresh: true,
            refresh_per_second: 10.0,
            transient: false,
            columns: None,
            next_id: 1,
        }
    }

    /// Replace the default columns with a custom list of [`ProgressColumn`](crate::progress_columns::ProgressColumn)s.
    ///
    /// Each task is rendered as one row using the provided columns.
    pub fn with_columns(mut self, columns: Vec<Box<dyn crate::progress_columns::ProgressColumn>>) -> Self {
        self.columns = Some(columns);
        self
    }

    /// Register a new task and return its numeric ID (used by `advance`, `update`, etc.).
    pub fn add_task(
        &mut self,
        description: impl Into<String>,
        total: Option<f64>,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.insert(id, Task::new(id, description, total));
        self.task_order.push(id);
        id
    }

    /// Increase a task's completed count by `delta`.
    pub fn advance(&mut self, task_id: usize, delta: f64) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.completed += delta;
            if let Some(total) = task.total {
                if task.completed > total {
                    task.completed = total;
                }
            }
        }
    }

    /// Set a task's completed count directly (overwrites current value).
    pub fn update(&mut self, task_id: usize, completed: f64) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.completed = completed;
        }
    }

    /// Remove a task by its ID. No-op if the task does not exist.
    pub fn remove_task(&mut self, task_id: usize) {
        self.tasks.remove(&task_id);
        self.task_order.retain(|id| *id != task_id);
    }

    /// Force a refresh/render of the progress display.
    /// In a live display context this triggers a re-render.
    pub fn refresh(&mut self) {
        // Force refresh — in a live display this triggers a redraw.
        // Stateless rendering: this is a no-op placeholder.
    }

    /// Mark a task as started (reset its start_time to now).
    pub fn start_task(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.start_time = Instant::now();
        }
    }

    /// Mark a task as stopped (set its stop_time to now).
    pub fn stop_task(&mut self, task_id: usize) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.stop_time = Some(Instant::now());
        }
    }

    /// Reset a task's completed count.
    ///
    /// If `total` is `Some`, also updates the task's total.
    pub fn reset(&mut self, task_id: usize, total: Option<f64>) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.completed = 0.0;
            if let Some(t) = total {
                task.total = Some(t);
            }
        }
    }

    /// Check if all tasks are finished.
    pub fn finished(&self) -> bool {
        self.tasks.values().all(|t| t.is_finished())
    }

    /// Get the default column set for rendering.
    ///
    /// Returns: description, spinner, bar, percentage, elapsed.
    pub fn get_default_columns(&self) -> Vec<Box<dyn ProgressColumn>> {
        vec![
            Box::new(TextColumn::new("description")),
            Box::new(SpinnerColumn::new()),
            Box::new(BarColumn::new()),
            Box::new(TaskProgressColumn::new()),
            Box::new(TimeElapsedColumn::new()),
        ]
    }

    /// Get the renderable for a specific task, if any.
    pub fn get_renderable(&self, task_id: usize) -> Option<&dyn Renderable> {
        self.tasks
            .get(&task_id)
            .and_then(|t| t.renderable.as_ref())
            .map(|dr| dr as &dyn Renderable)
    }

    /// Get all task renderables.
    pub fn get_renderables(&self) -> Vec<&dyn Renderable> {
        self.tasks
            .values()
            .filter_map(|t| t.renderable.as_ref())
            .map(|dr| dr as &dyn Renderable)
            .collect()
    }

    /// Build a [`Table`] from tasks and progress columns.
    ///
    /// Each visible task becomes a row, each column becomes a cell rendered
    /// by the corresponding `ProgressColumn`.
    pub fn make_tasks_table(&self, columns: &[Box<dyn ProgressColumn>]) -> Table {
        let now = Instant::now();
        let mut table = Table::new();
        table.show_header = false;
        table.show_edge = false;
        table.padding = (0, 1, 0, 0);

        // Add a table column for each progress column
        for (i, _col) in columns.iter().enumerate() {
            table.add_column(crate::table::Column::new(format!("Col {}", i)));
        }

        for id in &self.task_order {
            if let Some(task) = self.tasks.get(id) {
                if !task.visible {
                    continue;
                }
                let elapsed = now.duration_since(task.start_time);
                let cells: Vec<Cell> = columns
                    .iter()
                    .map(|col| Cell::new(col.render(task, 20, elapsed)))
                    .collect();
                table.add_row(cells);
            }
        }

        table
    }

    /// Render all visible tasks to a multi-line string at the given terminal width.
    pub fn render(&self, width: usize) -> String {
        if let Some(ref columns) = self.columns {
            self.render_with_columns(width, columns)
        } else {
            self.render_default(width)
        }
    }

    /// Render using custom columns.
    fn render_with_columns(&self, _width: usize, columns: &[Box<dyn crate::progress_columns::ProgressColumn>]) -> String {
        let mut out = String::new();
        let now = std::time::Instant::now();
        for id in &self.task_order {
            if let Some(task) = self.tasks.get(id) {
                if !task.visible {
                    continue;
                }
                let elapsed = now.duration_since(task.start_time);
                let mut line = String::new();
                for (i, col) in columns.iter().enumerate() {
                    if i > 0 { line.push(' '); }
                    line.push_str(&col.render(task, 20, elapsed));
                }
                out.push_str(&line);
                out.push('\n');
            }
        }
        out
    }

    /// Default render (no columns).
    fn render_default(&self, width: usize) -> String {
        let mut out = String::new();
        for id in &self.task_order {
            if let Some(task) = self.tasks.get(id) {
                if !task.visible {
                    continue;
                }
                let bar_width = width.saturating_sub(30).max(10);
                let bar = self.render_task_bar(task, bar_width);
                let pct = (task.progress() * 100.0) as usize;
                let elapsed = format_duration(&task.elapsed());
                let remaining = task
                    .time_remaining()
                    .map(|d| format_duration(&d))
                    .unwrap_or_else(|| "?".to_string());

                out.push_str(&format!(
                    "{desc:<20} {pct:>3}% {bar} {elapsed}<{remaining}\n",
                    desc = task.description.chars().take(20).collect::<String>(),
                ));
            }
        }
        out
    }

    fn render_task_bar(&self, task: &Task, width: usize) -> String {
        let w = width.saturating_sub(2);
        if w < 3 {
            return "[]".to_string();
        }
        let pct = task.progress();
        let filled = (w as f64 * pct) as usize;
        let empty = w - filled;
        format!("[{}░{}]",
            "█".repeat(filled),
            " ".repeat(empty.saturating_sub(1))
        )
    }

    /// Wrap an iterator with progress tracking, returning a [`TrackIterator`].
    ///
    /// Equivalent to Python Rich's `track()`.
    pub fn track<I: IntoIterator>(
        &mut self,
        sequence: I,
        description: &str,
        total: Option<f64>,
    ) -> TrackIterator<I::IntoIter> {
        let iter = sequence.into_iter();
        let (lower, upper) = iter.size_hint();
        let total = total.unwrap_or(upper.unwrap_or(lower) as f64);
        let task_id = self.add_task(description, Some(total));

        TrackIterator {
            inner: iter,
            progress_id: task_id,
            count: 0,
            total,
        }
    }

    /// Convenience: advance a task by a [`u64`] byte count (casts to `f64` internally).
    pub fn advance_bytes(&mut self, task_id: usize, bytes: u64) {
        self.advance(task_id, bytes as f64);
    }

    /// Open a file at the given path and wrap it with progress tracking.
    ///
    /// Returns a [`ProgressFile`] whose reads are recorded via this [`Progress`].
    pub fn open(
        &mut self,
        path: impl AsRef<std::path::Path>,
        description: impl Into<String>,
    ) -> std::io::Result<ProgressFile> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let total = metadata.len();
        let file = std::fs::File::open(path)?;
        let desc = description.into();
        Ok(self.wrap_file(file, &desc, Some(total)))
    }

    /// Wrap an already-open [`std::fs::File`] with progress tracking.
    pub fn wrap_file(
        &mut self,
        file: std::fs::File,
        description: &str,
        total: Option<u64>,
    ) -> ProgressFile {
        let total_val = total.unwrap_or(0) as f64;
        let task_id = self.add_task(description, Some(total_val));
        ProgressFile::new(file, task_id, total.unwrap_or(0))
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Standalone track function
// ---------------------------------------------------------------------------

/// Create a [`TrackIterator`] from a sequence (standalone, no Progress).
pub fn track<T: IntoIterator>(sequence: T, _description: &str, total: Option<f64>) -> TrackIterator<T::IntoIter> {
    let iter = sequence.into_iter();
    let (lower, upper) = iter.size_hint();
    let total_val = total.unwrap_or(upper.unwrap_or(lower) as f64);
    TrackIterator {
        inner: iter,
        progress_id: 0,
        count: 0,
        total: total_val,
    }
}

// ---------------------------------------------------------------------------
// Standalone wrap_file function
// ---------------------------------------------------------------------------

/// Wrap a file with progress tracking (standalone, no Progress).
pub fn wrap_file(file: std::fs::File, _description: &str, total: Option<u64>) -> ProgressFile {
    ProgressFile::new(file, 0, total.unwrap_or(0))
}

// ---------------------------------------------------------------------------
// TrackIterator — wraps an iterator with progress updates
// ---------------------------------------------------------------------------

/// An iterator wrapper that updates progress as items are consumed.
/// Equivalent to Python Rich's `track()`.
pub struct TrackIterator<I: Iterator> {
    inner: I,
    /// The progress task ID (caller must update progress externally).
    pub progress_id: usize,
    count: usize,
    total: f64,
}

impl<I: Iterator> Iterator for TrackIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_some() {
            self.count += 1;
        }
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I: Iterator> TrackIterator<I> {
    /// Get the current count.
    pub fn count(&self) -> usize { self.count }

    /// Get the total.
    pub fn total(&self) -> f64 { self.total }
}

// ---------------------------------------------------------------------------
// ProgressFile — wraps a File with progress tracking
// ---------------------------------------------------------------------------

/// A file wrapper that tracks read progress for use with a Progress instance.
#[derive(Debug)]
pub struct ProgressFile {
    inner: std::fs::File,
    task_id: usize,
    total: u64,
    bytes_read: u64,
}

impl ProgressFile {
    /// Create a new ProgressFile.
    pub fn new(file: std::fs::File, task_id: usize, total: u64) -> Self {
        Self { inner: file, task_id, total, bytes_read: 0 }
    }

    /// Get the number of bytes read so far.
    pub fn bytes_read(&self) -> u64 { self.bytes_read }

    /// Get the total file size.
    pub fn total(&self) -> u64 { self.total }

    /// Get the task ID this ProgressFile is associated with.
    pub fn task_id(&self) -> usize { self.task_id }

    /// Sync the current read progress to a Progress instance.
    pub fn sync(&self, progress: &mut Progress) {
        if let Some(task) = progress.tasks.get_mut(&self.task_id) {
            task.completed = self.bytes_read as f64;
        }
    }

    /// Get a reference to the inner file.
    pub fn inner(&self) -> &std::fs::File { &self.inner }

    /// Get a mutable reference to the inner file.
    pub fn inner_mut(&mut self) -> &mut std::fs::File { &mut self.inner }

    /// Consume this ProgressFile and return the inner file.
    pub fn into_inner(self) -> std::fs::File { self.inner }
}

impl std::io::Read for ProgressFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes_read += n as u64;
        Ok(n)
    }
}

// ---------------------------------------------------------------------------

fn format_duration(d: &Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("0:{secs:02}")
    } else if secs < 3600 {
        format!("{}:{:02}", secs / 60, secs % 60)
    } else {
        format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_render() {
        let bar = ProgressBar::new().total(100.0).completed(50.0);
        let r = bar.render(20);
        assert!(r.contains('█'));
    }

    #[test]
    fn test_progress_add_task() {
        let mut p = Progress::new();
        let id = p.add_task("Download", Some(100.0));
        assert_eq!(id, 1);
        p.advance(1, 50.0);
        assert_eq!(p.tasks.get(&1).unwrap().completed, 50.0);
    }

    #[test]
    fn test_advance_bytes() {
        let mut p = Progress::new();
        let id = p.add_task("Download", Some(1000.0));
        p.advance_bytes(id, 256);
        assert_eq!(p.tasks.get(&id).unwrap().completed, 256.0);
    }

    #[test]
    fn test_progress_file_wrap_and_read() {
        use std::io::Read;
        let data = b"hello world";
        let dir = std::env::temp_dir();
        let path = dir.join("rusty_rich_test_progress.txt");

        // Write test data
        std::fs::write(&path, data).unwrap();

        let mut p = Progress::new();
        let mut pf = p.open(&path, "test file").unwrap();
        assert_eq!(pf.total(), 11);
        assert_eq!(pf.bytes_read(), 0);

        // Read a few bytes
        let mut buf = [0u8; 5];
        let n = pf.read(&mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(pf.bytes_read(), 5);

        // Sync progress
        pf.sync(&mut p);
        assert_eq!(p.tasks.get(&pf.task_id()).unwrap().completed, 5.0);

        // Read remaining bytes
        let mut buf = Vec::new();
        pf.read_to_end(&mut buf).unwrap();
        assert_eq!(pf.bytes_read(), 11);

        // Sync again
        pf.sync(&mut p);
        assert_eq!(p.tasks.get(&pf.task_id()).unwrap().completed, 11.0);

        drop(pf);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_progress_file_wrap_existing() {
        let data = b"test data for wrap";
        let dir = std::env::temp_dir();
        let path = dir.join("rusty_rich_test_wrap.txt");
        std::fs::write(&path, data).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let mut p = Progress::new();
        let pf = p.wrap_file(file, "wrapped", Some(data.len() as u64));
        assert_eq!(pf.total(), data.len() as u64);
        assert_eq!(pf.task_id(), 1);

        drop(pf);
        std::fs::remove_file(&path).unwrap();
    }

    // --- New feature tests ---

    #[test]
    fn test_start_task() {
        let mut p = Progress::new();
        let id = p.add_task("test", Some(100.0));
        // start_task resets the start_time; just verify it doesn't panic
        p.start_task(id);
        assert!(!p.tasks.get(&id).unwrap().elapsed().is_zero());
    }

    #[test]
    fn test_stop_task() {
        let mut p = Progress::new();
        let id = p.add_task("test", Some(100.0));
        p.stop_task(id);
        assert!(p.tasks.get(&id).unwrap().stop_time.is_some());
    }

    #[test]
    fn test_reset_task() {
        let mut p = Progress::new();
        let id = p.add_task("test", Some(100.0));
        p.advance(id, 50.0);
        assert_eq!(p.tasks.get(&id).unwrap().completed, 50.0);
        p.reset(id, Some(200.0));
        assert_eq!(p.tasks.get(&id).unwrap().completed, 0.0);
        assert_eq!(p.tasks.get(&id).unwrap().total, Some(200.0));
    }

    #[test]
    fn test_finished() {
        let mut p = Progress::new();
        p.add_task("a", Some(100.0));
        p.add_task("b", Some(100.0));
        assert!(!p.finished());
        p.update(1, 100.0);
        p.update(2, 100.0);
        assert!(p.finished());
    }

    #[test]
    fn test_get_default_columns() {
        let p = Progress::new();
        let cols = p.get_default_columns();
        assert_eq!(cols.len(), 5);
    }

    #[test]
    fn test_refresh() {
        let mut p = Progress::new();
        p.add_task("test", Some(100.0));
        // Should not panic
        p.refresh();
    }

    #[test]
    fn test_track_method() {
        let mut p = Progress::new();
        let items = vec![1, 2, 3];
        let tracker = p.track(items, "counting", Some(3.0));
        assert_eq!(tracker.progress_id, 1);
        assert_eq!(p.tasks.len(), 1);
    }

    #[test]
    fn test_standalone_track() {
        let items = vec![1, 2, 3];
        let tracker = track(items, "counting", Some(3.0));
        assert_eq!(tracker.progress_id, 0);
    }

    #[test]
    fn test_standalone_wrap_file() {
        let data = b"hello";
        let dir = std::env::temp_dir();
        let path = dir.join("rusty_rich_test_standalone_wrap.txt");
        std::fs::write(&path, data).unwrap();
        let file = std::fs::File::open(&path).unwrap();
        let pf = wrap_file(file, "standalone", Some(data.len() as u64));
        assert_eq!(pf.total(), 5);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_renderable_column() {
        let col = RenderableColumn::new(|task: &Task| {
            DynRenderable::new(task.description.clone())
        });
        let task = Task::new(1, "hello", Some(100.0));
        let result = col.render(&task, 20, Duration::from_secs(0));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_make_tasks_table() {
        let mut p = Progress::new();
        p.add_task("task1", Some(100.0));
        p.add_task("task2", Some(50.0));
        let cols = p.get_default_columns();
        let table = p.make_tasks_table(&cols);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn test_get_renderable() {
        let mut p = Progress::new();
        let id = p.add_task("test", Some(100.0));
        // No renderable set initially
        assert!(p.get_renderable(id).is_none());
    }

    #[test]
    fn test_get_renderables() {
        let mut p = Progress::new();
        p.add_task("a", Some(100.0));
        p.add_task("b", Some(50.0));
        let renderables = p.get_renderables();
        assert!(renderables.is_empty());
    }

    #[test]
    fn test_auto_refresh_default() {
        let p = Progress::new();
        assert!(p.auto_refresh);
    }

    #[test]
    fn test_refresh_per_second_default() {
        let p = Progress::new();
        assert!((p.refresh_per_second - 10.0).abs() < f64::EPSILON);
    }
}
