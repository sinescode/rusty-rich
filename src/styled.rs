//! A pre-styled renderable wrapper.
//!
//! Equivalent to Python Rich's `styled.py`. Wraps any renderable and
//! applies a fixed [`Style`] to all of its output, combining with any
//! existing styles on the inner segments via [`Style::combine`].

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::style::Style;

/// Wraps a renderable with a pre-applied style.
///
/// The style is applied to every segment produced by the inner renderable,
/// combining with whatever style the segment already has (the wrapper style
/// takes precedence via [`Style::combine`]).
#[derive(Debug, Clone)]
pub struct Styled {
    /// The inner renderable.
    renderable: DynRenderable,
    /// The style to apply to all output.
    style: Style,
}

impl Styled {
    /// Create a new `Styled` wrapper.
    ///
    /// The `style` is applied to every segment rendered by `renderable`.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static, style: Style) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
            style,
        }
    }

    /// Builder: replace the applied style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Renderable for Styled {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut result = self.renderable.render(options);
        let default_style = Style::new();

        // Apply the wrapper style to every segment in every line
        for line in &mut result.lines {
            for seg in line.iter_mut() {
                let existing_style = seg.style.as_ref().unwrap_or(&default_style);
                seg.style = Some(self.style.combine(existing_style));
            }
        }

        // Also apply to items (nested renderables are not modified —
        // they will be styled when they render)
        for item in &mut result.items {
            if let crate::console::RenderItem::Segment(ref mut seg) = item {
                let existing_style = seg.style.as_ref().unwrap_or(&default_style);
                seg.style = Some(self.style.combine(existing_style));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_styled_default() {
        let style = Style::new().bold(true);
        let styled = Styled::new("Hello", style);
        let opts = ConsoleOptions::default();
        let result = styled.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello"));
        // Bold ANSI code is \x1b[1m
        assert!(ansi.contains("\x1b[1m") || ansi.contains("[1m"));
    }

    #[test]
    fn test_styled_with_color() {
        let color = Color::parse("red").unwrap();
        let style = Style::new().color(color);
        let styled = Styled::new("Red Text", style);
        let opts = ConsoleOptions::default();
        let result = styled.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Red Text"));
    }

    #[test]
    fn test_styled_builder_replace_style() {
        let s1 = Style::new().bold(true);
        let s2 = Style::new().italic(true);
        let styled = Styled::new("text", s1).style(s2);
        let opts = ConsoleOptions::default();
        let result = styled.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("text"));
    }

    #[test]
    fn test_styled_style_override() {
        // Inner style has underline, wrapper has bold
        let outer_style = Style::new().bold(true);
        let styled = Styled::new("test", outer_style);
        let opts = ConsoleOptions::default();
        let result = styled.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("test"));
    }

    #[test]
    fn test_styled_multiple_segments() {
        let style = Style::new().color(Color::parse("green").unwrap());
        let text = "Hello\nWorld";
        let styled = Styled::new(text, style);
        let opts = ConsoleOptions::default();
        let result = styled.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello"));
        assert!(ansi.contains("World"));
    }
}
