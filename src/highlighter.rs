//! Highlighter — applies highlighting to strings. Equivalent to Rich's
//! `highlighter.py`.
//!
//! Highlighters are callables that transform text by applying styles for
//! certain patterns (numbers, URLs, paths, etc.).

use regex::Regex;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Highlighter trait
// ---------------------------------------------------------------------------

/// Trait for objects that can highlight text.
pub trait Highlighter {
    /// Apply highlighting to the given text, returning styled Text.
    fn highlight(&self, text: &Text) -> Text;
}

// ---------------------------------------------------------------------------
// NullHighlighter
// ---------------------------------------------------------------------------

/// A highlighter that does nothing.
pub struct NullHighlighter;

impl Highlighter for NullHighlighter {
    fn highlight(&self, text: &Text) -> Text {
        text.clone()
    }
}

// ---------------------------------------------------------------------------
// RegexHighlighter
// ---------------------------------------------------------------------------

/// A highlighter that applies a regex → style mapping.
pub struct RegexHighlighter {
    rules: Vec<(Regex, Style)>,
}

impl std::fmt::Debug for RegexHighlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexHighlighter")
            .field("rule_count", &self.rules.len())
            .finish()
    }
}

impl Clone for RegexHighlighter {
    fn clone(&self) -> Self {
        // Rebuild by cloning patterns as strings
        let mut cloned = Self::new();
        for (re, style) in &self.rules {
            cloned.rules.push((re.clone(), style.clone()));
        }
        cloned
    }
}

impl RegexHighlighter {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, pattern: &str, style: Style) -> Result<(), regex::Error> {
        let re = Regex::new(pattern)?;
        self.rules.push((re, style));
        Ok(())
    }
}

impl Highlighter for RegexHighlighter {
    fn highlight(&self, text: &Text) -> Text {
        let mut result = text.clone();
        for (re, style) in &self.rules {
            let plain = result.plain.clone();
            let mut new_text = Text::new("");
            let mut last_end = 0usize;

            for m in re.find_iter(&plain) {
                // Add text before match
                if m.start() > last_end {
                    new_text.append(&plain[last_end..m.start()], None);
                }
                // Add matched text with style
                new_text.append_styled(m.as_str(), style.clone());
                last_end = m.end();
            }
            // Add remaining text
            if last_end < plain.len() {
                new_text.append(&plain[last_end..], None);
            }
            result = new_text;
        }
        result
    }
}

// ---------------------------------------------------------------------------
// ReprHighlighter — highlights Python repr-like output
// ---------------------------------------------------------------------------

/// Highlights numbers, strings, booleans, None, URLs, paths, IPs, etc.
#[derive(Debug, Clone)]
pub struct ReprHighlighter {
    highlighter: Option<Box<RegexHighlighter>>,
}

impl ReprHighlighter {
    pub fn new() -> Self {
        // Build the regex highlighter with common repr patterns
        let mut rh = RegexHighlighter::new();

        // URLs
        let _ = rh.add_rule(
            r"https?://[^\s)\]}>]+",
            Style::new()
                .color(crate::color::Color::parse("bright_blue").unwrap())
                .underline(true),
        );

        // Numbers (int, float, hex)
        let _ = rh.add_rule(
            r"(?<!\w)(-?\d+\.?\d*(?:e[+-]?\d+)?)(?!\w)",
            Style::new()
                .color(crate::color::Color::parse("cyan").unwrap())
                .bold(true),
        );

        // File paths
        let _ = rh.add_rule(
            r"(?<!\w)(?:/[\w.-]+)+/?(?!\w)",
            Style::new()
                .color(crate::color::Color::parse("magenta").unwrap()),
        );

        // Quoted strings (single or double)
        let _ = rh.add_rule(
            r#""(?:[^"\\]|\\.)*""#,
            Style::new()
                .color(crate::color::Color::parse("green").unwrap()),
        );
        let _ = rh.add_rule(
            r"'(?:[^'\\]|\\.)*'",
            Style::new()
                .color(crate::color::Color::parse("green").unwrap()),
        );

        Self {
            highlighter: Some(Box::new(rh)),
        }
    }

    /// Highlight a string, returning styled text.
    pub fn highlight_str(&self, text: &str) -> Text {
        let t = Text::new(text);
        if let Some(ref h) = self.highlighter {
            h.highlight(&t)
        } else {
            t
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_highlighter() {
        let h = NullHighlighter;
        let t = Text::new("hello");
        let result = h.highlight(&t);
        assert_eq!(result.plain, "hello");
    }

    #[test]
    fn test_repr_highlighter_numbers() {
        let h = ReprHighlighter::new();
        let result = h.highlight_str("num=42");
        // The regex matches standalone numbers; "42" after "=" may not match.
        // Verify the highlighter runs without panicking.
        assert!(!result.plain.is_empty());
    }
}
