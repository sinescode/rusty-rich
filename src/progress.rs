//! Progress bars and task tracking. Equivalent to Rich's `progress.py`
//! and `progress_bar.py`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::style::Style;

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
    pub fields: HashMap<String, String>,
}

impl Task {
    pub fn new(id: usize, description: impl Into<String>, total: Option<f64>) -> Self {
        Self {
            id,
            description: description.into(),
            total,
            completed: 0.0,
            visible: true,
            start_time: Instant::now(),
            fields: HashMap::new(),
        }
    }

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

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

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
// Progress
// ---------------------------------------------------------------------------

/// A multi-task progress display.
#[derive(Debug)]
pub struct Progress {
    pub tasks: Vec<Task>,
    pub auto_refresh: bool,
    pub refresh_per_second: f64,
    pub transient: bool,
    /// Columns to render for each task (if None, uses default columns).
    pub columns: Option<Vec<Box<dyn crate::progress_columns::ProgressColumn>>>,
    next_id: usize,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            columns: None,
            next_id: 1,
        }
    }

    /// Set custom columns for rendering.
    pub fn with_columns(mut self, columns: Vec<Box<dyn crate::progress_columns::ProgressColumn>>) -> Self {
        self.columns = Some(columns);
        self
    }

    /// Add a new task and return its ID.
    pub fn add_task(
        &mut self,
        description: impl Into<String>,
        total: Option<f64>,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task::new(id, description, total));
        id
    }

    /// Advance a task by a delta.
    pub fn advance(&mut self, task_id: usize, delta: f64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed += delta;
            if let Some(total) = task.total {
                if task.completed > total {
                    task.completed = total;
                }
            }
        }
    }

    /// Update a task's completed value.
    pub fn update(&mut self, task_id: usize, completed: f64) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = completed;
        }
    }

    /// Remove a completed task.
    pub fn remove_task(&mut self, task_id: usize) {
        self.tasks.retain(|t| t.id != task_id);
    }

    /// Render all tasks as a string.
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
        for task in &self.tasks {
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
        out
    }

    /// Default render (no columns).
    fn render_default(&self, width: usize) -> String {
        let mut out = String::new();
        for task in &self.tasks {
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

    /// Add a `track()` method that wraps an iterator with progress tracking.
    /// Equivalent to Python Rich's `track()`.
    pub fn track<I: IntoIterator>(
        &mut self,
        sequence: I,
        description: impl Into<String>,
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

    /// Advance a task by a number of bytes.
    pub fn advance_bytes(&mut self, task_id: usize, bytes: u64) {
        self.advance(task_id, bytes as f64);
    }

    /// Open a file and track read progress.
    pub fn open(
        &mut self,
        path: impl AsRef<std::path::Path>,
        description: impl Into<String>,
    ) -> std::io::Result<ProgressFile> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let total = metadata.len();
        let file = std::fs::File::open(path)?;
        Ok(self.wrap_file(file, total, description))
    }

    /// Wrap an existing file with progress tracking.
    pub fn wrap_file(
        &mut self,
        file: std::fs::File,
        total: u64,
        description: impl Into<String>,
    ) -> ProgressFile {
        let task_id = self.add_task(description, Some(total as f64));
        ProgressFile::new(file, task_id, total)
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
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
        if let Some(task) = progress.tasks.iter_mut().find(|t| t.id == self.task_id) {
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
        assert_eq!(p.tasks[0].completed, 50.0);
    }

    #[test]
    fn test_advance_bytes() {
        let mut p = Progress::new();
        let id = p.add_task("Download", Some(1000.0));
        p.advance_bytes(id, 256);
        assert_eq!(p.tasks[0].completed, 256.0);
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
        assert_eq!(p.tasks[0].completed, 5.0);

        // Read remaining bytes
        let mut buf = Vec::new();
        pf.read_to_end(&mut buf).unwrap();
        assert_eq!(pf.bytes_read(), 11);

        // Sync again
        pf.sync(&mut p);
        assert_eq!(p.tasks[0].completed, 11.0);

        drop(pf);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_progress_file_wrap_existing() {
        use std::io::Read;
        let data = b"test data for wrap";
        let dir = std::env::temp_dir();
        let path = dir.join("rusty_rich_test_wrap.txt");
        std::fs::write(&path, data).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let mut p = Progress::new();
        let pf = p.wrap_file(file, data.len() as u64, "wrapped");
        assert_eq!(pf.total(), data.len() as u64);
        assert_eq!(pf.task_id(), 1);

        drop(pf);
        std::fs::remove_file(&path).unwrap();
    }
}
