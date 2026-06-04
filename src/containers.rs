//! Container renderables — collections of lines and renderables.
//!
//! Provides [`Lines`] for rendering a collection of lines with optional
//! highlight support, and [`Renderables`] for rendering a sequence of
//! independent renderables one after another.

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Lines
// ---------------------------------------------------------------------------

/// A collection of lines (each line is a [`Renderable`]).
///
/// Each item in [`Lines`] is rendered as a single line of output. The
/// `highlight` option applies a style to a specific 0-indexed line.
///
/// # Example
///
/// ```rust
/// use rusty_rich::Lines;
///
/// let mut lines = Lines::new();
/// lines.add("First line");
/// lines.add("Second line");
/// ```
#[derive(Debug, Clone)]
pub struct Lines {
    lines: Vec<DynRenderable>,
    highlight: Option<usize>,
    style: Style,
}

impl Default for Lines {
    fn default() -> Self {
        Self::new()
    }
}

impl Lines {
    /// Create a new empty [`Lines`] container.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            highlight: None,
            style: Style::new(),
        }
    }

    /// Add a renderable line to the container.
    pub fn add(&mut self, renderable: impl Renderable + Send + Sync + 'static) -> &mut Self {
        self.lines.push(DynRenderable::new(renderable));
        self
    }

    /// Builder: highlight the line at the given 0-based index.
    pub fn highlight(mut self, index: usize) -> Self {
        self.highlight = Some(index);
        self
    }

    /// Builder: set the default style for all lines.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Renderable for Lines {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut all_lines: Vec<Vec<Segment>> = Vec::new();

        for (i, item) in self.lines.iter().enumerate() {
            let mut result = item.render(options);

            // Apply highlight style to the highlighted line
            if Some(i) == self.highlight {
                for line in &mut result.lines {
                    for seg in line.iter_mut() {
                        if let Some(ref existing) = seg.style {
                            seg.style = Some(existing.clone().bold(true));
                        } else {
                            seg.style = Some(self.style.clone().bold(true));
                        }
                    }
                }
            } else if !self.style.is_plain() {
                for line in &mut result.lines {
                    for seg in line.iter_mut() {
                        if seg.style.is_none() {
                            seg.style = Some(self.style.clone());
                        }
                    }
                }
            }

            all_lines.extend(result.lines);
        }

        RenderResult {
            lines: all_lines,
            items: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Renderables
// ---------------------------------------------------------------------------

/// A flexible container for multiple renderables.
///
/// Renders each contained renderable in sequence, concatenating their
/// output lines.
///
/// # Example
///
/// ```rust
/// use rusty_rich::Renderables;
///
/// let mut items = Renderables::new();
/// items.add("First item");
/// items.add("Second item");
/// ```
#[derive(Debug, Clone)]
pub struct Renderables {
    items: Vec<DynRenderable>,
}

impl Default for Renderables {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderables {
    /// Create a new empty [`Renderables`] container.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a renderable to the container.
    pub fn add(&mut self, renderable: impl Renderable + Send + Sync + 'static) -> &mut Self {
        self.items.push(DynRenderable::new(renderable));
        self
    }
}

impl Renderable for Renderables {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut all_lines: Vec<Vec<Segment>> = Vec::new();
        for item in &self.items {
            let result = item.render(options);
            all_lines.extend(result.lines);
        }
        RenderResult {
            lines: all_lines,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_lines_empty() {
        let lines = Lines::new();
        let opts = ConsoleOptions::default();
        let result = lines.render(&opts);
        assert!(result.lines.is_empty());
    }

    #[test]
    fn test_lines_with_content() {
        let mut lines = Lines::new();
        lines.add("Hello");
        lines.add("World");
        let opts = ConsoleOptions::default();
        let result = lines.render(&opts);
        assert_eq!(result.lines.len(), 2);
    }

    #[test]
    fn test_lines_highlight() {
        let mut lines = Lines::new().highlight(1);
        lines.add("First");
        lines.add("Highlighted");
        lines.add("Third");
        let opts = ConsoleOptions::default();
        let result = lines.render(&opts);
        assert_eq!(result.lines.len(), 3);
    }

    #[test]
    fn test_renderables_empty() {
        let items = Renderables::new();
        let opts = ConsoleOptions::default();
        let result = items.render(&opts);
        assert!(result.lines.is_empty());
    }

    #[test]
    fn test_renderables_with_content() {
        let mut items = Renderables::new();
        items.add("A");
        items.add("B");
        items.add("C");
        let opts = ConsoleOptions::default();
        let result = items.render(&opts);
        assert_eq!(result.lines.len(), 3);
    }
}
