//! Status — status message with spinner. Equivalent to Rich's `status.py`.

use std::io::{self, Write};
use std::time::Instant;

use crate::spinner::Spinner;
use crate::style::Style;

/// A status message rendered with an animated spinner.
///
/// Usage:
/// ```rust,no_run
/// # use rusty_rich::Status;
/// let mut status = Status::new("Working...");
/// status.start();
/// // do work...
/// status.update("Still working...");
/// status.stop();
/// ```
pub struct Status {
    pub spinner: Spinner,
    pub status: String,
    pub started: Option<Instant>,
    /// Style applied to the spinner character.
    pub spinner_style: Style,
    /// Style applied to the status text.
    pub status_style: Style,
    /// Animation speed multiplier (1.0 = default speed).
    pub speed: f64,
}

impl std::fmt::Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Status")
            .field("status", &self.status)
            .field("started", &self.started)
            .finish()
    }
}

impl Status {
    /// Create a new Status with the given message.
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            spinner: Spinner::default(),
            status: status.into(),
            started: None,
            spinner_style: Style::new(),
            status_style: Style::new(),
            speed: 1.0,
        }
    }

    /// Builder: replace the default spinner with a custom [`Spinner`].
    pub fn spinner(mut self, spinner: Spinner) -> Self {
        self.spinner = spinner;
        self
    }

    /// Builder: set the style for the spinner character.
    pub fn spinner_style(mut self, style: Style) -> Self {
        self.spinner_style = style;
        self
    }

    /// Builder: set the style for the status message text.
    pub fn status_style(mut self, style: Style) -> Self {
        self.status_style = style;
        self
    }

    /// Builder: set the animation speed multiplier.
    ///
    /// Values less than 1.0 slow down the animation; values greater than 1.0
    /// speed it up. Default is 1.0.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Start displaying the status.
    pub fn start(&mut self) -> io::Result<()> {
        self.started = Some(Instant::now());
        self.write_status()
    }

    /// Update the status message.
    pub fn update(&mut self, status: impl Into<String>) -> io::Result<()> {
        self.status = status.into();
        self.write_status()
    }

    /// Stop the status display (clears the line).
    pub fn stop(&mut self) -> io::Result<()> {
        // Carriage return + clear line
        let _ = write!(io::stdout(), "\r\x1b[K");
        let _ = io::stdout().flush();
        self.started = None;
        Ok(())
    }

    /// Refresh the display.
    pub fn refresh(&mut self) -> io::Result<()> {
        self.write_status()
    }

    fn write_status(&mut self) -> io::Result<()> {
        let elapsed = self
            .started
            .map(|s| s.elapsed())
            .unwrap_or_default();
        let scaled_elapsed = elapsed.mul_f64(self.speed);
        let spinner_str = self.spinner.render(scaled_elapsed);

        // Apply spinner style if set
        let styled_spinner = if self.spinner_style.is_plain() {
            spinner_str.to_string()
        } else {
            format!(
                "{}{}\x1b[0m",
                self.spinner_style.to_ansi(),
                spinner_str
            )
        };

        // Apply status style if set
        let styled_status = if self.status_style.is_plain() {
            self.status.clone()
        } else {
            format!(
                "{}{}\x1b[0m",
                self.status_style.to_ansi(),
                self.status
            )
        };

        write!(io::stdout(), "\r{styled_spinner} {styled_status}")?;
        io::stdout().flush()
    }
}

/// When a [`Status`] is dropped while still running, it automatically stops
/// the display and cleans up the terminal line.
impl Drop for Status {
    fn drop(&mut self) {
        if self.started.is_some() {
            let _ = self.stop();
        }
    }
}
