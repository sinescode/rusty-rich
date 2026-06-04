//! Rich repr protocol — equivalent to Python rich's `repr.py`.
//! Provides `__rich_repr__`-like functionality for Rust types.

#[cfg(feature = "syntax-highlighting")]
use crate::highlighter::ReprHighlighter;
use crate::style::Style;
use crate::text::Text;

/// Error type for repr operations.
#[derive(Debug, Clone)]
pub struct ReprError {
    pub message: String,
    pub source: Option<String>,
}

impl std::fmt::Display for ReprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReprError: {}", self.message)
    }
}

impl std::error::Error for ReprError {}

/// Trait for types that provide a rich representation.
pub trait RichRepr {
    /// Generate a rich representation of self.
    fn rich_repr(&self) -> Text;
}

/// Auto-implement RichRepr for Debug types using the ReprHighlighter.
pub fn auto<T: std::fmt::Debug>(value: &T) -> Text {
    let debug_str = format!("{:#?}", value);
    ReprHighlighter::new().highlight_str(&debug_str)
}

/// Create a rich representation with custom formatting.
pub fn rich_repr(
    type_name: &str,
    fields: &[(&str, &dyn std::fmt::Display)],
    _options: Option<&ReprOptions>,
) -> Text {
    let mut text = Text::new("");

    // Type name in bold
    text.append_styled(
        type_name,
        Style::new()
            .bold(true)
            .color(crate::color::Color::parse("cyan").unwrap()),
    );

    text.plain.push('(');

    for (i, (name, value)) in fields.iter().enumerate() {
        if i > 0 {
            text.plain.push_str(", ");
        }
        text.append_styled(
            format!("{}=", name),
            Style::new().color(crate::color::Color::parse("yellow").unwrap()),
        );
        text.append_styled(
            value.to_string(),
            Style::new().color(crate::color::Color::parse("green").unwrap()),
        );
    }

    text.plain.push(')');
    text
}

/// Options for repr formatting.
#[derive(Debug, Clone)]
pub struct ReprOptions {
    pub max_string: Option<usize>,
    pub max_depth: Option<usize>,
    pub max_length: Option<usize>,
    pub indent_guides: bool,
    pub expand_all: bool,
}

impl Default for ReprOptions {
    fn default() -> Self {
        Self {
            max_string: Some(100),
            max_depth: Some(4),
            max_length: Some(100),
            indent_guides: true,
            expand_all: false,
        }
    }
}

impl ReprOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn max_string(mut self, max: usize) -> Self {
        self.max_string = Some(max);
        self
    }
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }
    pub fn indent_guides(mut self, value: bool) -> Self {
        self.indent_guides = value;
        self
    }
    pub fn expand_all(mut self) -> Self {
        self.expand_all = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto() {
        let value = vec![1, 2, 3];
        let text = auto(&value);
        assert!(!text.plain.is_empty());
    }

    #[test]
    fn test_rich_repr() {
        let text = rich_repr("Point", &[("x", &1), ("y", &2)], None);
        assert!(text.plain.contains("Point"));
        assert!(text.plain.contains("x=1"));
        assert!(text.plain.contains("y=2"));
    }
}
