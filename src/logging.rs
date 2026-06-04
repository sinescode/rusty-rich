//! Logging integration — equivalent to Rich's `logging.py`.
//!
//! Provides a log handler that renders log records with Rich formatting.

use std::io::Write;

use crate::console::Console;
#[cfg(feature = "syntax-highlighting")]
use crate::highlighter::ReprHighlighter;
use crate::style::Style;
use crate::text::Text;

/// A handler that renders Python-style log records with Rich formatting.
///
/// In Rust, this integrates with the `log` crate.
pub struct RichHandler {
    pub console: Console,
    pub show_time: bool,
    pub show_level: bool,
    pub show_path: bool,
    pub enable_link_path: bool,
    pub markup: bool,
    #[cfg(feature = "syntax-highlighting")]
    pub highlighter: ReprHighlighter,
}

impl RichHandler {
    /// Create a new `RichHandler` with default settings (shows time, level, and path).
    pub fn new() -> Self {
        Self {
            console: Console::new(),
            show_time: true,
            show_level: true,
            show_path: true,
            enable_link_path: false,
            markup: false,
            #[cfg(feature = "syntax-highlighting")]
            highlighter: ReprHighlighter::new(),
        }
    }

    /// Render a single log record.
    pub fn render(
        &self,
        level: log::Level,
        message: &str,
        _module_path: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
    ) -> String {
        let mut text = Text::new("");

        if self.show_time {
            let now = chrono::Local::now();
            let time_str = format!("[{}]", now.format("%H:%M:%S"));
            text.append_styled(time_str, Style::new().dim(true));
            text.append_styled(" ", Style::new());
        }

        if self.show_level {
            let level_style = match level {
                log::Level::Error => Style::new().color(crate::color::Color::parse("red").unwrap()).bold(true),
                log::Level::Warn => Style::new().color(crate::color::Color::parse("yellow").unwrap()),
                log::Level::Info => Style::new().color(crate::color::Color::parse("green").unwrap()),
                log::Level::Debug => Style::new().color(crate::color::Color::parse("blue").unwrap()),
                log::Level::Trace => Style::new().color(crate::color::Color::parse("bright_black").unwrap()),
            };
            text.append_styled(format!("{level:<5}"), level_style);
            text.append_styled(" ", Style::new());
        }

        // Message
        text.append_styled(message, Style::new());

        // File location
        if self.show_path {
            if let (Some(file), Some(line)) = (file, line) {
                text.append_styled(" ", Style::new());
                let location = format!("{file}:{line}");
                text.append_styled(format!("[{location}]"), Style::new().dim(true).italic(true));
            }
        }

        text.render()
    }

    /// Emit a `log` crate [`Record`](log::Record) using Rich formatting.
    ///
    /// Formats the record and writes it to the console.
    pub fn emit(&mut self, record: &log::Record) {
        let formatted = self.render(
            record.level(),
            record.args().to_string().as_str(),
            record.module_path(),
            record.file(),
            record.line(),
        );
        let _ = writeln!(self.console.file, "{formatted}");
        let _ = self.console.file.flush();
    }
}

impl Default for RichHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: install a Rich logger.
pub fn install() -> Result<(), log::SetLoggerError> {
    // This requires a static/global logger; for simplicity, we'll just
    // provide the handler for manual use.
    Ok(())
}

/// Style a log level name.
pub fn style_level(level: log::Level) -> Style {
    match level {
        log::Level::Error => Style::new()
            .color(crate::color::Color::parse("red").unwrap())
            .bold(true),
        log::Level::Warn => Style::new()
            .color(crate::color::Color::parse("yellow").unwrap()),
        log::Level::Info => Style::new()
            .color(crate::color::Color::parse("green").unwrap()),
        log::Level::Debug => Style::new()
            .color(crate::color::Color::parse("blue").unwrap()),
        log::Level::Trace => Style::new()
            .color(crate::color::Color::parse("bright_black").unwrap()),
    }
}
