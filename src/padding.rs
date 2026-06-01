//! Padding — draw space around content. Equivalent to Rich's `padding.py`.

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// PaddingDimensions
// ---------------------------------------------------------------------------

/// Padding specification (CSS-style: 1, 2, or 4 values).
#[derive(Debug, Clone, Copy)]
pub struct PaddingDimensions {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl PaddingDimensions {
    /// Create from a single value (all sides equal).
    pub fn all(pad: usize) -> Self {
        Self { top: pad, right: pad, bottom: pad, left: pad }
    }

    /// Create from (vertical, horizontal).
    pub fn symmetric(vertical: usize, horizontal: usize) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }

    /// Create from (top, right, bottom, left).
    pub fn new(top: usize, right: usize, bottom: usize, left: usize) -> Self {
        Self { top, right, bottom, left }
    }
}

// ---------------------------------------------------------------------------
// Padding
// ---------------------------------------------------------------------------

/// A renderable that adds padding around its content.
#[derive(Clone)]
pub struct Padding {
    /// The inner renderable.
    pub renderable: DynRenderable,
    /// Padding dimensions.
    pub pad: PaddingDimensions,
    /// Style for the padding (space) characters.
    pub style: Style,
    /// If true, expand padding to fill available width.
    pub expand: bool,
}

impl Padding {
    /// Create a new Padding wrapper.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
            pad: PaddingDimensions::all(0),
            style: Style::new(),
            expand: true,
        }
    }

    /// Builder: set padding as (top, right, bottom, left).
    pub fn pad(mut self, top: usize, right: usize, bottom: usize, left: usize) -> Self {
        self.pad = PaddingDimensions::new(top, right, bottom, left);
        self
    }

    /// Builder: set uniform padding on all sides.
    pub fn pad_all(mut self, pad: usize) -> Self {
        self.pad = PaddingDimensions::all(pad);
        self
    }

    /// Builder: indent by `level` spaces on the left.
    pub fn indent(mut self, level: usize) -> Self {
        self.pad = PaddingDimensions::new(0, 0, 0, level);
        self.expand = false;
        self
    }

    /// Builder: set the style.
    pub fn style(mut self, style: Style) -> Self { self.style = style; self }
}

impl std::fmt::Debug for Padding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Padding")
            .field("pad", &self.pad)
            .finish()
    }
}

impl Renderable for Padding {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let pad = &self.pad;
        let width = if self.expand {
            options.max_width
        } else {
            (crate::measure::Measurement::fixed(0).maximum + pad.left + pad.right)
                .min(options.max_width)
        };

        let inner_width = width.saturating_sub(pad.left + pad.right);
        let inner_opts = options.update_width(inner_width.max(1));

        let inner_height = options.height.map(|h| h.saturating_sub(pad.top + pad.bottom));
        let inner_opts = if let Some(h) = inner_height {
            inner_opts.update_height(h)
        } else {
            inner_opts
        };

        let content = self.renderable.render(&inner_opts);
        let mut lines: Vec<Vec<Segment>> = Vec::new();

        // Top padding
        for _ in 0..pad.top {
            lines.push(vec![
                Segment::new(" ".repeat(width)),
                Segment::line(),
            ]);
        }

        // Content lines with left/right padding
        for content_line in &content.lines {
            let mut line = Vec::new();
            if pad.left > 0 {
                line.push(Segment::new(" ".repeat(pad.left)));
            }
            line.extend(content_line.iter().cloned());
            if pad.right > 0 {
                line.push(Segment::new(" ".repeat(pad.right)));
                line.push(Segment::line());
            }
            lines.push(line);
        }

        // Bottom padding
        for _ in 0..pad.bottom {
            lines.push(vec![
                Segment::new(" ".repeat(width)),
                Segment::line(),
            ]);
        }

        RenderResult { lines, items: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_padding() {
        let p = Padding::new("Hello").pad_all(2);
        let opts = ConsoleOptions::default();
        let result = p.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello"));
    }
}
