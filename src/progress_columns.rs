//! Progress column types — equivalent to Python Rich's progress column
//! system (SpinnerColumn, BarColumn, TextColumn, etc.).

use std::time::Instant;

use crate::progress::{ProgressBar, Task};
use crate::spinner::Spinner;
use crate::style::{Style, StyleType};

// ---------------------------------------------------------------------------
// ProgressColumn trait
// ---------------------------------------------------------------------------

/// A column in a progress display. Each column renders one cell per task.
pub trait ProgressColumn: std::fmt::Debug {
    /// Render this column for the given task into a string.
    fn render(&self, task: &Task, width: usize, elapsed: std::time::Duration) -> String;
}

// ---------------------------------------------------------------------------
// TextColumn
// ---------------------------------------------------------------------------

/// Displays a formatted text field. The text is taken from `task.fields["key"]`
/// and formatted with the given format string.
#[derive(Debug, Clone)]
pub struct TextColumn {
    /// Key into the task's `fields` HashMap.
    pub key: String,
    /// Format string (e.g. "{:>10}").
    pub format: String,
    /// Style for the text.
    pub style: Style,
}

impl TextColumn {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), format: "{:>11}".to_string(), style: Style::new() }
    }

    pub fn format(mut self, fmt: impl Into<String>) -> Self { self.format = fmt.into(); self }
    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
}

impl ProgressColumn for TextColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let value = task.fields.get(&self.key).map(|s| s.as_str()).unwrap_or("?");
        // Simple: just return the value (formatting could use format args but
        // we keep it simple)
        let ansi = self.style.to_ansi();
        let reset = self.style.reset_ansi();
        format!("{ansi}{value}{reset}")
    }
}

// ---------------------------------------------------------------------------
// BarColumn
// ---------------------------------------------------------------------------

/// Renders the progress bar itself.
#[derive(Debug, Clone)]
pub struct BarColumn {
    /// The underlying progress bar template.
    pub bar: ProgressBar,
    /// Width override (None = auto from available space).
    pub width: Option<usize>,
}

impl BarColumn {
    pub fn new() -> Self {
        Self { bar: ProgressBar::new(), width: None }
    }

    pub fn complete_style(mut self, s: Style) -> Self { self.bar = self.bar.complete_style(s); self }
    pub fn finished_style(mut self, s: Style) -> Self { self.bar = self.bar.remaining_style(s); self }
    pub fn width(mut self, w: usize) -> Self { self.width = Some(w); self }
}

impl ProgressColumn for BarColumn {
    fn render(&self, task: &Task, width: usize, _elapsed: std::time::Duration) -> String {
        let w = self.width.unwrap_or(width.saturating_sub(2));
        let mut bar = self.bar.clone();
        bar.total = task.total;
        bar.completed = task.completed;
        bar.width = Some(w);
        bar.render(w)
    }
}

impl Default for BarColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// SpinnerColumn
// ---------------------------------------------------------------------------

/// Shows a spinner for tasks that are not finished, and "✓" when complete.
#[derive(Debug, Clone)]
pub struct SpinnerColumn {
    pub spinner: Spinner,
    pub style: Style,
    pub finished_style: Style,
    pub finished_text: String,
}

impl SpinnerColumn {
    pub fn new() -> Self {
        Self {
            spinner: Spinner::default(),
            style: Style::new(),
            finished_style: Style::new().color(crate::color::Color::parse("green").unwrap()).bold(true),
            finished_text: "✓".to_string(),
        }
    }

    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
    pub fn finished_style(mut self, s: Style) -> Self { self.finished_style = s; self }
}

impl ProgressColumn for SpinnerColumn {
    fn render(&self, task: &Task, _width: usize, elapsed: std::time::Duration) -> String {
        if task.is_finished() {
            let a = self.finished_style.to_ansi();
            let r = self.finished_style.reset_ansi();
            format!("{a}{}{r}", self.finished_text)
        } else {
            let frame = self.spinner.frame_at(elapsed);
            let a = self.style.to_ansi();
            let r = self.style.reset_ansi();
            format!("{a}{frame}{r}")
        }
    }
}

impl Default for SpinnerColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// TimeElapsedColumn
// ---------------------------------------------------------------------------

/// Shows elapsed time since task started.
#[derive(Debug, Clone)]
pub struct TimeElapsedColumn {
    pub style: Style,
    pub paused_style: Style,
}

impl TimeElapsedColumn {
    pub fn new() -> Self {
        Self { style: Style::new(), paused_style: Style::new().dim(true) }
    }
}

impl ProgressColumn for TimeElapsedColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let d = task.elapsed();
        let s = format_duration_short(&d);
        let a = self.style.to_ansi();
        let r = self.style.reset_ansi();
        format!("{a}{s}{r}")
    }
}

impl Default for TimeElapsedColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// TimeRemainingColumn
// ---------------------------------------------------------------------------

/// Shows estimated time remaining.
#[derive(Debug, Clone)]
pub struct TimeRemainingColumn {
    pub style: Style,
    pub elapsed_when_finished: bool,
}

impl TimeRemainingColumn {
    pub fn new() -> Self {
        Self { style: Style::new(), elapsed_when_finished: false }
    }
}

impl ProgressColumn for TimeRemainingColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let text = if task.is_finished() {
            if self.elapsed_when_finished {
                format_duration_short(&task.elapsed())
            } else {
                String::new()
            }
        } else {
            task.time_remaining()
                .map(|d| format_duration_short(&d))
                .unwrap_or_else(|| "?".to_string())
        };

        let a = self.style.to_ansi();
        let r = self.style.reset_ansi();
        format!("{a}{text}{r}")
    }
}

impl Default for TimeRemainingColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// TaskProgressColumn
// ---------------------------------------------------------------------------

/// Shows percentage complete as text.
#[derive(Debug, Clone)]
pub struct TaskProgressColumn {
    pub style: Style,
}

impl TaskProgressColumn {
    pub fn new() -> Self { Self { style: Style::new() } }

    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
}

impl ProgressColumn for TaskProgressColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        if task.total.is_some() {
            let pct = (task.progress() * 100.0) as usize;
            let s = format!("{pct:>3}%");
            let a = self.style.to_ansi();
            let r = self.style.reset_ansi();
            format!("{a}{s}{r}")
        } else {
            String::new()
        }
    }
}

impl Default for TaskProgressColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// MofNCompleteColumn
// ---------------------------------------------------------------------------

/// Shows "completed / total" style.
#[derive(Debug, Clone)]
pub struct MofNCompleteColumn {
    pub style: Style,
    pub separator: String,
}

impl MofNCompleteColumn {
    pub fn new() -> Self {
        Self { style: Style::new(), separator: "/".to_string() }
    }
}

impl ProgressColumn for MofNCompleteColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let completed = task.completed as usize;
        if let Some(total) = task.total {
            let total = total as usize;
            let s = format!("{completed}{}{total}", self.separator);
            let a = self.style.to_ansi();
            let r = self.style.reset_ansi();
            format!("{a}{s}{r}")
        } else {
            format!("{completed}")
        }
    }
}

impl Default for MofNCompleteColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Helper: short duration format
// ---------------------------------------------------------------------------

fn format_duration_short(d: &std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("0:{secs:02}")
    } else if secs < 3600 {
        format!("{}:{:02}", secs / 60, secs % 60)
    } else {
        format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    }
}

// ---------------------------------------------------------------------------
// Helper: file size formatting
// ---------------------------------------------------------------------------

/// Format bytes into human-readable form using decimal (1000-based) units.
pub fn format_size(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes;
    let mut unit_idx = 0;
    while value >= 1000.0 && unit_idx < UNITS.len() - 1 {
        value /= 1000.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{:.0} {}", value, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", value, UNITS[unit_idx])
    }
}

/// Format a transfer speed (bytes per second) into human-readable form.
pub fn format_speed(bytes_per_sec: f64) -> String {
    format!("{}/s", format_size(bytes_per_sec))
}

// ---------------------------------------------------------------------------
// FileSizeColumn
// ---------------------------------------------------------------------------

/// Shows the completed file size in human-readable format.
#[derive(Debug, Clone)]
pub struct FileSizeColumn {
    pub style: Style,
}

impl FileSizeColumn {
    pub fn new() -> Self {
        Self { style: Style::new() }
    }

    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
}

impl ProgressColumn for FileSizeColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let size = format_size(task.completed);
        let a = self.style.to_ansi();
        let r = self.style.reset_ansi();
        format!("{a}{size}{r}")
    }
}

impl Default for FileSizeColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// TotalFileSizeColumn
// ---------------------------------------------------------------------------

/// Shows the total file size in human-readable format.
#[derive(Debug, Clone)]
pub struct TotalFileSizeColumn {
    pub style: Style,
}

impl TotalFileSizeColumn {
    pub fn new() -> Self {
        Self { style: Style::new() }
    }

    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
}

impl ProgressColumn for TotalFileSizeColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let a = self.style.to_ansi();
        let r = self.style.reset_ansi();
        if let Some(total) = task.total {
            let size = format_size(total);
            format!("{a}{size}{r}")
        } else {
            String::new()
        }
    }
}

impl Default for TotalFileSizeColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// DownloadColumn
// ---------------------------------------------------------------------------

/// Shows "completed/total" with file size formatting.
#[derive(Debug, Clone)]
pub struct DownloadColumn {
    pub style: Style,
    pub separator: String,
}

impl DownloadColumn {
    pub fn new() -> Self {
        Self { style: Style::new(), separator: "/".to_string() }
    }

    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
    pub fn separator(mut self, sep: impl Into<String>) -> Self { self.separator = sep.into(); self }
}

impl ProgressColumn for DownloadColumn {
    fn render(&self, task: &Task, _width: usize, _elapsed: std::time::Duration) -> String {
        let a = self.style.to_ansi();
        let r = self.style.reset_ansi();
        let completed = format_size(task.completed);
        if let Some(total) = task.total {
            let total = format_size(total);
            format!("{a}{completed}{}{total}{r}", self.separator)
        } else {
            format!("{a}{completed}{r}")
        }
    }
}

impl Default for DownloadColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// TransferSpeedColumn
// ---------------------------------------------------------------------------

/// Shows transfer speed in human-readable format (e.g., "1.5 MB/s").
#[derive(Debug, Clone)]
pub struct TransferSpeedColumn {
    pub style: Style,
}

impl TransferSpeedColumn {
    pub fn new() -> Self {
        Self { style: Style::new() }
    }

    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
}

impl ProgressColumn for TransferSpeedColumn {
    fn render(&self, task: &Task, _width: usize, elapsed: std::time::Duration) -> String {
        let secs = elapsed.as_secs_f64();
        let a = self.style.to_ansi();
        let r = self.style.reset_ansi();
        if secs > 0.0 && task.completed > 0.0 {
            let speed = task.completed / secs;
            let s = format_speed(speed);
            format!("{a}{s}{r}")
        } else {
            format!("{a}0 B/s{r}")
        }
    }
}

impl Default for TransferSpeedColumn {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::Task;

    #[test]
    fn test_text_column() {
        let col = TextColumn::new("name");
        let task = {
            let mut t = Task::new(1, "test", Some(100.0));
            t.fields.insert("name".into(), "Alice".into());
            t
        };
        let result = col.render(&task, 20, std::time::Duration::from_secs(5));
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_spinner_column() {
        let col = SpinnerColumn::new();
        let task = Task::new(1, "test", Some(100.0));
        let result = col.render(&task, 10, std::time::Duration::from_secs(1));
        assert!(!result.is_empty());
    }

    #[test]
    fn test_task_progress_column() {
        let col = TaskProgressColumn::new();
        let mut task = Task::new(1, "test", Some(100.0));
        task.completed = 42.0;
        let result = col.render(&task, 10, std::time::Duration::new(0, 0));
        assert!(result.contains("42%"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0.0), "0 B");
        assert_eq!(format_size(500.0), "500 B");
        assert_eq!(format_size(1500.0), "1.5 KB");
        assert_eq!(format_size(2_500_000.0), "2.5 MB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0.0), "0 B/s");
        assert_eq!(format_speed(1500.0), "1.5 KB/s");
    }

    #[test]
    fn test_file_size_column() {
        let col = FileSizeColumn::new();
        let mut task = Task::new(1, "test", Some(1000.0));
        task.completed = 500.0;
        let result = col.render(&task, 10, std::time::Duration::new(0, 0));
        assert!(result.contains("500 B"));
    }

    #[test]
    fn test_total_file_size_column() {
        let col = TotalFileSizeColumn::new();
        let task = Task::new(1, "test", Some(2_500_000.0));
        let result = col.render(&task, 10, std::time::Duration::new(0, 0));
        assert!(result.contains("2.5 MB"));
    }

    #[test]
    fn test_download_column() {
        let col = DownloadColumn::new();
        let mut task = Task::new(1, "test", Some(1_500_000.0));
        task.completed = 500_000.0;
        let result = col.render(&task, 10, std::time::Duration::new(0, 0));
        assert!(result.contains("500.0 KB"));
        assert!(result.contains("1.5 MB"));
    }

    #[test]
    fn test_transfer_speed_column() {
        let col = TransferSpeedColumn::new();
        let mut task = Task::new(1, "test", Some(1000.0));
        task.completed = 500.0;
        let result = col.render(&task, 10, std::time::Duration::from_secs(1));
        assert!(result.contains("500 B/s"));
    }
}
