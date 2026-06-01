//! Segment — styled text unit. Equivalent to Rich's `segment.py`.
//!
//! A `Segment` is the smallest unit of output: a piece of text with an
//! associated style and optional control codes.

use std::fmt;

use crate::style::Style;

// ---------------------------------------------------------------------------
// ControlType
// ---------------------------------------------------------------------------

/// Non-printable control codes (equivalent to Rich's `ControlType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlType {
    Bell,
    CarriageReturn,
    Home,
    Clear,
    ShowCursor,
    HideCursor,
    EnableAltScreen,
    DisableAltScreen,
    CursorUp,
    CursorDown,
    CursorForward,
    CursorBackward,
    CursorMoveToColumn,
    CursorMoveTo,
    EraseInLine,
    SetWindowTitle,
}

impl ControlType {
    /// Get the ANSI escape sequence for this control type.
    pub fn to_ansi(&self, params: &[i32]) -> String {
        match self {
            Self::Bell => "\x07".into(),
            Self::CarriageReturn => "\r".into(),
            Self::Home => "\x1b[H".into(),
            Self::Clear => "\x1b[2J".into(),
            Self::ShowCursor => "\x1b[?25h".into(),
            Self::HideCursor => "\x1b[?25l".into(),
            Self::EnableAltScreen => "\x1b[?1049h".into(),
            Self::DisableAltScreen => "\x1b[?1049l".into(),
            Self::CursorUp => {
                let n = params.first().copied().unwrap_or(1);
                format!("\x1b[{n}A")
            }
            Self::CursorDown => {
                let n = params.first().copied().unwrap_or(1);
                format!("\x1b[{n}B")
            }
            Self::CursorForward => {
                let n = params.first().copied().unwrap_or(1);
                format!("\x1b[{n}C")
            }
            Self::CursorBackward => {
                let n = params.first().copied().unwrap_or(1);
                format!("\x1b[{n}D")
            }
            Self::CursorMoveToColumn => {
                let col = params.first().copied().unwrap_or(0);
                format!("\x1b[{col}G")
            }
            Self::CursorMoveTo => {
                let row = params.first().copied().unwrap_or(0);
                let col = params.get(1).copied().unwrap_or(0);
                format!("\x1b[{row};{col}H")
            }
            Self::EraseInLine => {
                let mode = params.first().copied().unwrap_or(0);
                format!("\x1b[{mode}K")
            }
            Self::SetWindowTitle => {
                let title: String = params
                    .iter()
                    .map(|n| char::from(*n as u8))
                    .collect();
                format!("\x1b]0;{title}\x07")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ControlCode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlCode {
    Simple(ControlType),
    WithInt(ControlType, i32),
    WithTwoInts(ControlType, i32, i32),
    WithString(ControlType, String),
}

// ---------------------------------------------------------------------------
// Segment
// ---------------------------------------------------------------------------

/// A piece of text with an associated style.
///
/// Segments are produced during rendering and ultimately converted to strings
/// for terminal output.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub text: String,
    pub style: Option<Style>,
    pub control: Option<ControlCode>,
}

impl Segment {
    /// Create a new segment with just text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: None,
            control: None,
        }
    }

    /// Create a segment with text and style.
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style: Some(style),
            control: None,
        }
    }

    /// Create a control-only segment.
    pub fn control(code: ControlCode) -> Self {
        Self {
            text: String::new(),
            style: None,
            control: Some(code),
        }
    }

    /// Create a newline segment.
    pub fn line() -> Self {
        Self::new("\n")
    }

    /// Get the cell length of this segment's text (using Unicode width).
    pub fn cell_length(&self) -> usize {
        if self.control.is_some() {
            return 0;
        }
        unicode_width::UnicodeWidthStr::width(self.text.as_str())
    }

    /// Check if this segment has any text content.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() && self.control.is_none()
    }

    /// Split this segment at the given offset, returning two segments.
    /// The first goes up to (but not including) `offset`, the second from
    /// `offset` to end.
    pub fn split(&self, offset: usize) -> (Segment, Option<Segment>) {
        if offset == 0 {
            return (Segment::new(""), Some(self.clone()));
        }
        let cell_len = self.cell_length();
        if offset >= cell_len {
            return (self.clone(), None);
        }

        // Find the byte position at which the cell count reaches `offset`
        let mut cell_count = 0usize;
        let mut byte_pos = 0usize;
        for (i, ch) in self.text.char_indices() {
            let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            if cell_count + w > offset {
                break;
            }
            cell_count += w;
            byte_pos = i + ch.len_utf8();
        }

        let left = Segment {
            text: self.text[..byte_pos].to_string(),
            style: self.style.clone(),
            control: self.control.clone(),
        };
        let right = Segment {
            text: self.text[byte_pos..].to_string(),
            style: self.style.clone(),
            control: self.control.clone(),
        };
        (left, Some(right))
    }

    /// Return the ANSI string for this segment (style + text + reset).
    pub fn to_ansi(&self) -> String {
        if let Some(ref code) = self.control {
            return match code {
                ControlCode::Simple(ct) => ct.to_ansi(&[]),
                ControlCode::WithInt(ct, a) => ct.to_ansi(&[*a]),
                ControlCode::WithTwoInts(ct, a, b) => ct.to_ansi(&[*a, *b]),
                ControlCode::WithString(ct, s) => {
                    let params: Vec<i32> = s.bytes().map(|b| b as i32).collect();
                    ct.to_ansi(&params)
                }
            };
        }

        let style_ansi = self.style.as_ref().map(|s| s.to_ansi()).unwrap_or_default();
        let reset = self.style.as_ref().map(|s| s.reset_ansi()).unwrap_or("");

        if style_ansi.is_empty() {
            self.text.clone()
        } else {
            format!("{style_ansi}{}{reset}", self.text)
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}

// ---------------------------------------------------------------------------
// Segments — a collection of segments
// ---------------------------------------------------------------------------

/// A collection of `Segment`s, with convenience methods.
#[derive(Debug, Clone, Default)]
pub struct Segments {
    pub segments: Vec<Segment>,
}

impl Segments {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, seg: Segment) {
        self.segments.push(seg);
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = Segment>) {
        self.segments.extend(other);
    }

    /// Render all segments to an ANSI string.
    pub fn to_ansi(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            out.push_str(&seg.to_ansi());
        }
        out
    }

    /// Get the total cell width.
    pub fn cell_len(&self) -> usize {
        self.segments.iter().map(Segment::cell_length).sum()
    }
}

impl fmt::Display for Segments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}

impl From<Vec<Segment>> for Segments {
    fn from(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
}

impl IntoIterator for Segments {
    type Item = Segment;
    type IntoIter = std::vec::IntoIter<Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}

// ---------------------------------------------------------------------------
// Utility: line()
// ---------------------------------------------------------------------------

/// Helper: create a newline segment.
pub fn line() -> Segment {
    Segment::line()
}

/// Helper: create a space segment.
pub fn space(count: usize) -> Segment {
    Segment::new(" ".repeat(count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    #[test]
    fn test_segment_cell_length() {
        let seg = Segment::new("Hello");
        assert_eq!(seg.cell_length(), 5);
    }

    #[test]
    fn test_segment_split() {
        let seg = Segment::new("Hello World");
        let (left, right) = seg.split(5);
        assert_eq!(left.text, "Hello");
        assert_eq!(right.unwrap().text, " World");
    }

    #[test]
    fn test_segment_to_ansi() {
        let style = Style::new().bold(true);
        let seg = Segment::styled("Bold", style);
        let ansi = seg.to_ansi();
        assert!(ansi.contains("[1m"));
        assert!(ansi.contains("Bold"));
    }
}
