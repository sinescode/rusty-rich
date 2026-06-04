//! Columns — render renderables side by side. Equivalent to Rich's `columns.py`.

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::segment::Segment;

/// Renders a set of renderables in side-by-side columns.
#[derive(Clone)]
pub struct Columns {
    pub renderables: Vec<DynRenderable>,
    pub equal: bool,
    pub expand: bool,
    pub padding: usize,
    pub width: Option<usize>,
}

impl std::fmt::Debug for Columns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Columns")
            .field("count", &self.renderables.len())
            .field("equal", &self.equal)
            .finish()
    }
}

impl Columns {
    /// Create an empty [`Columns`] container.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rusty_rich::Columns;
    ///
    /// let mut cols = Columns::new();
    /// cols.add("item 1");
    /// cols.add("item 2");
    /// ```
    pub fn new() -> Self {
        Self {
            renderables: Vec::new(),
            equal: false,
            expand: false,
            padding: 1,
            width: None,
        }
    }

    /// Add a renderable to the column layout.
    pub fn add(&mut self, renderable: impl Renderable + Send + Sync + 'static) {
        self.renderables.push(DynRenderable::new(renderable));
    }

    /// Builder: set padding between columns (in characters).
    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }
    /// Builder: force all columns to have equal width.
    pub fn equal(mut self) -> Self {
        self.equal = true;
        self
    }
    /// Builder: expand columns to fill the available width.
    pub fn expand(mut self) -> Self {
        self.expand = true;
        self
    }
}

impl Renderable for Columns {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let count = self.renderables.len();
        if count == 0 {
            return RenderResult::new();
        }

        let available = self.width.unwrap_or(options.max_width);
        let total_padding = (count.saturating_sub(1)) * self.padding;
        let col_width = if self.equal {
            (available.saturating_sub(total_padding)) / count
        } else {
            available.saturating_sub(total_padding) / count
        };

        // Render each column
        let rendered: Vec<RenderResult> = self
            .renderables
            .iter()
            .map(|r| r.render(&options.update_width(col_width.max(1))))
            .collect();

        // Find max lines
        let max_lines = rendered.iter().map(|r| r.lines.len()).max().unwrap_or(0);

        let mut lines: Vec<Vec<Segment>> = Vec::new();

        for line_idx in 0..max_lines {
            let mut line_segments: Vec<Segment> = Vec::new();

            for (col_idx, col_result) in rendered.iter().enumerate() {
                if col_idx > 0 {
                    line_segments.push(Segment::new(" ".repeat(self.padding)));
                }

                if let Some(col_line) = col_result.lines.get(line_idx) {
                    line_segments.extend(col_line.iter().cloned());
                } else {
                    // Fill with spaces
                    line_segments.push(Segment::new(" ".repeat(col_width)));
                }
            }

            if line_idx < max_lines - 1 {
                line_segments.push(Segment::line());
            }
            lines.push(line_segments);
        }

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

impl Default for Columns {
    fn default() -> Self {
        Self::new()
    }
}
