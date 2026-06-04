//! Rule — horizontal rule / divider. Equivalent to Rich's `rule.py`.

use crate::align::AlignMethod;
use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use unicode_width::UnicodeWidthStr;

/// A horizontal rule (divider) with an optional title.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Optional title text.
    pub title: String,
    /// Character(s) used for the line.
    pub characters: String,
    /// Style for the rule line.
    pub style: Style,
    /// Text appended after the rule.
    pub end: String,
    /// Alignment of the title.
    pub align: AlignMethod,
}

impl Rule {
    /// Create a new Rule.
    pub fn new() -> Self {
        Self {
            title: String::new(),
            characters: "─".to_string(),
            style: Style::new(),
            end: "\n".to_string(),
            align: AlignMethod::Center,
        }
    }

    /// Builder: set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: set the characters.
    pub fn characters(mut self, chars: impl Into<String>) -> Self {
        self.characters = chars.into();
        self
    }

    /// Builder: set the style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the alignment.
    pub fn align(mut self, align: AlignMethod) -> Self {
        self.align = align;
        self
    }
}

impl Renderable for Rule {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let width = options.max_width;
        let chars = if options.ascii_only && !self.characters.is_ascii() {
            "-"
        } else {
            self.characters.as_str()
        };
        let char_w = UnicodeWidthStr::width(chars);

        if char_w == 0 {
            return RenderResult::from_text("");
        }

        let style_ansi = self.style.to_ansi();
        let style_reset = if style_ansi.is_empty() { "" } else { "\x1b[0m" };

        if self.title.is_empty() {
            // Simple rule line
            let count = width / char_w;
            let line = chars.repeat(count);
            return RenderResult::from_segments(vec![
                Segment::new(format!("{style_ansi}{line}{style_reset}")),
                Segment::line(),
            ]);
        }

        let title_w = UnicodeWidthStr::width(self.title.as_str());
        let required_space = if matches!(self.align, AlignMethod::Center) {
            4
        } else {
            2
        };
        let available = width.saturating_sub(required_space);

        if available < 1 {
            // Not enough space — just draw a plain rule
            let count = width / char_w;
            let line = chars.repeat(count);
            return RenderResult::from_segments(vec![
                Segment::new(format!("{style_ansi}{line}{style_reset}")),
                Segment::line(),
            ]);
        }

        let mut segments = Vec::new();

        match self.align {
            AlignMethod::Center => {
                let side = (width.saturating_sub(title_w)) / 2;
                let left_w = side.saturating_sub(1);
                let right_w = width
                    .saturating_sub(left_w)
                    .saturating_sub(title_w)
                    .saturating_sub(2);

                let left = chars.repeat((left_w / char_w).max(1));
                let right = chars.repeat((right_w / char_w).max(1));

                segments.push(Segment::new(format!(
                    "{style_ansi}{left} {}{} {right}{style_reset}",
                    self.title, style_ansi
                )));
            }
            AlignMethod::Left => {
                let rem = width.saturating_sub(title_w + 1);
                let right = chars.repeat((rem / char_w).max(1));
                segments.push(Segment::new(format!(
                    "{style_ansi}{} {right}{style_reset}",
                    self.title
                )));
            }
            AlignMethod::Right => {
                let rem = width.saturating_sub(title_w + 1);
                let left = chars.repeat((rem / char_w).max(1));
                segments.push(Segment::new(format!(
                    "{style_ansi}{left} {}{style_reset}",
                    self.title
                )));
            }
            AlignMethod::Full => {
                let count = width / char_w;
                let line = chars.repeat(count);
                segments.push(Segment::new(format!("{style_ansi}{line}{style_reset}")));
            }
        }

        segments.push(Segment::line());
        RenderResult::from_segments(segments)
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_plain_rule() {
        let rule = Rule::new();
        let opts = ConsoleOptions {
            max_width: 40,
            ..Default::default()
        };
        let result = rule.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains('─'));
    }

    #[test]
    fn test_rule_with_title() {
        let rule = Rule::new().title("Section");
        let opts = ConsoleOptions {
            max_width: 40,
            ..Default::default()
        };
        let result = rule.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Section"));
    }
}
