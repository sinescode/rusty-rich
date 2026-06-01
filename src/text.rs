//! Text with spans — equivalent to Rich's `text.py`.
//!
//! `Text` is a styled string: plain text plus a collection of `Span`s that
//! mark regions with specific styles.

use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::align::AlignMethod;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Span
// ---------------------------------------------------------------------------

/// A marked-up region in some text.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

impl Span {
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Self { start, end, style }
    }

    /// Check if this span is non-empty.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Split a span at the given offset, producing two spans.
    /// The first covers [start, offset), the second [offset, end).
    pub fn split(&self, offset: usize) -> (Self, Option<Self>) {
        if offset <= self.start || offset >= self.end {
            return (self.clone(), None);
        }
        let span1 = Self::new(self.start, self.end.min(offset), self.style.clone());
        let span2 = Self::new(span1.end, self.end, self.style.clone());
        (span1, Some(span2))
    }

    /// Move start and end by a given offset.
    pub fn move_by(&self, offset: isize) -> Self {
        let start = (self.start as isize + offset).max(0) as usize;
        let end = (self.end as isize + offset).max(0) as usize;
        Self::new(start, end, self.style.clone())
    }

    /// Crop the span end to not exceed `offset`.
    pub fn right_crop(&self, offset: usize) -> Self {
        if offset >= self.end {
            self.clone()
        } else {
            Self::new(self.start, self.end.min(offset), self.style.clone())
        }
    }
}

// ---------------------------------------------------------------------------
// Text
// ---------------------------------------------------------------------------

/// A renderable piece of text with optional style spans.
#[derive(Debug, Clone)]
pub struct Text {
    /// The plain text content.
    pub plain: String,
    /// Style spans applied over the text.
    pub spans: Vec<Span>,
    /// Default style for the entire text.
    pub style: Style,
    /// Justification method.
    pub justify: JustifyMethod,
    /// End string (appended during rendering).
    pub end: String,
    /// Overflow method.
    pub overflow: OverflowMethod,
    /// If true, don't wrap.
    pub no_wrap: bool,
}

pub type JustifyMethod = crate::align::AlignMethod;
pub type OverflowMethod = crate::console::OverflowMethod;

impl Text {
    /// Create a new Text with the given plain content.
    pub fn new(plain: impl Into<String>) -> Self {
        Self {
            plain: plain.into(),
            spans: Vec::new(),
            style: Style::new(),
            justify: JustifyMethod::Left,
            end: "\n".to_string(),
            overflow: OverflowMethod::Fold,
            no_wrap: false,
        }
    }

    /// Create a styled Text.
    pub fn styled(plain: impl Into<String>, style: Style) -> Self {
        Self {
            plain: plain.into(),
            spans: Vec::new(),
            style,
            justify: JustifyMethod::Left,
            end: "\n".to_string(),
            overflow: OverflowMethod::Fold,
            no_wrap: false,
        }
    }

    /// Builder: set the default style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the justification.
    pub fn justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = justify;
        self
    }

    /// Builder: set the end string.
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.end = end.into();
        self
    }

    /// Builder: set overflow method.
    pub fn overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = overflow;
        self
    }

    /// Append another Text or string to this one, with an optional style.
    pub fn append(&mut self, text: impl Into<Text>, style: Option<Style>) {
        let text: Text = text.into();
        let offset = self.plain.len();
        self.plain.push_str(&text.plain);

        // Shift and add spans from the appended text
        for span in &text.spans {
            let mut s = span.clone();
            s.start += offset;
            s.end += offset;
            self.spans.push(s);
        }

        // If a style is given, add a span for the appended text
        if let Some(st) = style {
            self.spans.push(Span::new(
                offset,
                offset + text.plain.len(),
                st,
            ));
        }
    }

    /// Append a plain string with a style.
    pub fn append_styled(&mut self, text: impl Into<String>, style: Style) {
        let text = text.into();
        let offset = self.plain.len();
        self.plain.push_str(&text);
        self.spans.push(Span::new(offset, offset + text.len(), style));
    }

    /// Get the cell length of the plain text.
    pub fn cell_len(&self) -> usize {
        UnicodeWidthStr::width(self.plain.as_str())
    }

    /// Get the style at a given position, combining spans.
    pub fn style_at(&self, position: usize) -> Style {
        let mut style = self.style.clone();
        for span in &self.spans {
            if position >= span.start && position < span.end {
                style = style.combine(&span.style);
            }
        }
        style
    }

    /// Truncate the text to the given maximum width.
    pub fn truncate(&mut self, max_width: usize, overflow: OverflowMethod) {
        let w = self.cell_len();
        if w <= max_width {
            return;
        }

        match overflow {
            OverflowMethod::Ellipsis => {
                let ellipsis = "…";
                let ellip_w = UnicodeWidthStr::width(ellipsis);
                if max_width <= ellip_w {
                    self.plain = ellipsis[..max_width].to_string();
                    self.spans.clear();
                    return;
                }
                // Walk chars from the left until we reach max_width - ellipsis width
                let target = max_width - ellip_w;
                let _char_count = 0usize;
                let mut byte_pos = 0usize;
                let mut w_count = 0usize;
                for (i, ch) in self.plain.char_indices() {
                    let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if w_count + cw > target {
                        break;
                    }
                    w_count += cw;
                    byte_pos = i + ch.len_utf8();
                }
                self.plain.truncate(byte_pos);
                self.plain.push_str(ellipsis);
                // Crop spans
                let crop_at = byte_pos;
                self.spans.retain(|s| s.start < crop_at);
                for s in &mut self.spans {
                    if s.end > crop_at {
                        s.end = crop_at;
                    }
                }
            }
            OverflowMethod::Crop => {
                // Walk chars up to max_width
                let mut w_count = 0usize;
                let mut byte_pos = 0usize;
                for (i, ch) in self.plain.char_indices() {
                    let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if w_count + cw > max_width {
                        break;
                    }
                    w_count += cw;
                    byte_pos = i + ch.len_utf8();
                }
                self.plain.truncate(byte_pos);
                let crop_at = byte_pos;
                self.spans.retain(|s| s.start < crop_at);
                for s in &mut self.spans {
                    if s.end > crop_at {
                        s.end = crop_at;
                    }
                }
            }
            _ => {} // Fold / Ignore: don't truncate
        }
    }

    /// Expand tab characters to spaces.
    pub fn expand_tabs(&mut self) {
        let tab_width = 8;
        let mut result = String::new();
        let mut col = 0usize;
        for ch in self.plain.chars() {
            if ch == '\t' {
                let spaces = tab_width - (col % tab_width);
                result.push_str(&" ".repeat(spaces));
                col += spaces;
            } else {
                result.push(ch);
                col += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            }
        }
        self.plain = result;
    }

    /// Split the text into lines.
    pub fn split_lines(&self) -> Vec<Text> {
        self.plain
            .split('\n')
            .map(|line| Text::new(line.to_string()))
            .collect()
    }

    /// Render the text, applying styles and returning a styled string.
    pub fn render(&self) -> String {
        // Simple: apply spans as ANSI codes
        if self.spans.is_empty() && self.style.is_plain() {
            return self.plain.clone();
        }

        let mut out = String::new();
        let chars: Vec<(usize, char)> = self.plain.char_indices().collect();
        let default_ansi = self.style.to_ansi();
        let reset = if default_ansi.is_empty() { "" } else { "\x1b[0m" };

        if !default_ansi.is_empty() {
            out.push_str(&default_ansi);
        }

        for (byte_pos, ch) in &chars {
            // Apply any spans that start at this position
            let mut applied = String::new();
            for span in &self.spans {
                if span.start == *byte_pos {
                    applied.push_str(&span.style.to_ansi());
                }
            }
            out.push_str(&applied);

            // Output the character
            out.push(*ch);

            // End any spans that finish after this character
            let char_end = byte_pos + ch.len_utf8();
            let mut ended = false;
            for span in &self.spans {
                if span.end == char_end {
                    out.push_str("\x1b[0m");
                    ended = true;
                }
            }
            // If we ended spans but the default style needs re-applying
            if ended && !default_ansi.is_empty() {
                out.push_str(&default_ansi);
            }
        }

        if !reset.is_empty() {
            out.push_str(reset);
        }

        out
    }

    /// Pad the text on both sides with `count` copies of `character`.
    pub fn pad(&mut self, count: usize, character: char) {
        self.plain = format!(
            "{}{}{}",
            character.to_string().repeat(count),
            self.plain,
            character.to_string().repeat(count)
        );
        // Shift spans by `count`
        for span in &mut self.spans {
            span.start += count;
            span.end += count;
        }
    }

    /// Pad the left side only.
    pub fn pad_left(&mut self, count: usize, character: char) {
        self.plain = format!("{}{}", character.to_string().repeat(count), self.plain);
        for span in &mut self.spans {
            span.start += count;
            span.end += count;
        }
    }

    /// Pad the right side only.
    pub fn pad_right(&mut self, count: usize, character: char) {
        self.plain = format!("{}{}", self.plain, character.to_string().repeat(count));
    }

    /// Align the text within a given width using the specified method.
    pub fn align(&mut self, method: AlignMethod, width: usize) {
        let current = self.cell_len();
        if current >= width {
            return;
        }
        let padding = width - current;
        match method {
            AlignMethod::Left => self.pad_right(padding, ' '),
            AlignMethod::Right => self.pad_left(padding, ' '),
            AlignMethod::Center => {
                let left = padding / 2;
                self.pad_left(left, ' ');
                self.pad_right(padding - left, ' ');
            }
            AlignMethod::Full => {} // not applicable
        }
    }

    /// Apply a style to a range of text.
    /// Equivalent to Python's `Text.stylize()`.
    pub fn stylize(&mut self, style: Style, start: usize, end: Option<usize>) {
        let end = end.unwrap_or(self.plain.len());
        if start < end && start < self.plain.len() {
            self.spans.push(Span::new(start, end, style));
        }
    }

    /// Highlight all regex matches with the given style. Returns count of
    /// matches. Equivalent to Python's `Text.highlight_regex()`.
    pub fn highlight_regex(&mut self, pattern: &str, style: Style) -> usize {
        let re = regex::Regex::new(pattern);
        let re = match re {
            Ok(r) => r,
            Err(_) => return 0,
        };

        let mut count = 0usize;
        // Find all matches in the plain text
        let matches: Vec<(usize, usize)> = re
            .find_iter(&self.plain)
            .map(|m| (m.start(), m.end()))
            .collect();

        for (start, end) in matches {
            self.spans.push(Span::new(start, end, style.clone()));
            count += 1;
        }
        count
    }

    /// Simple word-wrap: split text into lines, each not exceeding `width`
    /// cells. Returns a `Vec<Text>`, one per wrapped line.
    /// Equivalent to Python's `Text.wrap()`.
    pub fn wrap(&self, width: usize) -> Vec<Text> {
        let mut lines: Vec<Text> = Vec::new();
        let mut current = Text::new("");

        for word in self.plain.split_whitespace() {
            let word_w = unicode_width::UnicodeWidthStr::width(word);
            let cur_w = current.cell_len();

            if cur_w == 0 {
                // First word
                current = Text::new(word);
            } else if cur_w + 1 + word_w <= width {
                // Fits on current line
                current.plain.push(' ');
                current.plain.push_str(word);
            } else {
                // Start new line
                if !current.plain.is_empty() {
                    lines.push(current);
                }
                current = Text::new(word);
            }
        }

        if !current.plain.is_empty() {
            lines.push(current);
        }

        lines
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ---------------------------------------------------------------------------
// TextType — either a &str or a Text
// ---------------------------------------------------------------------------

/// Represents something that can be used as text: a plain `&str` or a `Text`.
#[derive(Debug, Clone)]
pub enum TextType {
    Plain(String),
    Rich(Text),
}

impl TextType {
    pub fn render(&self) -> String {
        match self {
            Self::Plain(s) => s.clone(),
            Self::Rich(t) => t.render(),
        }
    }
}

impl From<&str> for TextType {
    fn from(s: &str) -> Self {
        Self::Plain(s.to_string())
    }
}

impl From<String> for TextType {
    fn from(s: String) -> Self {
        Self::Plain(s)
    }
}

impl From<Text> for TextType {
    fn from(t: Text) -> Self {
        Self::Rich(t)
    }
}

impl fmt::Display for TextType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(s) => write!(f, "{s}"),
            Self::Rich(t) => write!(f, "{t}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_append() {
        let mut t = Text::new("Hello");
        t.append_styled(" World", Style::new().bold(true));
        assert_eq!(t.plain, "Hello World");
        assert_eq!(t.spans.len(), 1);
        assert_eq!(t.spans[0].start, 5);
        assert_eq!(t.spans[0].end, 11);
    }

    #[test]
    fn test_text_truncate() {
        let mut t = Text::new("Hello World");
        t.truncate(5, OverflowMethod::Ellipsis);
        assert!(t.plain.contains('…'));
    }
}
