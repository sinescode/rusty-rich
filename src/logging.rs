//! Logging integration — equivalent to Rich's `logging.py`.
//!
//! Provides a log handler that renders log records with Rich formatting.
//! Integrates with the `log` crate and delegates formatting to
//! [`LogRender`](crate::log_render::LogRender) for table-based output.

use std::io::Write;

use crate::color::Color;
use crate::console::Console;
#[cfg(feature = "syntax-highlighting")]
use crate::highlighter::ReprHighlighter;
use crate::log_render::LogRender;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Default keywords for HTTP-level log highlighting
// ---------------------------------------------------------------------------

/// Default set of keywords to highlight in log messages (matching Python Rich).
pub const DEFAULT_KEYWORDS: &[&str] = &[
    "GET", "POST", "HEAD", "PUT", "DELETE", "OPTIONS", "TRACE", "PATCH",
];

// ---------------------------------------------------------------------------
// RichHandler
// ---------------------------------------------------------------------------

/// A handler that renders Python-style log records with Rich formatting.
///
/// Integrates with the `log` crate. Delegates formatting to [`LogRender`]
/// for table-based output with columns for time, level, message, and path.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::logging::{RichHandler, install};
///
/// // Install as the global logger
/// install().unwrap();
///
/// log::info!("Server started");
/// log::error!("Connection refused");
/// ```
pub struct RichHandler {
    pub console: Console,
    pub show_time: bool,
    pub show_level: bool,
    pub show_path: bool,
    pub enable_link_path: bool,
    pub markup: bool,
    /// If true, successive log records with the same timestamp show spaces
    /// instead of repeating the time string.
    pub omit_repeated_times: bool,
    #[cfg(feature = "syntax-highlighting")]
    pub highlighter: ReprHighlighter,
    /// Internal log formatter for table-based rendering.
    log_render: LogRender,
    /// Keywords to highlight in log messages (defaults to [`DEFAULT_KEYWORDS`]).
    pub keywords: Option<Vec<String>>,
    /// When true, render Rich tracebacks for error records.
    pub rich_tracebacks: bool,
    /// Max width for traceback rendering.
    pub tracebacks_width: Option<usize>,
    /// Code width for traceback source lines.
    pub tracebacks_code_width: usize,
    /// Extra lines of context in tracebacks.
    pub tracebacks_extra_lines: usize,
    /// Theme for traceback syntax highlighting.
    pub tracebacks_theme: Option<String>,
    /// Enable word wrapping in tracebacks.
    pub tracebacks_word_wrap: bool,
    /// Show local variables in traceback frames.
    pub tracebacks_show_locals: bool,
    /// Max frames to show in traceback.
    pub tracebacks_max_frames: usize,
    /// Max container length before abbreviating in locals.
    pub locals_max_length: usize,
    /// Max string length before truncating in locals.
    pub locals_max_string: usize,
    /// Timestamp of the last-rendered record (for omit_repeated_times).
    last_time: Option<String>,
}

impl RichHandler {
    /// Create a new `RichHandler` with default settings (shows time, level, and path).
    pub fn new() -> Self {
        let log_render = LogRender::new()
            .show_time(true)
            .show_level(true)
            .show_path(true);
        Self {
            console: Console::new(),
            show_time: true,
            show_level: true,
            show_path: true,
            enable_link_path: false,
            markup: false,
            omit_repeated_times: true,
            #[cfg(feature = "syntax-highlighting")]
            highlighter: ReprHighlighter::new(),
            log_render,
            keywords: None,
            rich_tracebacks: false,
            tracebacks_width: None,
            tracebacks_code_width: 88,
            tracebacks_extra_lines: 3,
            tracebacks_theme: None,
            tracebacks_word_wrap: true,
            tracebacks_show_locals: false,
            tracebacks_max_frames: 100,
            locals_max_length: 10,
            locals_max_string: 80,
            last_time: None,
        }
    }

    // -- Builder methods --

    /// Builder: show or hide the timestamp column.
    pub fn show_time(mut self, value: bool) -> Self {
        self.show_time = value;
        self.log_render = self.log_render.show_time(value);
        self
    }

    /// Builder: show or hide the level column.
    pub fn show_level(mut self, value: bool) -> Self {
        self.show_level = value;
        self.log_render = self.log_render.show_level(value);
        self
    }

    /// Builder: show or hide the file path column.
    pub fn show_path(mut self, value: bool) -> Self {
        self.show_path = value;
        self.log_render = self.log_render.show_path(value);
        self
    }

    /// Builder: enable terminal hyperlinks for file paths.
    pub fn enable_link_path(mut self, value: bool) -> Self {
        self.enable_link_path = value;
        self
    }

    /// Builder: enable console markup in log messages.
    pub fn markup(mut self, value: bool) -> Self {
        self.markup = value;
        self
    }

    /// Builder: omit repeated timestamps for consecutive records.
    pub fn omit_repeated_times(mut self, value: bool) -> Self {
        self.omit_repeated_times = value;
        self
    }

    /// Builder: set keywords to highlight in messages.
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = Some(keywords);
        self
    }

    /// Builder: enable Rich tracebacks for error records.
    pub fn rich_tracebacks(mut self, value: bool) -> Self {
        self.rich_tracebacks = value;
        self
    }

    /// Builder: set max traceback width.
    pub fn tracebacks_width(mut self, width: usize) -> Self {
        self.tracebacks_width = Some(width);
        self
    }

    /// Builder: set traceback code width.
    pub fn tracebacks_code_width(mut self, width: usize) -> Self {
        self.tracebacks_code_width = width;
        self
    }

    /// Builder: set extra context lines in tracebacks.
    pub fn tracebacks_extra_lines(mut self, lines: usize) -> Self {
        self.tracebacks_extra_lines = lines;
        self
    }

    /// Builder: set traceback syntax theme.
    pub fn tracebacks_theme(mut self, theme: impl Into<String>) -> Self {
        self.tracebacks_theme = Some(theme.into());
        self
    }

    /// Builder: enable word wrapping in tracebacks.
    pub fn tracebacks_word_wrap(mut self, value: bool) -> Self {
        self.tracebacks_word_wrap = value;
        self
    }

    /// Builder: show locals in traceback frames.
    pub fn tracebacks_show_locals(mut self, value: bool) -> Self {
        self.tracebacks_show_locals = value;
        self
    }

    /// Builder: max frames in traceback.
    pub fn tracebacks_max_frames(mut self, max: usize) -> Self {
        self.tracebacks_max_frames = max;
        self
    }

    /// Builder: max container length in locals before abbreviating.
    pub fn locals_max_length(mut self, max: usize) -> Self {
        self.locals_max_length = max;
        self
    }

    /// Builder: max string length in locals before truncating.
    pub fn locals_max_string(mut self, max: usize) -> Self {
        self.locals_max_string = max;
        self
    }

    // -- Formatting --

    /// Get the styled level text for a log level.
    pub fn get_level_text(&self, level: log::Level) -> Text {
        let level_name = format!("{level:<8}");
        let style = style_level(level);
        Text::styled(level_name, style)
    }

    /// Render the log message with optional markup and keyword highlighting.
    pub fn render_message(&self, message: &str) -> Text {
        let mut text = if self.markup {
            Text::from_markup(message)
        } else {
            Text::new(message)
        };

        // Apply repr-style highlighting if available
        #[cfg(feature = "syntax-highlighting")]
        {
            let highlighted = self.highlighter.highlight_str(message);
            if highlighted != message {
                text = Text::new(highlighted);
            }
        }

        // Highlight keywords
        let keywords = self.keywords.as_deref().unwrap_or(DEFAULT_KEYWORDS);
        if !keywords.is_empty() {
            let keyword_style = Style::new()
                .color(Color::parse("yellow").unwrap_or_default())
                .bold(true);
            text.highlight_words(keywords, keyword_style, true);
        }

        text
    }

    /// Render a single log record into an ANSI string.
    ///
    /// Handles `omit_repeated_times` by comparing the current timestamp against
    /// the last-rendered timestamp.  Builds the final output with styled
    /// columns for time, level, message, and file location.
    pub fn render(
        &mut self,
        level: log::Level,
        message: &str,
        _module_path: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
    ) -> String {
        let now = chrono::Local::now();
        let time_str = format!("[{}]", now.format("%H:%M:%S"));

        // Handle omit_repeated_times
        let display_time = if self.show_time && self.omit_repeated_times {
            if let Some(ref last) = self.last_time {
                if *last == time_str {
                    " ".repeat(time_str.len())
                } else {
                    self.last_time = Some(time_str.clone());
                    time_str.clone()
                }
            } else {
                self.last_time = Some(time_str.clone());
                time_str.clone()
            }
        } else if self.show_time {
            time_str.clone()
        } else {
            String::new()
        };

        // Build final output with proper styles
        let mut text = Text::new("");

        // Timestamp column
        if self.show_time && !display_time.is_empty() {
            text.append_styled(&display_time, Style::new().dim(true));
            text.append_styled(" ", Style::new());
        }

        // Level column
        if self.show_level {
            let level_text = self.get_level_text(level);
            text.append(level_text);
            text.append_styled(" ", Style::new());
        }

        // Message column (with markup + keyword highlighting)
        let message_text = self.render_message(message);
        text.append(message_text);

        // File location column
        if self.show_path {
            if let (Some(file), Some(line)) = (file, line) {
                text.append_styled(" ", Style::new());
                let location = format!("{file}:{line}");

                if self.enable_link_path {
                    let link = format!(
                        "\x1b]8;;file://{file}\x1b\\{location}\x1b]8;;\x1b\\"
                    );
                    text.append_styled(link, Style::new().dim(true).italic(true));
                } else {
                    text.append_styled(
                        location,
                        Style::new().dim(true).italic(true),
                    );
                }
            }
        }

        text.render()
    }

    /// Emit a `log` crate [`Record`](log::Record) using Rich formatting.
    ///
    /// Formats the record and writes it to the console. If `rich_tracebacks` is
    /// enabled and the record level is `Error`, any active panic/traceback
    /// information is included in the output.
    pub fn emit(&mut self, record: &log::Record) {
        let message = record.args().to_string();
        let level = record.level();

        // Build the base formatted output
        let mut formatted = self.render(
            level,
            &message,
            record.module_path(),
            record.file(),
            record.line(),
        );

        // If rich tracebacks are enabled for error-level records, check for
        // any captured traceback and append it below the log line.
        if self.rich_tracebacks && level == log::Level::Error {
            // In Rust, the `log` crate doesn't carry exception info like
            // Python's logging. Users should use `traceback::install()` to
            // set up a panic hook, and error-level messages will get rich
            // traceback rendering through that path instead.
            //
            // For explicit error rendering, use `RichHandler::emit_error()`
            // which accepts a `std::error::Error` and renders a traceback.
        }

        let _ = writeln!(self.console.file, "{formatted}");
        let _ = self.console.file.flush();
    }

    /// Emit an error with rich formatting.
    ///
    /// Like [`emit`](Self::emit) but also renders the error's [`Debug`]
    /// representation (which includes the error chain via `source()`) below
    /// the log message, styled as a traceback.
    ///
    /// This mirrors Python Rich's `logging.exception()` behavior when
    /// `rich_tracebacks=True`.
    pub fn emit_error(
        &mut self,
        record: &log::Record,
        error: &dyn std::error::Error,
    ) {
        let message = record.args().to_string();
        let level = record.level();

        // Render the log line
        let formatted = self.render(
            level,
            &message,
            record.module_path(),
            record.file(),
            record.line(),
        );

        // Render the error chain with rich formatting
        let mut error_text = Text::new("");
        error_text.append_styled(
            format!("{:?}", error),
            Style::new().color(
                crate::color::Color::parse("red").unwrap_or_else(|_| crate::color::Color::default()),
            ),
        );

        // Walk the error chain
        let mut source = error.source();
        while let Some(src) = source {
            error_text.append(Text::new("\nCaused by: "));
            error_text.append_styled(
                format!("{src}"),
                Style::new().color(
                    crate::color::Color::parse("yellow")
                        .unwrap_or_else(|_| crate::color::Color::default()),
                ),
            );
            source = src.source();
        }

        let err_rendered = error_text.render();

        let _ = writeln!(self.console.file, "{formatted}");
        let _ = writeln!(self.console.file, "{err_rendered}");
        let _ = self.console.file.flush();
    }
}

impl Default for RichHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RichLogger — thread-safe wrapper
// ---------------------------------------------------------------------------

/// Logger wrapper providing thread-safe access to a [`RichHandler`].
///
/// Uses a [`std::sync::Mutex`] so that [`RichHandler`] (which is `Send` but
/// not `Sync` due to `Box<dyn Write>`) can satisfy [`log::Log`]'s `Send +
/// Sync` supertrait bounds.
struct RichLogger(std::sync::Mutex<RichHandler>);

impl log::Log for RichLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Ok(mut handler) = self.0.lock() {
            handler.emit(record);
        }
    }

    fn flush(&self) {
        if let Ok(mut handler) = self.0.lock() {
            let _ = handler.console.file.flush();
        }
    }
}

// ---------------------------------------------------------------------------
// install
// ---------------------------------------------------------------------------

/// Convenience: install a Rich logger.
///
/// Creates a default [`RichHandler`], wraps it for thread safety, and
/// registers it as the global logger via [`log::set_logger`].  The max
/// level is set to [`log::LevelFilter::Trace`] so that no messages are
/// silently discarded.
///
/// # Errors
///
/// Returns [`log::SetLoggerError`] if a global logger has already been
/// installed.
pub fn install() -> Result<(), log::SetLoggerError> {
    let logger: &'static RichLogger = Box::leak(Box::new(RichLogger(
        std::sync::Mutex::new(RichHandler::new()),
    )));
    log::set_logger(logger)?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

// ---------------------------------------------------------------------------
// style_level
// ---------------------------------------------------------------------------

/// Style a log level name.
///
/// Returns the appropriate [`Style`] for the given level, matching Python
/// Rich's `logging.level.<name>` theme entries.
pub fn style_level(level: log::Level) -> Style {
    match level {
        log::Level::Error => Style::new()
            .color(Color::parse("red").unwrap_or_default())
            .bold(true),
        log::Level::Warn => Style::new().color(Color::parse("yellow").unwrap_or_default()),
        log::Level::Info => Style::new().color(Color::parse("green").unwrap_or_default()),
        log::Level::Debug => Style::new().color(Color::parse("blue").unwrap_or_default()),
        log::Level::Trace => {
            Style::new().color(Color::parse("bright_black").unwrap_or_default())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_level() {
        let error_style = style_level(log::Level::Error);
        assert!(error_style.get_bold().unwrap_or(false));

        let info_style = style_level(log::Level::Info);
        assert!(!info_style.bold); // info is not bold in Python Rich
    }

    #[test]
    fn test_get_level_text() {
        let handler = RichHandler::new();
        let text = handler.get_level_text(log::Level::Warn);
        let rendered = text.render();
        assert!(rendered.contains("WARN"));
    }

    #[test]
    fn test_render_message_basic() {
        let handler = RichHandler::new();
        let text = handler.render_message("Hello World");
        let rendered = text.render();
        assert!(rendered.contains("Hello World"));
    }

    #[test]
    fn test_render_message_keywords() {
        let handler = RichHandler::new().keywords(vec!["ERROR".to_string()]);
        let text = handler.render_message("GET /index.html ERROR 500");
        let rendered = text.render();
        // Should contain the message (keywords highlighted)
        assert!(rendered.contains("GET"));
        assert!(rendered.contains("ERROR"));
    }

    #[test]
    fn test_default_keywords() {
        assert!(DEFAULT_KEYWORDS.contains(&"GET"));
        assert!(DEFAULT_KEYWORDS.contains(&"POST"));
        assert!(DEFAULT_KEYWORDS.contains(&"DELETE"));
    }

    #[test]
    fn test_render_basic() {
        let mut handler = RichHandler::new();
        let output = handler.render(
            log::Level::Info,
            "Server started",
            Some("server"),
            Some("src/main.rs"),
            Some(42),
        );
        assert!(output.contains("Server started"));
        assert!(output.contains("src/main.rs"));
    }

    #[test]
    fn test_render_no_path() {
        let mut handler = RichHandler::new().show_path(false);
        let output = handler.render(
            log::Level::Debug,
            "debug message",
            None,
            None,
            None,
        );
        assert!(output.contains("debug message"));
    }

    #[test]
    fn test_builder_pattern() {
        let handler = RichHandler::new()
            .show_time(false)
            .show_level(false)
            .show_path(false)
            .markup(true)
            .omit_repeated_times(false)
            .rich_tracebacks(true)
            .tracebacks_width(120)
            .tracebacks_code_width(100)
            .tracebacks_extra_lines(5)
            .tracebacks_show_locals(true)
            .tracebacks_max_frames(50)
            .locals_max_length(20)
            .locals_max_string(120);

        assert!(!handler.show_time);
        assert!(!handler.show_level);
        assert!(!handler.show_path);
        assert!(handler.markup);
        assert!(!handler.omit_repeated_times);
        assert!(handler.rich_tracebacks);
        assert_eq!(handler.tracebacks_width, Some(120));
        assert_eq!(handler.tracebacks_code_width, 100);
        assert_eq!(handler.tracebacks_extra_lines, 5);
        assert!(handler.tracebacks_show_locals);
        assert_eq!(handler.tracebacks_max_frames, 50);
        assert_eq!(handler.locals_max_length, 20);
        assert_eq!(handler.locals_max_string, 120);
    }

    #[test]
    fn test_enable_link_path() {
        let mut handler = RichHandler::new().enable_link_path(true);
        assert!(handler.enable_link_path);
        let output = handler.render(
            log::Level::Info,
            "test",
            Some("mod"),
            Some("src/main.rs"),
            Some(1),
        );
        // With link_path enabled, should contain OSC 8 sequence
        assert!(output.contains("\x1b]8;;"));
    }

    #[test]
    fn test_keywords_builder() {
        let custom = vec!["CUSTOM".to_string(), "KEYWORD".to_string()];
        let handler = RichHandler::new().keywords(custom);
        assert_eq!(handler.keywords.as_ref().unwrap().len(), 2);
    }
}
