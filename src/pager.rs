//! System pager integration — pipes output to `less` or `$PAGER`.
//!
//! Equivalent to Python Rich's `pager.py`. Provides a configurable pager
//! that sends content through an external pager program (e.g. `less`)
//! for scrolling through long output.

use std::io::Write;
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// SystemPager
// ---------------------------------------------------------------------------

/// A pager that uses the system's default pager (`$PAGER` or `less`/`more`).
///
/// The pager command is split into program + arguments to prevent command
/// injection via environment variables (VULN-007).
#[derive(Debug, Clone)]
pub struct SystemPager {
    /// The pager program to execute.
    program: String,
    /// Arguments to pass to the pager program.
    args: Vec<String>,
}

impl SystemPager {
    /// Create a new `SystemPager`, detecting the system pager from the
    /// `PAGER` environment variable. Falls back to `less` on Unix and
    /// `more` on Windows.
    pub fn new() -> Self {
        let pager_cmd = std::env::var("PAGER").unwrap_or_else(|_| default_pager());
        let (program, args) = split_pager_command(&pager_cmd);
        Self { program, args }
    }

    /// Pipe `content` through the system pager.
    ///
    /// Spawns the pager process, writes content to its stdin, and waits
    /// for it to finish.
    pub fn show(&self, content: &str) -> std::io::Result<()> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(content.as_bytes())?;
        }

        // Close stdin explicitly so the pager knows there's no more input
        drop(child.stdin.take());

        child.wait()?;
        Ok(())
    }
}

impl Default for SystemPager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pager
// ---------------------------------------------------------------------------

/// A configurable pager for displaying long output.
///
/// Wraps [`SystemPager`] with options for enabling/disabling paging,
/// setting a custom command, and preserving ANSI color codes.
#[derive(Debug, Clone)]
pub struct Pager {
    /// Whether paging is enabled.
    enabled: bool,
    /// The pager command to use.
    command: String,
    /// Whether to preserve ANSI color codes in paged output.
    color: bool,
}

impl Pager {
    /// Create a new `Pager` with default settings (enabled, uses `$PAGER`,
    /// color enabled). Falls back to `less` on Unix, `more` on Windows.
    pub fn new() -> Self {
        Self {
            enabled: true,
            command: std::env::var("PAGER").unwrap_or_else(|_| default_pager()),
            color: true,
        }
    }

    /// Builder: enable or disable paging.
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }

    /// Builder: set a custom pager command.
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.command = cmd.into();
        self
    }

    /// Builder: enable or disable ANSI color passthrough.
    pub fn color(mut self, value: bool) -> Self {
        self.color = value;
        self
    }

    /// Return `true` if paging is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Return the pager command string.
    pub fn command_str(&self) -> &str {
        &self.command
    }

    /// Return `true` if color preservation is enabled.
    pub fn is_color(&self) -> bool {
        self.color
    }

    /// Show `content` through the pager.
    ///
    /// If paging is disabled, this is a no-op. If color is disabled,
    /// ANSI escape sequences are stripped before sending to the pager.
    pub fn show(&self, content: &str) -> std::io::Result<()> {
        if !self.enabled {
            // Paging disabled — just print to stdout
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(content.as_bytes())?;
            handle.flush()?;
            return Ok(());
        }

        let display = if !self.color {
            // Strip ANSI escape sequences using the comprehensive FSM-based
            // version from the export module (handles CSI, OSC, DCS, etc.)
            crate::export::strip_ansi_escapes(content)
        } else {
            content.to_string()
        };

        let (program, args) = split_pager_command(&self.command);
        let pager = SystemPager { program, args };
        pager.show(&display)
    }
}

impl Default for Pager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PagerContext
// ---------------------------------------------------------------------------

/// A context that accumulates content and pages it on drop.
///
/// Provides RAII-style paging: content is fed via [`feed`](PagerContext::feed)
/// and automatically sent to the pager when the context is dropped.
#[derive(Debug)]
pub struct PagerContext {
    /// The pager configuration.
    pager: Pager,
    /// The accumulated content.
    content: String,
    /// Whether paging is enabled for this context.
    enabled: bool,
}

impl PagerContext {
    /// Create a new `PagerContext` that uses the given [`Pager`].
    pub fn new(pager: Pager) -> Self {
        let enabled = pager.enabled;
        Self {
            pager,
            content: String::new(),
            enabled,
        }
    }

    /// Append text to the content buffer.
    pub fn feed(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /// Flush the accumulated content to the pager immediately,
    /// bypassing the drop handler.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if !self.content.is_empty() {
            let result = self.pager.show(&self.content);
            self.content.clear();
            result
        } else {
            Ok(())
        }
    }

    /// Return a reference to the accumulated content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Return whether paging is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Write for PagerContext {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.feed(&s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for PagerContext {
    fn drop(&mut self) {
        if self.enabled && !self.content.is_empty() {
            let _ = self.pager.show(&self.content);
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Return the platform-appropriate default pager command.
fn default_pager() -> String {
    if cfg!(windows) {
        "more".into()
    } else {
        "less".into()
    }
}

/// Split a pager command string into a program and arguments.
///
/// This prevents command injection via `$PAGER` by ensuring the program
/// and arguments are passed separately to `Command::new()`.
fn split_pager_command(cmd: &str) -> (String, Vec<String>) {
    let mut parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return (String::new(), vec![]);
    }
    let program = parts.remove(0).to_string();
    let args: Vec<String> = parts.into_iter().map(String::from).collect();
    (program, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_pager_creation() {
        let pager = SystemPager::new();
        // Should detect PAGER or default to "less"/"more"
        assert!(!pager.program.is_empty());
    }

    #[test]
    fn test_pager_defaults() {
        let pager = Pager::new();
        assert!(pager.is_enabled());
        assert!(pager.is_color());
        assert!(!pager.command_str().is_empty());
    }

    #[test]
    fn test_pager_builder() {
        let pager = Pager::new()
            .enabled(false)
            .command("more")
            .color(false);
        assert!(!pager.is_enabled());
        assert!(!pager.is_color());
        assert_eq!(pager.command_str(), "more");
    }

    #[test]
    fn test_pager_disabled_show() {
        let pager = Pager::new().enabled(false);
        // When disabled, show() writes to stdout — just verify it returns Ok
        assert!(pager.show("test").is_ok());
    }

    #[test]
    fn test_pager_context_feed() {
        let pager = Pager::new().enabled(false);
        let mut ctx = PagerContext::new(pager);
        ctx.feed("Hello, ");
        ctx.feed("World!");
        assert_eq!(ctx.content(), "Hello, World!");
    }

    #[test]
    fn test_pager_context_write_trait() {
        use std::io::Write;
        let pager = Pager::new().enabled(false);
        let mut ctx = PagerContext::new(pager);
        write!(ctx, "Hello {}!", "World").unwrap();
        assert!(ctx.content().contains("Hello"));
        assert!(ctx.content().contains("World"));
    }

    #[test]
    fn test_strip_ansi_via_export() {
        // Uses crate::export::strip_ansi_escapes (hand-written FSM)
        let input = "\x1b[31mhello\x1b[0m world";
        let result = crate::export::strip_ansi_escapes(input);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_pager_context_flush() {
        let pager = Pager::new().enabled(false);
        let mut ctx = PagerContext::new(pager);
        ctx.feed("test");
        assert!(ctx.flush().is_ok());
        assert!(ctx.content().is_empty());
    }
}
