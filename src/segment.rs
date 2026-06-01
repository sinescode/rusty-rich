//! Segment — styled text unit. Equivalent to Rich's `segment.py`.
//!
//! A [`Segment`] is the smallest unit of output: a piece of text with an
//! associated [`Style`](crate::Style) and optional control code.
//!
//! # Core Types
//!
//! - [`Segment`] — text + optional style + optional control code
//! - [`Segments`] — a collection of segments with convenience methods
//! - `ControlType` — 16 terminal control codes (bell, cursor movement, etc.)
//! - `ControlCode` — a control type with optional parameters
//!
//! # Utility Functions (v0.2)
//!
//! | Function | Description |
//! |----------|-------------|
//! | `Segments::simplify` | Combine adjacent segments with identical styles |
//! | `split_lines` | Split segments into lines at newline boundaries |
//! | `strip_styles` | Remove all styling, returning plain text |
//! | `strip_links` | Remove link IDs and URLs from segment styles |
//! | `align_top` / `align_middle` / `align_bottom` | Vertical alignment helpers |
//! | `divide` | Split segments at given cell offsets |
//! | `set_shape` | Pad or truncate segments to exact width × height |
//! | `filter_control` | Keep only control segments (or only non-control) |
//! | `get_line_length` | Total cell width of a line of segments |
//!
//! # Example
//!
//! ```rust
//! use rusty_rich::{Segment, Segments, Style};
//!
//! let segs = Segments::from(vec![
//!     Segment::styled("Hello ", Style::new().bold(true)),
//!     Segment::styled("World", Style::new().bold(true)),
//! ]);
//!
//! // Combine adjacent same-styled segments
//! let merged = segs.simplify();
//! assert_eq!(merged.segments.len(), 1);
//! ```

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

// ---------------------------------------------------------------------------
// Segment collection utilities
// ---------------------------------------------------------------------------

impl Segments {
    /// Combine adjacent segments that have the same style.
    pub fn simplify(&self) -> Segments {
        let mut result: Vec<Segment> = Vec::new();
        for seg in &self.segments {
            if let Some(last) = result.last_mut() {
                if last.style == seg.style && last.control.is_none() && seg.control.is_none() {
                    last.text.push_str(&seg.text);
                    continue;
                }
            }
            result.push(seg.clone());
        }
        Segments { segments: result }
    }
}

/// Split an iterable of segments into lines at newline boundaries.
pub fn split_lines(segments: &[Segment]) -> Vec<Vec<Segment>> {
    let mut lines: Vec<Vec<Segment>> = Vec::new();
    let mut current: Vec<Segment> = Vec::new();
    for seg in segments {
        if seg.text == "\n" && seg.style.is_none() && seg.control.is_none() {
            lines.push(std::mem::take(&mut current));
        } else if seg.text.contains('\n') && seg.style.is_none() && seg.control.is_none() {
            let parts: Vec<&str> = seg.text.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    lines.push(std::mem::take(&mut current));
                }
                if !part.is_empty() {
                    current.push(Segment::new(*part));
                }
            }
        } else {
            current.push(seg.clone());
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Remove all styles from segments, returning plain text only.
pub fn strip_styles(segments: &[Segment]) -> String {
    let mut out = String::new();
    for seg in segments {
        if seg.control.is_none() {
            out.push_str(&seg.text);
        }
    }
    out
}

/// Remove link IDs and URLs from all segment styles.
pub fn strip_links(segments: &[Segment]) -> Vec<Segment> {
    segments
        .iter()
        .map(|seg| {
            let mut s = seg.clone();
            if let Some(ref style) = seg.style {
                let mut new_style = style.clone();
                new_style.link_id = 0;
                new_style.link = None;
                s.style = Some(new_style);
            }
            s
        })
        .collect()
}

/// Align lines to the top of a region of given height.
pub fn align_top(
    lines: &[Vec<Segment>],
    _width: usize,
    height: usize,
    _style: Option<&Style>,
) -> Vec<Vec<Segment>> {
    let blank_line = vec![Segment::new(" ".repeat(_width))];
    let mut result: Vec<Vec<Segment>> = lines.to_vec();
    while result.len() < height {
        result.push(blank_line.clone());
    }
    result.truncate(height);
    result
}

/// Align lines to the middle of a region of given height.
pub fn align_middle(
    lines: &[Vec<Segment>],
    _width: usize,
    height: usize,
    _style: Option<&Style>,
) -> Vec<Vec<Segment>> {
    let blank_line = vec![Segment::new(" ".repeat(_width))];
    let top_pad = (height.saturating_sub(lines.len())) / 2;
    let mut result: Vec<Vec<Segment>> = Vec::new();
    for _ in 0..top_pad {
        result.push(blank_line.clone());
    }
    result.extend(lines.iter().cloned());
    while result.len() < height {
        result.push(blank_line.clone());
    }
    result.truncate(height);
    result
}

/// Align lines to the bottom of a region of given height.
pub fn align_bottom(
    lines: &[Vec<Segment>],
    _width: usize,
    height: usize,
    _style: Option<&Style>,
) -> Vec<Vec<Segment>> {
    let blank_line = vec![Segment::new(" ".repeat(_width))];
    let bottom_pad = height.saturating_sub(lines.len());
    let mut result: Vec<Vec<Segment>> = Vec::new();
    for _ in 0..bottom_pad {
        result.push(blank_line.clone());
    }
    result.extend(lines.iter().cloned());
    result.truncate(height);
    result
}

/// Divide segments at the given cell offsets.
pub fn divide(segments: &[Segment], cuts: &[usize]) -> Vec<Vec<Segment>> {
    let mut result: Vec<Vec<Segment>> = Vec::new();
    let mut remaining = segments.to_vec();
    let mut offset = 0usize;

    for &cut in cuts {
        let mut chunk: Vec<Segment> = Vec::new();
        let target = cut.saturating_sub(offset);

        let mut chunk_cells = 0usize;
        while chunk_cells < target && !remaining.is_empty() {
            let seg = remaining.remove(0);
            let seg_len = seg.cell_length();
            if chunk_cells + seg_len <= target {
                chunk_cells += seg_len;
                chunk.push(seg);
            } else {
                let split_at = target - chunk_cells;
                let (left, right) = seg.split(split_at);
                chunk.push(left);
                if let Some(r) = right {
                    remaining.insert(0, r);
                }
                chunk_cells = target;
            }
        }
        result.push(chunk);
        offset = cut;
    }

    if !remaining.is_empty() {
        result.push(remaining);
    }

    result
}

/// Set segments to an exact width and height, padding/truncating as needed.
pub fn set_shape(
    lines: &[Vec<Segment>],
    width: usize,
    height: usize,
    _style: Option<&Style>,
) -> Vec<Vec<Segment>> {
    let blank_line = vec![Segment::new(" ".repeat(width))];
    let mut result: Vec<Vec<Segment>> = Vec::new();

    for line in lines.iter().take(height) {
        let cell_len: usize = line.iter().map(|s| s.cell_length()).sum();
        let mut new_line = line.clone();
        if cell_len < width {
            new_line.push(Segment::new(" ".repeat(width - cell_len)));
        } else if cell_len > width {
            let mut truncated = Vec::new();
            let mut count = 0usize;
            for seg in line {
                let seg_len = seg.cell_length();
                if count + seg_len <= width {
                    truncated.push(seg.clone());
                    count += seg_len;
                } else if count < width {
                    let (left, _) = seg.split(width - count);
                    truncated.push(left);
                    break;
                }
            }
            new_line = truncated;
        }
        result.push(new_line);
    }

    while result.len() < height {
        result.push(blank_line.clone());
    }

    result
}

/// Filter segments, keeping only control codes if `is_control` is true,
/// or only non-control segments if `is_control` is false.
pub fn filter_control(segments: &[Segment], is_control: bool) -> Vec<Segment> {
    segments
        .iter()
        .filter(|seg| seg.control.is_some() == is_control)
        .cloned()
        .collect()
}

/// Get the total cell length of a line of segments.
pub fn get_line_length(line: &[Segment]) -> usize {
    line.iter().map(|s| s.cell_length()).sum()
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
