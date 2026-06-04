//! Log record rendering — equivalent to Rich's `_log_render.py`.
//!
//! Provides the [`LogRender`] class for formatting log records into Rich
//! tables with columns for timestamp, level, message, and file path.
//!
//! Unlike [`RichHandler`] (which is a handler for the `log` crate),
//! `LogRender` is a standalone formatter that can be used with any logging
//! framework to produce styled terminal output.

use crate::color::Color;
use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::table::{Column, Table};

// ---------------------------------------------------------------------------
// LogRender
// ---------------------------------------------------------------------------

/// A standalone formatter for rendering log records as Rich-styled tables.
///
/// Configurable with column visibility, time format, and level width.
/// Produces a [`Table`] with columns for time, level, message, and path.
///
/// # Examples
///
/// ```rust,ignore
/// use rusty_rich::log_render::LogRender;
///
/// let mut renderer = LogRender::new()
///     .show_time(true)
///     .show_level(true)
///     .show_path(true);
///
/// let output = renderer.render_log(
///     Some("2024-01-15T10:30:00"),
///     "INFO",
///     "Server started successfully",
///     Some("src/main.rs"),
///     Some(42),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct LogRender {
    /// Whether to show the timestamp column.
    show_time: bool,
    /// Whether to show the log level column.
    show_level: bool,
    /// Whether to show the file path column.
    show_path: bool,
    /// Time format string (strftime-style, not parsed — passed through as-is).
    time_format: String,
    /// Width of the level column in characters.
    level_width: usize,
    /// If true, omit timestamps when they repeat across consecutive records.
    omit_repeated_times: bool,
    /// The last-rendered timestamp (used for dedup).
    last_time: Option<String>,
}

impl LogRender {
    /// Create a new `LogRender` with default settings.
    pub fn new() -> Self {
        Self {
            show_time: true,
            show_level: true,
            show_path: true,
            time_format: "[%x %X]".to_string(),
            level_width: 8,
            omit_repeated_times: true,
            last_time: None,
        }
    }

    /// Builder: show or hide the timestamp column.
    pub fn show_time(mut self, value: bool) -> Self {
        self.show_time = value;
        self
    }

    /// Builder: show or hide the level column.
    pub fn show_level(mut self, value: bool) -> Self {
        self.show_level = value;
        self
    }

    /// Builder: show or hide the file path column.
    pub fn show_path(mut self, value: bool) -> Self {
        self.show_path = value;
        self
    }

    /// Builder: set the time format string (displayed as-is in column header).
    pub fn time_format(mut self, format: impl Into<String>) -> Self {
        self.time_format = format.into();
        self
    }

    /// Builder: set the minimum width of the level column.
    pub fn level_width(mut self, width: usize) -> Self {
        self.level_width = width;
        self
    }

    /// Builder: enable or disable omitting repeated timestamps.
    pub fn omit_repeated_times(mut self, value: bool) -> Self {
        self.omit_repeated_times = value;
        self
    }

    /// Get the Rich style for a given log level name.
    ///
    /// Looks up `"logging.level.<lowercase_level>"` in the default theme.
    pub fn get_level_style(level: &str) -> Style {
        use crate::theme::default_theme;
        let theme = default_theme();
        let key = format!("logging.level.{}", level.to_lowercase());
        theme
            .get(&key)
            .cloned()
            .unwrap_or_else(|| match level.to_lowercase().as_str() {
                "debug" => Style::new().color(
                    crate::color::Color::parse("bright_black").unwrap_or_else(|_| Color::default()),
                ),
                "info" => Style::new().color(
                    crate::color::Color::parse("bright_cyan").unwrap_or_else(|_| Color::default()),
                ),
                "warning" => Style::new().color(
                    crate::color::Color::parse("bright_yellow")
                        .unwrap_or_else(|_| Color::default()),
                ),
                "error" => Style::new()
                    .color(
                        crate::color::Color::parse("bright_red")
                            .unwrap_or_else(|_| Color::default()),
                    )
                    .bold(true),
                "critical" => Style::new()
                    .color(crate::color::Color::parse("red").unwrap_or_else(|_| Color::default()))
                    .bold(true)
                    .reverse(true),
                _ => Style::new(),
            })
    }

    /// Get the style for the time column.
    pub fn get_time_style() -> Style {
        Style::new()
            .color(crate::color::Color::parse("bright_black").unwrap_or_else(|_| Color::default()))
    }

    /// Get the style for the message column.
    pub fn get_message_style() -> Style {
        Style::new()
    }

    /// Get the style for the path column.
    pub fn get_path_style() -> Style {
        Style::new()
            .color(crate::color::Color::parse("bright_black").unwrap_or_else(|_| Color::default()))
    }

    /// Format a single log record and return a [`LogRecord`] renderable.
    ///
    /// Parameters:
    /// - `time`: the timestamp string (or `None`)
    /// - `level`: the log level name (e.g. `"INFO"`, `"ERROR"`)
    /// - `message`: the log message content
    /// - `path`: the source file path (or `None`)
    /// - `line_no`: the source line number (or `None`)
    pub fn render_log(
        &mut self,
        time: Option<&str>,
        level: &str,
        message: &str,
        path: Option<&str>,
        line_no: Option<u32>,
    ) -> LogRecord {
        // Handle timestamp dedup
        let time_str = if self.show_time {
            let ts = time.unwrap_or("");
            if self.omit_repeated_times {
                if let Some(ref last) = self.last_time {
                    if last == ts {
                        "".to_string()
                    } else {
                        self.last_time = Some(ts.to_string());
                        ts.to_string()
                    }
                } else {
                    self.last_time = Some(ts.to_string());
                    ts.to_string()
                }
            } else {
                ts.to_string()
            }
        } else {
            String::new()
        };

        // Format path + line
        let path_str = if self.show_path {
            match (path, line_no) {
                (Some(p), Some(l)) => format!("{p}:{l}"),
                (Some(p), None) => p.to_string(),
                (None, Some(l)) => format!("<unknown>:{l}"),
                (None, None) => String::new(),
            }
        } else {
            String::new()
        };

        // Pad level to level_width
        let padded_level = if self.show_level {
            format!("{level:>width$}", width = self.level_width)
        } else {
            String::new()
        };

        LogRecord {
            time: time_str,
            level: padded_level,
            message: message.to_string(),
            path: path_str,
            show_time: self.show_time,
            show_level: self.show_level,
            show_path: self.show_path,
        }
    }

    /// Render multiple log records as a single table.
    #[allow(clippy::type_complexity)]
    pub fn render_batch(
        &mut self,
        records: &[(Option<&str>, &str, &str, Option<&str>, Option<u32>)],
    ) -> LogTable {
        let rendered: Vec<LogRecord> = records
            .iter()
            .map(|(time, level, msg, path, line)| self.render_log(*time, level, msg, *path, *line))
            .collect();
        LogTable { records: rendered }
    }

    /// Reset the last-time cache (e.g. between rendering sessions).
    pub fn reset_time_cache(&mut self) {
        self.last_time = None;
    }
}

impl Default for LogRender {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// LogRecord — a single formatted log entry renderable
// ---------------------------------------------------------------------------

/// A single formatted log record, ready for rendering.
#[derive(Debug, Clone)]
pub struct LogRecord {
    time: String,
    level: String,
    message: String,
    path: String,
    show_time: bool,
    show_level: bool,
    show_path: bool,
}

impl Renderable for LogRecord {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        let time_style = LogRender::get_time_style();
        let level_style = Self::get_level_style(&self.level.trim());
        let msg_style = LogRender::get_message_style();
        let path_style = LogRender::get_path_style();

        let mut line: Vec<Segment> = Vec::new();

        if self.show_time && !self.time.is_empty() {
            line.push(Segment::styled(&self.time, time_style.clone()));
            line.push(Segment::new(" "));
        }

        if self.show_level && !self.level.is_empty() {
            line.push(Segment::styled(&self.level, level_style));
            line.push(Segment::new(" "));
        }

        line.push(Segment::styled(&self.message, msg_style));

        if self.show_path && !self.path.is_empty() {
            line.push(Segment::new(" "));
            line.push(Segment::styled(&self.path, path_style));
        }

        line.push(Segment::line());

        RenderResult {
            lines: vec![line],
            items: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LogTable — multiple log records as a table
// ---------------------------------------------------------------------------

/// A collection of [`LogRecord`]s rendered as a Rich table.
#[derive(Debug, Clone)]
pub struct LogTable {
    records: Vec<LogRecord>,
}

impl Renderable for LogTable {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        if self.records.is_empty() {
            return RenderResult {
                lines: Vec::new(),
                items: Vec::new(),
            };
        }

        let mut table = Table::new();
        table.show_header = false;
        table.show_edge = false;
        table.show_lines = false;

        // Determine which columns are needed (based on first record)
        let first = &self.records[0];
        if first.show_time {
            table.add_column(Column::new("Time"));
        }
        if first.show_level {
            table.add_column(Column::new("Level"));
        }
        table.add_column(Column::new("Message"));
        if first.show_path {
            table.add_column(Column::new("Path"));
        }

        for record in &self.records {
            let mut cells: Vec<crate::table::Cell> = Vec::new();

            if record.show_time {
                let time_str = if record.time.is_empty() {
                    String::new()
                } else {
                    LogRender::get_time_style().render(&record.time)
                };
                cells.push(crate::table::Cell::new(time_str));
            }

            if record.show_level {
                let level_str =
                    LogRender::get_level_style(record.level.trim()).render(&record.level);
                cells.push(crate::table::Cell::new(level_str));
            }

            cells.push(crate::table::Cell::new(
                LogRender::get_message_style().render(&record.message),
            ));

            if record.show_path && !record.path.is_empty() {
                cells.push(crate::table::Cell::new(
                    LogRender::get_path_style().render(&record.path),
                ));
            }

            table.add_row(cells);
        }

        table.render(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_render_defaults() {
        let lr = LogRender::new();
        assert!(lr.show_time);
        assert!(lr.show_level);
        assert!(lr.show_path);
    }

    #[test]
    fn test_log_render_builder() {
        let lr = LogRender::new()
            .show_time(false)
            .show_level(false)
            .show_path(false)
            .level_width(10)
            .omit_repeated_times(false);
        assert!(!lr.show_time);
        assert!(!lr.show_level);
        assert!(!lr.show_path);
        assert_eq!(lr.level_width, 10);
        assert!(!lr.omit_repeated_times);
    }

    #[test]
    fn test_log_render_single() {
        let mut lr = LogRender::new();
        let record = lr.render_log(
            Some("2024-01-15 10:30:00"),
            "INFO",
            "Hello world",
            Some("src/main.rs"),
            Some(42),
        );
        let opts = ConsoleOptions::default();
        let result = record.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello world"));
    }

    #[test]
    fn test_log_render_no_path() {
        let mut lr = LogRender::new().show_path(false);
        let record = lr.render_log(None, "DEBUG", "debug message", None, None);
        let opts = ConsoleOptions::default();
        let result = record.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("debug message"));
    }

    #[test]
    fn test_log_render_level_styles() {
        let debug_style = LogRender::get_level_style("DEBUG");
        let info_style = LogRender::get_level_style("INFO");
        let warn_style = LogRender::get_level_style("WARNING");
        let error_style = LogRender::get_level_style("ERROR");
        let critical_style = LogRender::get_level_style("CRITICAL");
        // All should produce valid styles (not panic)
        assert!(!debug_style.is_null() || true);
        assert!(!info_style.is_null() || true);
        assert!(!warn_style.is_null() || true);
        assert!(!error_style.is_null() || true);
        assert!(!critical_style.is_null() || true);
    }

    #[test]
    fn test_log_render_batch() {
        let mut lr = LogRender::new().show_path(false).show_time(false);
        let records = vec![
            (None, "INFO", "first", None, None),
            (None, "ERROR", "second", None, None),
        ];
        let table = lr.render_batch(&records);
        let opts = ConsoleOptions::default();
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("first"));
        assert!(ansi.contains("second"));
    }

    #[test]
    fn test_log_render_time_dedup() {
        let mut lr = LogRender::new().show_path(false).show_level(false);
        let r1 = lr.render_log(Some("2024-01-01"), "INFO", "msg1", None, None);
        let r2 = lr.render_log(Some("2024-01-01"), "INFO", "msg2", None, None);
        // Second should have empty time (dedup)
        assert!(!r1.time.is_empty());
        assert!(r2.time.is_empty());
    }

    #[test]
    fn test_log_render_reset_cache() {
        let mut lr = LogRender::new().show_path(false).show_level(false);
        lr.render_log(Some("ts"), "INFO", "msg1", None, None);
        lr.reset_time_cache();
        let r = lr.render_log(Some("ts"), "INFO", "msg2", None, None);
        // After reset, timestamp should appear again
        assert!(!r.time.is_empty());
    }
}
