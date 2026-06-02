//! Text with spans — equivalent to Rich's `text.py`.
//!
//! `Text` is a styled string: plain text plus a collection of `Span`s that
//! mark regions with specific styles.

use std::fmt;

use regex::Regex;
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
    /// Create a new `Span` covering byte range `[start, end)` with a given style.
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
    /// Tab width for expand_tabs (default 8).
    pub tab_size: usize,
    /// Whether to show indent guide lines.
    pub indent_guides: bool,
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
            tab_size: 8,
            indent_guides: false,
        }
    }

    /// Create a styled Text with a default style but no content.
    pub fn styled(style: Style) -> Self {
        Self {
            plain: String::new(),
            spans: Vec::new(),
            style,
            justify: JustifyMethod::Left,
            end: "\n".to_string(),
            overflow: OverflowMethod::Fold,
            no_wrap: false,
            tab_size: 8,
            indent_guides: false,
        }
    }

    /// Parse an ANSI-escaped string into a styled Text.
    ///
    /// Extracts SGR (Select Graphic Rendition) codes as style spans and strips
    /// all other ANSI escape sequences.
    pub fn from_ansi(text: &str) -> Self {
        let re = Regex::new(r"\x1b\[([\d;]*)([a-zA-Z])").unwrap();
        let mut plain = String::new();
        let mut spans: Vec<Span> = Vec::new();
        let mut current_style = Style::new();
        let mut last_end = 0usize;

        for cap in re.captures_iter(text) {
            let m = cap.get(0).unwrap();
            let match_start = m.start();
            let match_end = m.end();
            let cmd = cap.get(2).map_or("", |m| m.as_str());

            // Text before this escape sequence
            if match_start > last_end {
                let segment = &text[last_end..match_start];
                let start = plain.len();
                plain.push_str(segment);
                let end = plain.len();
                if !current_style.is_plain() && start < end {
                    spans.push(Span::new(start, end, current_style.clone()));
                }
            }

            // Only parse SGR (m command), strip everything else
            if cmd == "m" {
                let params_str = cap.get(1).map_or("", |m| m.as_str());
                apply_sgr(&mut current_style, params_str);
            }

            last_end = match_end;
        }

        // Remaining text after last escape
        if last_end < text.len() {
            let segment = &text[last_end..];
            let start = plain.len();
            plain.push_str(segment);
            let end = plain.len();
            if !current_style.is_plain() && start < end {
                spans.push(Span::new(start, end, current_style.clone()));
            }
        }

        Self {
            plain,
            spans,
            style: Style::new(),
            justify: JustifyMethod::Left,
            end: "\n".to_string(),
            overflow: OverflowMethod::Fold,
            no_wrap: false,
            tab_size: 8,
            indent_guides: false,
        }
    }

    /// Parse a BBCode-like markup string into a styled Text.
    /// Delegates to [`crate::markup::render()`].
    pub fn from_markup(markup: &str) -> Self {
        crate::markup::render(markup)
    }

    /// Get a reference to the default style.
    pub fn get_style(&self) -> &Style {
        &self.style
    }

    /// Get a mutable reference to the default style.
    pub fn get_style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Get a reference to the spans slice.
    pub fn spans(&self) -> &[Span] {
        &self.spans
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

    /// Builder: set no_wrap flag.
    pub fn no_wrap(mut self, value: bool) -> Self {
        self.no_wrap = value;
        self
    }

    /// Builder: set tab size for expand_tabs.
    pub fn tab_size(mut self, size: usize) -> Self {
        self.tab_size = size;
        self
    }

    /// Builder: show indent guides.
    pub fn with_indent_guides(mut self, show: bool) -> Self {
        self.indent_guides = show;
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

    /// Append a sequence of (text, style) tokens.
    pub fn append_tokens(&mut self, tokens: Vec<(String, Style)>) {
        for (text, style) in tokens {
            self.append_styled(text, style);
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

    /// Get the combined style at a specific byte offset.
    /// Iterates through spans that cover the offset and combines them.
    pub fn get_style_at_offset(&self, offset: usize) -> Style {
        self.style_at(offset)
    }

    /// Truncate the text to the given maximum width.
    pub fn truncate(&mut self, max_width: usize, overflow: OverflowMethod) {
        let w = self.cell_len();
        if w <= max_width {
            return;
        }

        match overflow {
            OverflowMethod::Ellipsis => {
                let ellipsis = "\u{2026}";
                let ellip_w = UnicodeWidthStr::width(ellipsis);
                if max_width <= ellip_w {
                    self.plain = ellipsis[..max_width].to_string();
                    self.spans.clear();
                    return;
                }
                // Walk chars from the left until we reach max_width - ellipsis width
                let target = max_width - ellip_w;
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
        let tab_width = self.tab_size;
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

    /// Apply a style to a range of text.
    /// Equivalent to Python's `Text.stylize()`.
    pub fn stylize(&mut self, style: Style, start: usize, end: Option<usize>) {
        let end = end.unwrap_or(self.plain.len());
        if start < end && start < self.plain.len() {
            self.spans.push(Span::new(start, end, style));
        }
    }

    /// Like [`stylize`] but inserts the span at the beginning of the spans
    /// list, giving it lower priority than existing spans.
    pub fn stylize_before(&mut self, style: Style, start: usize, end: Option<usize>) {
        let end = end.unwrap_or(self.plain.len());
        if start < end && start < self.plain.len() {
            self.spans.insert(0, Span::new(start, end, style));
        }
    }

    /// Apply metadata to the text at the given spans.
    /// Sets the style.meta field on overlapping spans.
    pub fn apply_meta(&mut self, meta: Vec<u8>, spans: &[Span]) {
        for span in spans {
            let start = span.start;
            let end = span.end;
            // Apply meta to existing spans that overlap
            for existing in &mut self.spans {
                if existing.start < end && existing.end > start {
                    existing.style.meta = Some(meta.clone());
                }
            }
            // Add a span for this region with the meta
            self.spans.push(Span::new(start, end, {
                let mut s = Style::new();
                s.meta = Some(meta.clone());
                s
            }));
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

    /// Return a new Text with the same style settings but empty content.
    pub fn blank_copy(&self) -> Self {
        Self {
            plain: String::new(),
            spans: Vec::new(),
            style: self.style.clone(),
            justify: self.justify,
            end: self.end.clone(),
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            tab_size: self.tab_size,
            indent_guides: self.indent_guides,
        }
    }

    /// Return a copy with only the plain text (no spans, default style).
    pub fn copy_styles(&self) -> Self {
        Self::new(self.plain.clone())
    }

    /// Detect the leading whitespace indentation.
    /// Returns (indent_string, indent_count) where indent_count is the number
    /// of occurrences of the first indent character.
    pub fn detect_indentation(&self) -> (String, usize) {
        let trimmed_start = self.plain.len() - self.plain.trim_start().len();
        if trimmed_start == 0 {
            return (String::new(), 0);
        }
        let indent_str = self.plain[..trimmed_start].to_string();
        let first_char = indent_str.chars().next().unwrap_or(' ');
        let indent_count = indent_str.chars().filter(|&c| c == first_char).count();
        (indent_str, indent_count)
    }

    /// Split the text at the given byte offsets into multiple pieces,
    /// preserving style spans.
    pub fn divide(&self, offsets: &[usize]) -> Vec<Text> {
        let mut result: Vec<Text> = Vec::new();
        let mut prev = 0usize;
        for &offset in offsets {
            if offset <= prev || offset > self.plain.len() {
                continue;
            }
            result.push(self.slice(prev, offset));
            prev = offset;
        }
        if prev < self.plain.len() {
            result.push(self.slice(prev, self.plain.len()));
        }
        result
    }

    /// Extract a sub-text covering byte range [start, end), preserving spans.
    fn slice(&self, start: usize, end: usize) -> Text {
        let piece = self.plain[start..end].to_string();
        let mut text = Text::new(piece);
        text.style = self.style.clone();
        for span in &self.spans {
            if span.start < end && span.end > start {
                let s_start = if span.start > start { span.start - start } else { 0 };
                let s_end = if span.end < end { span.end - start } else { end - start };
                if s_start < s_end {
                    text.spans.push(Span::new(s_start, s_end, span.style.clone()));
                }
            }
        }
        text
    }

    /// Apply a style to the entire text by adding a full-length span.
    pub fn extend_style(&mut self, style: Style) {
        if !self.plain.is_empty() {
            self.spans.push(Span::new(0, self.plain.len(), style));
        }
    }

    /// Truncate or pad to exactly fit `width` cells.
    /// Uses existing truncate + align methods.
    pub fn fit(&self, width: usize) -> Text {
        let mut copy = self.clone();
        let cell_len = copy.cell_len();
        if cell_len > width {
            copy.truncate(width, OverflowMethod::Crop);
        } else if cell_len < width {
            copy.align(AlignMethod::Left, width);
        }
        copy
    }

    /// Pad with spaces or truncate to exact `length` cells.
    pub fn set_length(&mut self, length: usize) {
        let cell_len = self.cell_len();
        if cell_len > length {
            self.truncate(length, OverflowMethod::Crop);
        } else if cell_len < length {
            self.pad_right(length - cell_len, ' ');
        }
    }

    /// Remove a suffix from the text, returning true if it was present.
    pub fn remove_suffix(&mut self, suffix: &str) -> bool {
        if self.plain.ends_with(suffix) {
            let end = self.plain.len() - suffix.len();
            self.plain.truncate(end);
            self.spans.retain(|s| s.start < end);
            for s in &mut self.spans {
                if s.end > end {
                    s.end = end;
                }
            }
            true
        } else {
            false
        }
    }

    /// Remove a trailing end string, returning true if it was present.
    pub fn rstrip_end(&mut self, end: &str) -> bool {
        self.remove_suffix(end)
    }

    /// Crop all text beyond `offset`. Returns the cropped (removed) portion.
    pub fn right_crop(&mut self, offset: usize) -> Text {
        if offset >= self.plain.len() {
            // Nothing to crop
            return Text::new("");
        }

        let cropped_text = self.plain[offset..].to_string();
        let mut cropped = Text::new(&*cropped_text);
        cropped.style = self.style.clone();

        self.plain.truncate(offset);

        // Split spans at the boundary
        let mut kept_spans: Vec<Span> = Vec::new();
        let mut cropped_spans: Vec<Span> = Vec::new();
        for span in &self.spans {
            if span.start < offset {
                let mut s = span.clone();
                if s.end > offset {
                    // Span crosses the boundary
                    cropped_spans.push(Span::new(0, s.end - offset, span.style.clone()));
                    s.end = offset;
                }
                kept_spans.push(s);
            } else {
                // Entirely in the cropped portion
                cropped_spans.push(Span::new(
                    span.start - offset,
                    span.end - offset,
                    span.style.clone(),
                ));
            }
        }
        self.spans = kept_spans;
        cropped.spans = cropped_spans;
        cropped
    }

    /// Remove trailing whitespace.
    pub fn rstrip(&mut self) -> &mut Self {
        let trimmed_end = self.plain.len() - self.plain.trim_end().len();
        if trimmed_end > 0 {
            let new_len = self.plain.len() - trimmed_end;
            self.plain.truncate(new_len);
            self.spans.retain(|s| s.start < new_len);
            for s in &mut self.spans {
                if s.end > new_len {
                    s.end = new_len;
                }
            }
        }
        self
    }

    /// Split at every character boundary, producing per-character Text pieces.
    pub fn split(&self) -> Vec<Text> {
        let mut result: Vec<Text> = Vec::new();
        let mut byte_pos = 0usize;
        for ch in self.plain.chars() {
            let ch_len = ch.len_utf8();
            let ch_str = &self.plain[byte_pos..byte_pos + ch_len];
            let mut text = Text::new(ch_str.to_string());
            text.style = self.style_at(byte_pos);
            result.push(text);
            byte_pos += ch_len;
        }
        result
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

    /// Serialize the text to a BBCode-like markup string.
    /// Each span is wrapped in `[style]...[/]` tags.
    pub fn markup(&self) -> String {
        if self.spans.is_empty() {
            return crate::markup::escape(&self.plain);
        }

        let mut sorted: Vec<&Span> = self.spans.iter().collect();
        sorted.sort_by_key(|s| (s.start, s.end));

        let mut result = String::new();
        let mut pos = 0usize;

        for span in &sorted {
            if span.start > pos {
                result.push_str(&crate::markup::escape(&self.plain[pos..span.start]));
            }
            if span.start < span.end {
                let style_str = span.style.to_string();
                if style_str != "none" && !style_str.is_empty() {
                    result.push_str(&format!("[{}]", style_str));
                    result.push_str(&crate::markup::escape(&self.plain[span.start..span.end]));
                    result.push_str("[/]");
                } else {
                    result.push_str(&crate::markup::escape(&self.plain[span.start..span.end]));
                }
            }
            pos = pos.max(span.end);
        }

        if pos < self.plain.len() {
            result.push_str(&crate::markup::escape(&self.plain[pos..]));
        }

        result
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
}

impl Default for Text {
    fn default() -> Self {
        Self::new("")
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

// ---------------------------------------------------------------------------
// Internal: parse an SGR parameter string and apply to a Style
// ---------------------------------------------------------------------------

/// Apply ANSI SGR parameters to a style in-place.
/// `params` is the semicolon-separated string after `\x1b[` and before `m`.
fn apply_sgr(style: &mut Style, params: &str) {
    if params.is_empty() || params == "0" {
        // Full reset
        *style = Style::new();
        return;
    }

    let parts: Vec<&str> = params.split(';').collect();
    let mut i = 0usize;
    while i < parts.len() {
        match parts[i] {
            "1" => { *style = style.clone().bold(true); }
            "2" => { *style = style.clone().dim(true); }
            "3" => { *style = style.clone().italic(true); }
            "4" => { *style = style.clone().underline(true); }
            "5" => { *style = style.clone().blink(true); }
            "6" => { *style = style.clone().blink2(true); }
            "7" => { *style = style.clone().reverse(true); }
            "8" => { *style = style.clone().conceal(true); }
            "9" => { *style = style.clone().strike(true); }
            "21" => { *style = style.clone().underline2(true); }
            "22" => { *style = style.clone().bold(false).dim(false); }
            "23" => { *style = style.clone().italic(false); }
            "24" => { *style = style.clone().underline(false); }
            "25" => { *style = style.clone().blink(false).blink2(false); }
            "27" => { *style = style.clone().reverse(false); }
            "28" => { *style = style.clone().conceal(false); }
            "29" => { *style = style.clone().strike(false); }
            "51" => { *style = style.clone().frame(true); }
            "52" => { *style = style.clone().encircle(true); }
            "53" => { *style = style.clone().overline(true); }
            "54" => { *style = style.clone().frame(false).encircle(false); }
            "55" => { *style = style.clone().overline(false); }
            "38" => {
                // Extended foreground color
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "5" => {
                            if i + 2 < parts.len() {
                                if let Ok(n) = parts[i + 2].parse::<u8>() {
                                    let c = crate::color::Color::from_8bit(n);
                                    *style = style.clone().color(c);
                                }
                                i += 2;
                            }
                        }
                        "2" => {
                            if i + 4 < parts.len() {
                                let r = parts[i + 2].parse::<u8>().unwrap_or(0);
                                let g = parts[i + 3].parse::<u8>().unwrap_or(0);
                                let b = parts[i + 4].parse::<u8>().unwrap_or(0);
                                *style = style.clone().color(crate::color::Color::from_rgb(r, g, b));
                                i += 4;
                            }
                        }
                        _ => {}
                    }
                }
            }
            "48" => {
                // Extended background color
                if i + 1 < parts.len() {
                    match parts[i + 1] {
                        "5" => {
                            if i + 2 < parts.len() {
                                if let Ok(n) = parts[i + 2].parse::<u8>() {
                                    let c = crate::color::Color::from_8bit(n);
                                    *style = style.clone().bgcolor(c);
                                }
                                i += 2;
                            }
                        }
                        "2" => {
                            if i + 4 < parts.len() {
                                let r = parts[i + 2].parse::<u8>().unwrap_or(0);
                                let g = parts[i + 3].parse::<u8>().unwrap_or(0);
                                let b = parts[i + 4].parse::<u8>().unwrap_or(0);
                                *style = style.clone().bgcolor(crate::color::Color::from_rgb(r, g, b));
                                i += 4;
                            }
                        }
                        _ => {}
                    }
                }
            }
            "39" => { style.color = None; }
            "49" => { style.bgcolor = None; }
            n => {
                // Standard colors (30-37, 40-47, 90-97, 100-107)
                if let Ok(num) = n.parse::<u8>() {
                    match num {
                        30..=37 => {
                            let c = crate::color::Color::from_8bit(num - 30);
                            *style = style.clone().color(c);
                        }
                        40..=47 => {
                            let c = crate::color::Color::from_8bit(num - 40);
                            *style = style.clone().bgcolor(c);
                        }
                        90..=97 => {
                            let c = crate::color::Color::from_8bit(num - 82);
                            *style = style.clone().color(c);
                        }
                        100..=107 => {
                            let c = crate::color::Color::from_8bit(num - 92);
                            *style = style.clone().bgcolor(c);
                        }
                        _ => {}
                    }
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

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
        assert!(t.plain.contains('\u{2026}'));
    }

    #[test]
    fn test_styled_constructor() {
        let style = Style::new().bold(true).color(Color::parse("red").unwrap());
        let t = Text::styled(style.clone());
        assert_eq!(t.plain, "");
        assert_eq!(t.style, style);
        assert!(t.spans.is_empty());
    }

    #[test]
    fn test_from_ansi_empty() {
        let t = Text::from_ansi("");
        assert_eq!(t.plain, "");
        assert!(t.spans.is_empty());
    }

    #[test]
    fn test_from_ansi_no_escapes() {
        let t = Text::from_ansi("hello world");
        assert_eq!(t.plain, "hello world");
        assert!(t.spans.is_empty());
    }

    #[test]
    fn test_from_ansi_bold() {
        let t = Text::from_ansi("\x1b[1mbold\x1b[0m");
        assert_eq!(t.plain, "bold");
        assert!(!t.spans.is_empty());
    }

    #[test]
    fn test_style_getter() {
        let style = Style::new().bold(true);
        let t = Text::styled(style.clone());
        assert_eq!(t.get_style(), &style);
    }

    #[test]
    fn test_style_mut_getter() {
        let mut t = Text::new("hello");
        t.style = Style::new().bold(true);
        assert_eq!(t.get_style().get_bold(), Some(true));
    }

    #[test]
    fn test_spans_getter() {
        let mut t = Text::new("hello");
        t.stylize(Style::new().bold(true), 0, Some(3));
        assert_eq!(t.spans().len(), 1);
    }

    #[test]
    fn test_from_markup() {
        let t = Text::from_markup("[bold]hello[/bold]");
        assert_eq!(t.plain, "hello");
        assert!(!t.spans.is_empty());
    }

    #[test]
    fn test_append_tokens() {
        let mut t = Text::new("");
        let tokens = vec![
            ("Hello ".to_string(), Style::new().bold(true)),
            ("World".to_string(), Style::new().italic(true)),
        ];
        t.append_tokens(tokens);
        assert_eq!(t.plain, "Hello World");
        assert_eq!(t.spans.len(), 2);
    }

    #[test]
    fn test_stylize_before() {
        let mut t = Text::new("hello");
        t.stylize(Style::new().bold(true), 0, Some(5));
        t.stylize_before(Style::new().italic(true), 0, Some(5));
        // stylize_before inserts at the beginning (index 0)
        assert_eq!(t.spans.len(), 2);
        assert_eq!(t.spans[0].style.get_italic(), Some(true));
    }

    #[test]
    fn test_blank_copy() {
        let mut t = Text::new("hello");
        t.stylize(Style::new().bold(true), 0, Some(3));
        t.justify = JustifyMethod::Center;
        let blank = t.blank_copy();
        assert_eq!(blank.plain, "");
        assert!(blank.spans.is_empty());
        assert_eq!(blank.justify, JustifyMethod::Center);
    }

    #[test]
    fn test_copy_styles() {
        let mut t = Text::new("hello");
        t.stylize(Style::new().bold(true), 0, Some(3));
        let copy = t.copy_styles();
        assert_eq!(copy.plain, "hello");
        assert!(copy.spans.is_empty());
    }

    #[test]
    fn test_detect_indentation() {
        let t = Text::new("  hello");
        let (indent, count) = t.detect_indentation();
        assert_eq!(indent, "  ");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_detect_indentation_none() {
        let t = Text::new("hello");
        let (indent, count) = t.detect_indentation();
        assert_eq!(indent, "");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_style_at_offset() {
        let mut t = Text::new("hello world");
        t.stylize(Style::new().bold(true), 0, Some(5));
        let s0 = t.get_style_at_offset(0);
        assert_eq!(s0.get_bold(), Some(true));
        let s6 = t.get_style_at_offset(6);
        assert_eq!(s6.get_bold(), None); // not in span, default style has no bold
    }

    #[test]
    fn test_divide() {
        let mut t = Text::new("abcdef");
        t.stylize(Style::new().bold(true), 0, Some(3));
        let parts = t.divide(&[2, 4]);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].plain, "ab");
        assert_eq!(parts[1].plain, "cd");
        assert_eq!(parts[2].plain, "ef");
    }

    #[test]
    fn test_extend_style() {
        let mut t = Text::new("hello");
        t.extend_style(Style::new().bold(true));
        assert_eq!(t.spans.len(), 1);
        assert_eq!(t.spans[0].start, 0);
        assert_eq!(t.spans[0].end, 5);
    }

    #[test]
    fn test_fit_truncate() {
        let t = Text::new("hello world");
        let fitted = t.fit(5);
        assert_eq!(fitted.cell_len(), 5);
    }

    #[test]
    fn test_fit_pad() {
        let t = Text::new("hi");
        let fitted = t.fit(10);
        assert_eq!(fitted.cell_len(), 10);
    }

    #[test]
    fn test_set_length_truncate() {
        let mut t = Text::new("hello world");
        t.set_length(5);
        assert_eq!(t.cell_len(), 5);
    }

    #[test]
    fn test_set_length_pad() {
        let mut t = Text::new("hi");
        t.set_length(10);
        assert_eq!(t.cell_len(), 10);
    }

    #[test]
    fn test_remove_suffix() {
        let mut t = Text::new("hello.txt");
        assert!(t.remove_suffix(".txt"));
        assert_eq!(t.plain, "hello");
    }

    #[test]
    fn test_remove_suffix_not_found() {
        let mut t = Text::new("hello");
        assert!(!t.remove_suffix(".txt"));
        assert_eq!(t.plain, "hello");
    }

    #[test]
    fn test_right_crop() {
        let mut t = Text::new("hello world");
        let cropped = t.right_crop(5);
        assert_eq!(t.plain, "hello");
        assert_eq!(cropped.plain, " world");
    }

    #[test]
    fn test_rstrip() {
        let mut t = Text::new("hello   ");
        t.rstrip();
        assert_eq!(t.plain, "hello");
    }

    #[test]
    fn test_rstrip_end() {
        let mut t = Text::new("hello\n");
        assert!(t.rstrip_end("\n"));
        assert_eq!(t.plain, "hello");
    }

    #[test]
    fn test_split_chars() {
        let t = Text::new("abc");
        let chars = t.split();
        assert_eq!(chars.len(), 3);
        assert_eq!(chars[0].plain, "a");
        assert_eq!(chars[1].plain, "b");
        assert_eq!(chars[2].plain, "c");
    }

    #[test]
    fn test_markup() {
        let mut t = Text::new("hello world");
        t.stylize(Style::new().bold(true), 0, Some(5));
        let markup = t.markup();
        assert!(markup.contains("[bold]"));
        assert!(markup.contains("[/]"));
        assert!(markup.contains("hello"));
        assert!(markup.contains(" world"));
    }

    #[test]
    fn test_markup_no_spans() {
        let t = Text::new("hello");
        assert_eq!(t.markup(), "hello");
    }

    #[test]
    fn test_no_wrap_builder() {
        let t = Text::new("hello").no_wrap(true);
        assert!(t.no_wrap);
    }

    #[test]
    fn test_tab_size_builder() {
        let t = Text::new("hello").tab_size(4);
        assert_eq!(t.tab_size, 4);
    }

    #[test]
    fn test_with_indent_guides_builder() {
        let t = Text::new("hello").with_indent_guides(true);
        assert!(t.indent_guides);
    }

    #[test]
    fn test_expand_tabs_with_custom_size() {
        let mut t = Text::new("\thello").tab_size(4);
        t.expand_tabs();
        assert_eq!(t.plain, "    hello");
    }

    #[test]
    fn test_apply_meta() {
        let mut t = Text::new("hello world");
        let meta = vec![1u8, 2u8, 3u8];
        let spans = vec![Span::new(0, 5, Style::new())];
        t.apply_meta(meta, &spans);
        // Should have added a span with meta
        assert!(t.spans.iter().any(|s| s.style.meta.is_some()));
    }

    #[test]
    fn test_default_impl() {
        let t: Text = Default::default();
        assert_eq!(t.plain, "");
        assert_eq!(t.tab_size, 8);
    }
}
