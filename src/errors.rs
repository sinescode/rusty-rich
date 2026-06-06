//! Error types — equivalent to Rich's `errors.py`.
//!
//! Dedicated error types for the rendering pipeline. These provide clear
//! diagnostic messages for common failure modes in style parsing, markup
//! interpretation, and live display management.

use std::fmt;

// ---------------------------------------------------------------------------
// ConsoleError
// ---------------------------------------------------------------------------

/// Top-level error for console rendering failures.
#[derive(Debug)]
pub struct ConsoleError {
    pub message: String,
}

impl ConsoleError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConsoleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Console error: {}", self.message)
    }
}

impl std::error::Error for ConsoleError {}

// ---------------------------------------------------------------------------
// StyleError
// ---------------------------------------------------------------------------

/// Errors from style parsing or application.
#[derive(Debug, Clone)]
pub enum StyleError {
    /// Style string was badly formatted (e.g. unknown attribute name).
    Syntax(String),
    /// Referenced style does not exist in the theme.
    Missing(String),
    /// Style stack is in an invalid state (underflow, overflow).
    Stack(String),
    /// Unknown attribute in style string.
    UnknownAttribute(String),
}

impl fmt::Display for StyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(msg) => write!(f, "Style syntax error: {msg}"),
            Self::Missing(name) => write!(f, "Missing style: '{name}'"),
            Self::Stack(msg) => write!(f, "Style stack error: {msg}"),
            Self::UnknownAttribute(attr) => write!(f, "Unknown style attribute: '{attr}'"),
        }
    }
}

impl std::error::Error for StyleError {}

// ---------------------------------------------------------------------------
// MarkupError
// ---------------------------------------------------------------------------

/// Markup parsing errors (BBCode-like tags).
#[derive(Debug, Clone)]
pub enum MarkupError {
    /// An opening tag was never closed.
    UnmatchedOpen(String),
    /// A closing tag has no matching opening tag.
    UnmatchedClose(String),
    /// A tag is malformed (e.g. missing `]`).
    MalformedTag(String),
}

impl fmt::Display for MarkupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnmatchedOpen(tag) => write!(f, "Unmatched opening tag: '[{tag}]'"),
            Self::UnmatchedClose(tag) => write!(f, "Unmatched closing tag: '[/{tag}]'"),
            Self::MalformedTag(msg) => write!(f, "Malformed markup tag: {msg}"),
        }
    }
}

impl std::error::Error for MarkupError {}

// ---------------------------------------------------------------------------
// NotRenderableError
// ---------------------------------------------------------------------------

/// Raised when an object cannot be rendered.
#[derive(Debug, Clone)]
pub struct NotRenderableError(pub String);

impl NotRenderableError {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self(type_name.into())
    }
}

impl fmt::Display for NotRenderableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object of type '{}' is not renderable", self.0)
    }
}

impl std::error::Error for NotRenderableError {}

// ---------------------------------------------------------------------------
// LiveError
// ---------------------------------------------------------------------------

/// Live display errors.
#[derive(Debug, Clone)]
pub enum LiveError {
    /// Alternate screen mode was requested but is unavailable.
    NoAltScreen,
    /// Operation requires the live display to be started.
    NotStarted,
    /// Live display is already running.
    AlreadyStarted,
    /// Content exceeds the available region.
    Overflow(String),
}

impl fmt::Display for LiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAltScreen => write!(f, "Alternate screen mode is not available"),
            Self::NotStarted => write!(f, "Live display has not been started"),
            Self::AlreadyStarted => write!(f, "Live display is already running"),
            Self::Overflow(msg) => write!(f, "Live display overflow: {msg}"),
        }
    }
}

impl std::error::Error for LiveError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_error_display() {
        let err = ConsoleError::new("something went wrong");
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn test_style_error_display() {
        let err = StyleError::Syntax("bad format".into());
        assert!(err.to_string().contains("bad format"));

        let err = StyleError::Missing("unknown_style".into());
        assert!(err.to_string().contains("unknown_style"));

        let err = StyleError::Stack("underflow".into());
        assert!(err.to_string().contains("underflow"));
    }

    #[test]
    fn test_markup_error_display() {
        let err = MarkupError::UnmatchedOpen("bold".into());
        assert!(err.to_string().contains("[bold]"));

        let err = MarkupError::UnmatchedClose("red".into());
        assert!(err.to_string().contains("[/red]"));

        let err = MarkupError::MalformedTag("missing bracket".into());
        assert!(err.to_string().contains("missing bracket"));
    }

    #[test]
    fn test_not_renderable_error() {
        let err = NotRenderableError::new("std::net::TcpStream");
        assert!(err.to_string().contains("std::net::TcpStream"));
        assert!(err.to_string().contains("not renderable"));
    }

    #[test]
    fn test_live_error_display() {
        let err = LiveError::NoAltScreen;
        assert!(err.to_string().contains("Alternate screen"));

        let err = LiveError::NotStarted;
        assert!(err.to_string().contains("not been started"));

        let err = LiveError::AlreadyStarted;
        assert!(err.to_string().contains("already running"));

        let err = LiveError::Overflow("too many lines".into());
        assert!(err.to_string().contains("too many lines"));
    }

    #[test]
    fn test_error_trait_impl() {
        // Verify std::error::Error is implemented
        let err: Box<dyn std::error::Error> = Box::new(ConsoleError::new("test"));
        let _ = err.to_string();

        let err: Box<dyn std::error::Error> = Box::new(StyleError::Syntax("x".into()));
        let _ = err.to_string();

        let err: Box<dyn std::error::Error> = Box::new(MarkupError::MalformedTag("x".into()));
        let _ = err.to_string();

        let err: Box<dyn std::error::Error> = Box::new(NotRenderableError::new("T"));
        let _ = err.to_string();

        let err: Box<dyn std::error::Error> = Box::new(LiveError::NotStarted);
        let _ = err.to_string();
    }
}
