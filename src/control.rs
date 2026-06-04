//! Terminal control sequence generation — equivalent to Rich's `control.py`.
//!
//! Provides a unified [`Control`] type for composing terminal escape sequences:
//! cursor movement, screen manipulation, window titles, and bells. Individual
//! control codes can be combined and rendered as ANSI escape sequences.
//!
//! # Quick Example
//!
//! ```rust
//! use rusty_rich::control::{Control, control_bell, control_home};
//!
//! let bell = control_bell();
//! assert_eq!(bell.to_ansi(), "\x07");
//!
//! let combined = Control::new(&["\x1b[H", "\x1b[2J"]);
//! ```

// ---------------------------------------------------------------------------
// ANSI escape sequence constants (zero-allocation, for hot-path use)
// ---------------------------------------------------------------------------

/// Clear entire screen and move cursor to home: `\x1b[2J\x1b[H`
pub const CLEAR_HOME: &str = "\x1b[2J\x1b[H";
/// Clear entire screen: `\x1b[2J`
pub const CLEAR_SCREEN: &str = "\x1b[2J";
/// Move cursor to home: `\x1b[H`
pub const CURSOR_HOME: &str = "\x1b[H";
/// Show cursor: `\x1b[?25h`
pub const CURSOR_SHOW: &str = "\x1b[?25h";
/// Hide cursor: `\x1b[?25l`
pub const CURSOR_HIDE: &str = "\x1b[?25l";
/// Enter alternate screen buffer: `\x1b[?1049h`
pub const ALT_SCREEN_ENTER: &str = "\x1b[?1049h";
/// Exit alternate screen buffer: `\x1b[?1049l`
pub const ALT_SCREEN_EXIT: &str = "\x1b[?1049l";
/// Erase the current line: `\x1b[2K`
pub const ERASE_LINE: &str = "\x1b[2K";
/// Move cursor up one row: `\x1b[1A`
pub const CURSOR_UP: &str = "\x1b[1A";
/// Carriage return: `\r`
pub const CARRIAGE_RETURN: &str = "\r";
/// Newline: `\n`
pub const NEWLINE: &str = "\n";
/// Operating System Command (OSC) introducer: `\x1b]`
pub const OSC: &str = "\x1b]";
/// String Terminator (ST) for OSC sequences: `\x07`
pub const ST: &str = "\x07";

// ---------------------------------------------------------------------------
// Control — composable control sequence
// ---------------------------------------------------------------------------

/// A composable terminal control sequence.
///
/// A `Control` holds one or more ANSI escape sequences and can render them
/// as raw ANSI bytes or as a [`crate::segment::Segment`] with control codes.
///
/// This is the Rust equivalent of Python Rich's `Control` class.
///
/// # Examples
///
/// ```rust
/// use rusty_rich::control::{Control, control_home, control_clear};
///
/// // Named constructors
/// let home = control_home();
///
/// // Build combined controls
/// let clear_and_home = Control::new(&["\x1b[2J", "\x1b[H"]);
///
/// // Cursor positioning
/// let go_to = Control::cursor_to(10, 5);
/// ```
#[derive(Debug, Clone)]
pub struct Control {
    sequences: Vec<String>,
}

impl Control {
    /// Create a `Control` from a slice of raw ANSI escape sequences.
    pub fn new(sequences: &[&str]) -> Self {
        Self {
            sequences: sequences.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Ring the terminal bell.
    pub fn bell() -> Self {
        Self::new(&["\x07"])
    }

    /// Move cursor to home position.
    pub fn home() -> Self {
        Self::new(&["\x1b[H"])
    }

    /// Clear the entire screen.
    pub fn clear() -> Self {
        Self::new(&["\x1b[2J"])
    }

    /// Clear screen and move cursor to home.
    pub fn clear_home() -> Self {
        Self::new(&["\x1b[2J", "\x1b[H"])
    }

    /// Move cursor up by `n` rows.
    pub fn cursor_up(n: u16) -> Self {
        Self::new(&[&format!("\x1b[{n}A")])
    }

    /// Move cursor down by `n` rows.
    pub fn cursor_down(n: u16) -> Self {
        Self::new(&[&format!("\x1b[{n}B")])
    }

    /// Move cursor forward by `n` columns.
    pub fn cursor_forward(n: u16) -> Self {
        Self::new(&[&format!("\x1b[{n}C")])
    }

    /// Move cursor back by `n` columns.
    pub fn cursor_back(n: u16) -> Self {
        Self::new(&[&format!("\x1b[{n}D")])
    }

    /// Move cursor to an absolute position (1-based row, column).
    pub fn cursor_to(row: u16, col: u16) -> Self {
        Self::new(&[&format!("\x1b[{row};{col}H")])
    }

    /// Move cursor to a specific row (1-based).
    pub fn cursor_to_row(row: u16) -> Self {
        Self::new(&[&format!("\x1b[{row}d")])
    }

    /// Move cursor to a specific column (1-based).
    pub fn cursor_to_column(col: u16) -> Self {
        Self::new(&[&format!("\x1b[{col}G")])
    }

    /// Enable or disable the alternate screen buffer.
    pub fn alt_screen(enable: bool) -> Self {
        if enable {
            Self::new(&["\x1b[?1049h"])
        } else {
            Self::new(&["\x1b[?1049l"])
        }
    }

    /// Show or hide the cursor.
    pub fn show_cursor(show: bool) -> Self {
        if show {
            Self::new(&["\x1b[?25h"])
        } else {
            Self::new(&["\x1b[?25l"])
        }
    }

    /// Set the terminal window title.
    pub fn title(title: impl Into<String>) -> Self {
        let t: String = title.into();
        Self::new(&[&format!("\x1b]0;{t}\x07")])
    }

    /// Erase from cursor to end of line.
    pub fn erase_end_line() -> Self {
        Self::new(&["\x1b[K"])
    }

    /// Erase from cursor to beginning of line.
    pub fn erase_start_line() -> Self {
        Self::new(&["\x1b[1K"])
    }

    /// Erase the entire current line.
    pub fn erase_line() -> Self {
        Self::new(&["\x1b[2K"])
    }

    /// Erase from cursor to end of screen.
    pub fn erase_end_screen() -> Self {
        Self::new(&["\x1b[J"])
    }

    /// Erase from cursor to beginning of screen.
    pub fn erase_start_screen() -> Self {
        Self::new(&["\x1b[1J"])
    }

    /// Insert `n` blank lines at the cursor position.
    pub fn insert_lines(n: u16) -> Self {
        Self::new(&[&format!("\x1b[{n}L")])
    }

    /// Delete `n` lines at the cursor position.
    pub fn delete_lines(n: u16) -> Self {
        Self::new(&[&format!("\x1b[{n}M")])
    }

    /// Carriage return.
    pub fn carriage_return() -> Self {
        Self::new(&["\r"])
    }

    /// Newline.
    pub fn newline() -> Self {
        Self::new(&["\n"])
    }

    /// Convert this `Control` to a single ANSI escape sequence string.
    pub fn to_ansi(&self) -> String {
        self.sequences.concat()
    }

    /// Return the individual ANSI sequences.
    pub fn sequences(&self) -> &[String] {
        &self.sequences
    }

    /// Return the number of control sequences in this command.
    pub fn len(&self) -> usize {
        self.sequences.len()
    }

    /// Return `true` if this control has no sequences.
    pub fn is_empty(&self) -> bool {
        self.sequences.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/// Ring the terminal bell.
pub fn control_bell() -> Control { Control::bell() }

/// Move cursor to home position.
pub fn control_home() -> Control { Control::home() }

/// Clear the entire screen.
pub fn control_clear() -> Control { Control::clear() }

/// Move cursor to an absolute position.
pub fn control_move_to(row: u16, col: u16) -> Control { Control::cursor_to(row, col) }

/// Show or hide the cursor.
pub fn control_show_cursor(show: bool) -> Control { Control::show_cursor(show) }

/// Set terminal window title.
pub fn control_title(title: impl Into<String>) -> Control { Control::title(title) }

// ---------------------------------------------------------------------------
// Strip/escape control characters
// ---------------------------------------------------------------------------

/// Strip control characters (bell, backspace, vertical tab, form feed) from
/// a string.
///
/// Equivalent to Python Rich's `strip_control_codes()`.
pub fn strip_control_codes(text: &str) -> String {
    text.chars()
        .filter(|&c| !matches!(c, '\x07' | '\x08' | '\x0b' | '\x0c'))
        .collect()
}

/// Escape control characters in a string, replacing them with visible
/// representations like `\\a`, `\\b`, etc.
pub fn escape_control_codes(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\x07' => "\\a".to_string(),
            '\x08' => "\\b".to_string(),
            '\x0b' => "\\v".to_string(),
            '\x0c' => "\\f".to_string(),
            '\r' => "\\r".to_string(),
            '\n' => "\\n".to_string(),
            '\t' => "\\t".to_string(),
            other => other.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_bell() {
        let c = Control::bell();
        assert_eq!(c.to_ansi(), "\x07");
    }

    #[test]
    fn test_control_home() {
        let c = Control::home();
        assert_eq!(c.to_ansi(), "\x1b[H");
    }

    #[test]
    fn test_control_clear() {
        let c = Control::clear();
        assert_eq!(c.to_ansi(), "\x1b[2J");
    }

    #[test]
    fn test_control_clear_home() {
        let c = Control::clear_home();
        assert_eq!(c.to_ansi(), "\x1b[2J\x1b[H");
    }

    #[test]
    fn test_control_cursor_to() {
        let c = Control::cursor_to(10, 5);
        assert_eq!(c.to_ansi(), "\x1b[10;5H");
    }

    #[test]
    fn test_control_cursor_up() {
        let c = Control::cursor_up(5);
        assert_eq!(c.to_ansi(), "\x1b[5A");
    }

    #[test]
    fn test_control_show_cursor() {
        let c = Control::show_cursor(true);
        assert_eq!(c.to_ansi(), "\x1b[?25h");
        let c = Control::show_cursor(false);
        assert_eq!(c.to_ansi(), "\x1b[?25l");
    }

    #[test]
    fn test_control_alt_screen() {
        let c = Control::alt_screen(true);
        assert_eq!(c.to_ansi(), "\x1b[?1049h");
        let c = Control::alt_screen(false);
        assert_eq!(c.to_ansi(), "\x1b[?1049l");
    }

    #[test]
    fn test_control_title() {
        let c = Control::title("My App");
        assert_eq!(c.to_ansi(), "\x1b]0;My App\x07");
    }

    #[test]
    fn test_control_erase() {
        assert_eq!(Control::erase_line().to_ansi(), "\x1b[2K");
        assert_eq!(Control::erase_end_line().to_ansi(), "\x1b[K");
        assert_eq!(Control::erase_start_line().to_ansi(), "\x1b[1K");
    }

    #[test]
    fn test_control_insert_delete_lines() {
        assert_eq!(Control::insert_lines(3).to_ansi(), "\x1b[3L");
        assert_eq!(Control::delete_lines(2).to_ansi(), "\x1b[2M");
    }

    #[test]
    fn test_control_len_empty() {
        assert_eq!(Control::bell().len(), 1);
        assert_eq!(Control::clear_home().len(), 2);
        assert!(!Control::bell().is_empty());
    }

    #[test]
    fn test_strip_control_codes() {
        assert_eq!(strip_control_codes("hello\x07world"), "helloworld");
        assert_eq!(strip_control_codes("normal text"), "normal text");
    }

    #[test]
    fn test_escape_control_codes() {
        let result = escape_control_codes("\x07\x08\x0b\x0c");
        assert_eq!(result, "\\a\\b\\v\\f");
    }

    #[test]
    fn test_convenience_functions() {
        assert_eq!(control_bell().to_ansi(), "\x07");
        assert_eq!(control_home().to_ansi(), "\x1b[H");
        assert_eq!(control_move_to(3, 8).to_ansi(), "\x1b[3;8H");
    }
}
