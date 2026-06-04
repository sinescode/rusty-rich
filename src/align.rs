//! Text alignment — equivalent to Rich's `align.py`.
//!
//! Provides horizontal and vertical alignment for renderables.

use std::fmt;

// ---------------------------------------------------------------------------
// AlignMethod
// ---------------------------------------------------------------------------

/// Horizontal alignment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignMethod {
    /// Left-align (the default).
    Left,
    /// Center-align.
    Center,
    /// Right-align.
    Right,
    /// Full justification (spaces distributed between words).
    Full,
}

impl AlignMethod {
    /// Align text within the given width, returning a padded string.
    pub fn align_text(&self, text: &str, width: usize) -> String {
        let text_width = unicode_width::UnicodeWidthStr::width(text);
        if text_width >= width {
            return text.to_string();
        }
        let padding = width - text_width;
        match self {
            Self::Left => format!("{}{}", text, " ".repeat(padding)),
            Self::Right => format!("{}{}", " ".repeat(padding), text),
            Self::Center => {
                let left = padding / 2;
                let right = padding - left;
                format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
            }
            Self::Full => {
                // Full justification: distribute spaces between words
                let words: Vec<&str> = text.split_whitespace().collect();
                if words.len() <= 1 {
                    return format!("{}{}", text, " ".repeat(padding));
                }
                let word_chars: usize = words.iter().map(|w| w.chars().count()).sum();
                let total_gaps = width - word_chars;
                let gap_count = words.len() - 1;
                let gap_size = total_gaps / gap_count;
                let extra = total_gaps % gap_count;
                let mut out = String::new();
                for (i, word) in words.iter().enumerate() {
                    out.push_str(word);
                    if i < gap_count {
                        let spaces = gap_size + if i < extra { 1 } else { 0 };
                        out.push_str(&" ".repeat(spaces));
                    }
                }
                out
            }
        }
    }

    /// Parse an alignment method from its string name (`"left"`, `"center"`, `"right"`, or `"full"`).
    pub fn from_str(s: &str) -> Self {
        match s {
            "left" | "default" => Self::Left,
            "center" => Self::Center,
            "right" => Self::Right,
            "full" => Self::Full,
            _ => Self::Left,
        }
    }
}

impl fmt::Display for AlignMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Center => write!(f, "center"),
            Self::Right => write!(f, "right"),
            Self::Full => write!(f, "full"),
        }
    }
}

impl Default for AlignMethod {
    fn default() -> Self {
        Self::Left
    }
}

// ---------------------------------------------------------------------------
// VerticalAlignMethod
// ---------------------------------------------------------------------------

/// Vertical alignment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalAlignMethod {
    /// Align to the top edge.
    Top,
    /// Align to the vertical center.
    Middle,
    /// Align to the bottom edge.
    Bottom,
}

impl VerticalAlignMethod {
    /// Parse a vertical alignment method from its string name (`"top"`, `"middle"`, or `"bottom"`).
    pub fn from_str(s: &str) -> Self {
        match s {
            "top" => Self::Top,
            "middle" => Self::Middle,
            "bottom" => Self::Bottom,
            _ => Self::Top,
        }
    }
}

impl fmt::Display for VerticalAlignMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Top => write!(f, "top"),
            Self::Middle => write!(f, "middle"),
            Self::Bottom => write!(f, "bottom"),
        }
    }
}

impl Default for VerticalAlignMethod {
    fn default() -> Self {
        Self::Top
    }
}

// ---------------------------------------------------------------------------
// Align — Wraps a renderable with alignment
// ---------------------------------------------------------------------------

use crate::console::{ConsoleOptions, RenderResult};
use crate::segment::Segment;

/// Wraps a renderable to apply horizontal and/or vertical alignment.
#[derive(Debug, Clone)]
pub struct Align<T: crate::console::Renderable> {
    pub renderable: T,
    pub align: AlignMethod,
    pub vertical: VerticalAlignMethod,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

impl<T: crate::console::Renderable> Align<T> {
    /// Wrap a renderable with default left/top alignment.
    pub fn new(renderable: T) -> Self {
        Self {
            renderable,
            align: AlignMethod::Left,
            vertical: VerticalAlignMethod::Top,
            width: None,
            height: None,
        }
    }

    /// Set the horizontal alignment.
    pub fn align(mut self, align: AlignMethod) -> Self {
        self.align = align;
        self
    }

    pub fn vertical(mut self, vertical: VerticalAlignMethod) -> Self {
        self.vertical = vertical;
        self
    }

    pub fn center(renderable: T) -> Self {
        Self::new(renderable).align(AlignMethod::Center)
    }

    pub fn middle(renderable: T) -> Self {
        Self::new(renderable).vertical(VerticalAlignMethod::Middle)
    }
}

impl<T: crate::console::Renderable + Clone> crate::console::Renderable for Align<T> {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let inner_result = self.renderable.render(options);
        let width = self.width.unwrap_or(options.max_width);

        let mut lines: Vec<Vec<Segment>> = Vec::new();

        for line_segs in inner_result.lines {
            // Measure the line width
            let line_text: String = line_segs.iter().map(|s| s.text.as_str()).collect();
            let line_width = unicode_width::UnicodeWidthStr::width(line_text.as_str());

            if line_width >= width {
                lines.push(line_segs);
            } else {
                let padding = width - line_width;
                let (left_pad, _right_pad) = match self.align {
                    AlignMethod::Left => (0, padding),
                    AlignMethod::Right => (padding, 0),
                    AlignMethod::Center => (padding / 2, padding - padding / 2),
                    AlignMethod::Full => (0, padding),
                };

                let mut aligned = Vec::new();
                if left_pad > 0 {
                    aligned.push(Segment::new(" ".repeat(left_pad)));
                }
                aligned.extend(line_segs);
                if padding - left_pad > 0 {
                    aligned.push(Segment::new(" ".repeat(padding - left_pad)));
                }
                aligned.push(Segment::line());
                lines.push(aligned);
            }
        }

        // Vertical alignment
        if let Some(h) = self.height {
            if lines.len() < h {
                let empty_lines = h - lines.len();
                match self.vertical {
                    VerticalAlignMethod::Bottom => {
                        let mut top: Vec<Vec<Segment>> = (0..empty_lines)
                            .map(|_| vec![Segment::new(" ".repeat(width)), Segment::line()])
                            .collect();
                        top.extend(lines);
                        lines = top;
                    }
                    VerticalAlignMethod::Middle => {
                        let top_h = empty_lines / 2;
                        let bottom_h = empty_lines - top_h;
                        let mut result: Vec<Vec<Segment>> = (0..top_h)
                            .map(|_| vec![Segment::new(" ".repeat(width)), Segment::line()])
                            .collect();
                        result.extend(lines);
                        result.extend(
                            (0..bottom_h)
                                .map(|_| vec![Segment::new(" ".repeat(width)), Segment::line()]),
                        );
                        lines = result;
                    }
                    VerticalAlignMethod::Top => {
                        lines.extend(
                            (0..empty_lines)
                                .map(|_| vec![Segment::new(" ".repeat(width)), Segment::line()]),
                        );
                    }
                }
            }
        }

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_center() {
        let result = AlignMethod::Center.align_text("Hi", 10);
        assert_eq!(result.len(), 10);
        assert!(result.starts_with("    "));
    }
}
