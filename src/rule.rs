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
            return RenderResult::new();
        }

        // Build a simple rule line using styled segments
        let make_rule_line = |w: usize| -> Vec<Segment> {
            let count = w / char_w;
            let line_text = chars.repeat(count.max(1));
            let mut seg = if self.style.is_plain() {
                Segment::new(line_text)
            } else {
                Segment::styled(line_text, self.style.clone())
            };
            let cell_len = seg.cell_length();
            if cell_len > w {
                // Truncate to exact width
                let new_text = crate::cells::set_cell_size(&seg.text, w);
                seg.text = new_text;
            }
            vec![seg, Segment::line()]
        };

        if self.title.is_empty() {
            return RenderResult::from_segments(make_rule_line(width));
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
            return RenderResult::from_segments(make_rule_line(width));
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

                let left_text = chars.repeat((left_w / char_w).max(1));
                let right_text = chars.repeat((right_w / char_w).max(1));

                // Truncate to exact width
                let left_actual = crate::cells::set_cell_size(&left_text, left_w);
                let right_actual = crate::cells::set_cell_size(&right_text, right_w);

                if self.style.is_plain() {
                    segments.push(Segment::new(format!(
                        "{left_actual} {} {right_actual}",
                        self.title
                    )));
                } else {
                    segments.push(Segment::styled(
                        left_actual,
                        self.style.clone(),
                    ));
                    segments.push(Segment::new(format!(" {} ", self.title)));
                    segments.push(Segment::styled(
                        right_actual,
                        self.style.clone(),
                    ));
                }
            }
            AlignMethod::Left => {
                let rem = width.saturating_sub(title_w + 1);
                let right_text = chars.repeat((rem / char_w).max(1));
                let right_actual = crate::cells::set_cell_size(&right_text, rem);

                segments.push(Segment::new(format!("{} ", self.title)));
                if self.style.is_plain() {
                    segments.push(Segment::new(right_actual));
                } else {
                    segments.push(Segment::styled(right_actual, self.style.clone()));
                }
            }
            AlignMethod::Right => {
                let rem = width.saturating_sub(title_w + 1);
                let left_text = chars.repeat((rem / char_w).max(1));
                let left_actual = crate::cells::set_cell_size(&left_text, rem);

                if self.style.is_plain() {
                    segments.push(Segment::new(format!("{left_actual} ")));
                } else {
                    segments.push(Segment::styled(left_actual, self.style.clone()));
                    segments.push(Segment::new(" "));
                }
                segments.push(Segment::new(self.title.clone()));
            }
            AlignMethod::Full => {
                return RenderResult::from_segments(make_rule_line(width));
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
