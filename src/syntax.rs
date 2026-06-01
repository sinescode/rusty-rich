//! Syntax highlighting — equivalent to Rich's `syntax.py`.
//!
//! Uses `syntect` for syntax highlighting (Rust equivalent of Pygments).

use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style as SyntectStyle};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

/// A syntax-highlighted source code renderable.
#[derive(Debug, Clone)]
pub struct Syntax {
    /// The source code.
    pub code: String,
    /// The language name (e.g. "rust", "python", "javascript").
    pub language: String,
    /// Optional theme name.
    pub theme: String,
    /// Starting line number (for line numbers).
    pub start_line: usize,
    /// If true, show line numbers.
    pub line_numbers: bool,
    /// If true, highlight the code.
    pub highlight: bool,
    /// Optional background color.
    pub background_color: Option<crate::color::Color>,
    /// Tab size.
    pub tab_size: usize,
}

impl Syntax {
    /// Create a new Syntax renderable for the given code and language.
    pub fn new(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language: language.into(),
            theme: "base16-ocean.dark".to_string(),
            start_line: 1,
            line_numbers: false,
            highlight: true,
            background_color: None,
            tab_size: 4,
        }
    }

    /// Builder: set theme.
    pub fn theme(mut self, theme: impl Into<String>) -> Self { self.theme = theme.into(); self }

    /// Builder: show line numbers.
    pub fn line_numbers(mut self) -> Self { self.line_numbers = true; self }

    /// Builder: set start line.
    pub fn start_line(mut self, n: usize) -> Self { self.start_line = n; self }

    /// Builder: set background color.
    pub fn background(mut self, color: crate::color::Color) -> Self { self.background_color = Some(color); self }
}

impl Renderable for Syntax {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        if !self.highlight || self.language.is_empty() {
            // No highlighting — just render as plain text
            let lines: Vec<Vec<Segment>> = self
                .code
                .lines()
                .map(|line| vec![Segment::new(line), Segment::line()])
                .collect();
            return RenderResult { lines, items: Vec::new() };
        }

        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let syntax = ss
            .find_syntax_by_name(&self.language)
            .or_else(|| ss.find_syntax_by_extension(&self.language))
            .unwrap_or_else(|| ss.find_syntax_plain_text());

        let theme = &ts.themes[&self.theme];

        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let line_num_width = if self.line_numbers {
            (self.code.lines().count().saturating_add(self.start_line))
                .to_string()
                .len()
        } else {
            0
        };

        for (i, line) in LinesWithEndings::from(&self.code).enumerate() {
            let mut line_segments: Vec<Segment> = Vec::new();

            // Line number
            if self.line_numbers {
                let num = i + self.start_line;
                let num_str = format!("{:>width$} │ ", num, width = line_num_width);
                line_segments.push(Segment::new(num_str));
            }

            // Highlight the line
            match highlighter.highlight_line(line, &ss) {
                Ok(highlighted) => {
                    for (syntect_style, text) in &highlighted {
                        let style = syntect_to_rich_style(syntect_style);
                        line_segments.push(Segment::styled(
                            text.to_string(),
                            style,
                        ));
                    }
                }
                Err(_) => {
                    line_segments.push(Segment::new(line));
                }
            }

            lines.push(line_segments);
        }

        RenderResult { lines, items: Vec::new() }
    }
}

/// Convert a syntect `Style` to our `Style`.
fn syntect_to_rich_style(ss: &SyntectStyle) -> Style {
    let mut style = Style::new();
    let fg = ss.foreground;
    style = style.color(crate::color::Color::from_rgb(fg.r, fg.g, fg.b));

    if ss.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
        style = style.bold(true);
    }
    if ss.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
        style = style.italic(true);
    }
    if ss.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
        style = style.underline(true);
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_no_highlight() {
        let s = Syntax::new("fn main() {}", "rust");
        let opts = ConsoleOptions::default();
        let result = s.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("fn main"));
    }

    #[test]
    fn test_syntax_line_numbers() {
        let s = Syntax::new("line1\nline2\nline3", "").line_numbers();
        let opts = ConsoleOptions::default();
        let result = s.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("1"));
    }
}
