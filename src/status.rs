//! Status — status message with spinner. Equivalent to Rich's `status.py`.

use std::io::{self, Write};
use std::time::Instant;

use crate::spinner::Spinner;

/// A status message rendered with an animated spinner.
///
/// Usage:
/// ```ignore
/// let status = Status::new("Working...");
/// status.start();
/// // do work...
/// status.update("Still working...");
/// status.stop();
/// ```
pub struct Status {
    pub spinner: Spinner,
    pub status: String,
    pub started: Option<Instant>,
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
        }
    }

    /// Builder: replace the default spinner with a custom [`Spinner`].
    pub fn spinner(mut self, spinner: Spinner) -> Self {
        self.spinner = spinner;
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
        let elapsed = self.started.map(|s| s.elapsed()).unwrap_or_default();
        let spinner_str = self.spinner.render(elapsed);
        write!(io::stdout(), "\r{spinner_str} {}", self.status)?;
        io::stdout().flush()
    }
}
