//! Columns — render renderables side by side. Equivalent to Rich's `columns.py`.

use crate::align::AlignMethod;
use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{Text, TextPart};

/// Renders a set of renderables in side-by-side columns.
///
/// Columns automatically determines the optimal number of columns based on
/// the available width. Supports equal-width columns, column-first (top-to-bottom)
/// filling, right-to-left ordering, alignment, and an optional title.
#[derive(Clone)]
pub struct Columns {
    /// The list of renderables to display in columns.
    pub renderables: Vec<DynRenderable>,
    /// If true, all columns have equal width.
    pub equal: bool,
    /// If true, expand to fill the available width.
    pub expand: bool,
    /// Padding between columns (in characters).
    pub padding: (usize, usize, usize, usize), // top, right, bottom, left
    /// Optional fixed column width.
    pub width: Option<usize>,
    /// If true, fill columns top-to-bottom rather than left-to-right.
    pub column_first: bool,
    /// If true, start columns from the right side.
    pub right_to_left: bool,
    /// Optional alignment for column content.
    pub align: Option<AlignMethod>,
    /// Optional title displayed above the columns.
    pub title: Option<String>,
    /// Optional title style.
    pub title_style: Option<Style>,
}

impl std::fmt::Debug for Columns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Columns")
            .field("count", &self.renderables.len())
            .field("equal", &self.equal)
            .field("column_first", &self.column_first)
            .field("right_to_left", &self.right_to_left)
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
            padding: (0, 1, 0, 1), // top, right, bottom, left
            width: None,
            column_first: false,
            right_to_left: false,
            align: None,
            title: None,
            title_style: None,
        }
    }

    /// Add a renderable to the column layout.
    pub fn add(&mut self, renderable: impl Renderable + Send + Sync + 'static) {
        self.renderables.push(DynRenderable::new(renderable));
    }

    /// Builder: add a renderable, returning self for chaining.
    pub fn with_renderable(mut self, renderable: impl Renderable + Send + Sync + 'static) -> Self {
        self.add(renderable);
        self
    }

    /// Builder: set uniform padding between columns (in characters).
    pub fn padding(mut self, pad: usize) -> Self {
        self.padding = (0, pad, 0, pad);
        self
    }

    /// Builder: set horizontal and vertical padding (h = left/right, v = top/bottom).
    pub fn padding_hv(mut self, h: usize, v: usize) -> Self {
        self.padding = (v, h, v, h);
        self
    }

    /// Builder: set full padding (top, right, bottom, left).
    pub fn padding_full(mut self, top: usize, right: usize, bottom: usize, left: usize) -> Self {
        self.padding = (top, right, bottom, left);
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

    /// Builder: set a fixed column width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: fill columns top-to-bottom (column-first).
    pub fn column_first(mut self, value: bool) -> Self {
        self.column_first = value;
        self
    }

    /// Builder: start columns from the right side.
    pub fn right_to_left(mut self, value: bool) -> Self {
        self.right_to_left = value;
        self
    }

    /// Builder: set content alignment within columns.
    pub fn align(mut self, align: AlignMethod) -> Self {
        self.align = Some(align);
        self
    }

    /// Builder: set a title displayed above the columns.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Builder: set the title style.
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Determine the optimal number of columns given the available width.
    fn compute_column_count(&self, max_width: usize) -> usize {
        let item_count = self.renderables.len();
        if item_count == 0 {
            return 0;
        }
        if item_count == 1 {
            return 1;
        }

        let (_, right, _, left) = self.padding;
        let width_padding = left.max(right);

        // Measure each renderable
        let widths: Vec<usize> = self
            .renderables
            .iter()
            .map(|r| {
                let opts = ConsoleOptions::default();
                if let Some(m) = r.measure(&opts) {
                    m.maximum
                } else {
                    let result = r.render(&opts);
                    result
                        .lines
                        .iter()
                        .flat_map(|l| l.iter())
                        .map(|s| s.cell_length())
                        .max()
                        .unwrap_or(0)
                }
            })
            .collect();

        let max_item_width = *widths.iter().max().unwrap_or(&0);

        // Try decreasing column counts until all fit
        for col_count in (1..=item_count).rev() {
            let total_padding = width_padding * (col_count.saturating_sub(1)) * 2;
            let available_per_col = max_width.saturating_sub(total_padding) / col_count;
            if available_per_col >= max_item_width || col_count == 1 {
                return col_count;
            }
        }
        1
    }
}

impl Renderable for Columns {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let item_count = self.renderables.len();
        if item_count == 0 {
            return RenderResult::new();
        }

        let (_top, right, _bottom, left) = self.padding;
        let width_padding = left.max(right);
        let max_width = options.max_width;
        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let mut items: Vec<crate::console::RenderItem> = Vec::new();

        // Render title if present
        if let Some(ref title_text) = self.title {
            let style = self.title_style.clone().unwrap_or_default();
            let title_line = Text::assemble(
                vec![TextPart::Styled(title_text.clone(), style)],
                None, None, None, None, None, None,
            );
            items.push(crate::console::RenderItem::Nested(
                DynRenderable::new(title_line),
            ));
            // Add blank line after title
            items.push(crate::console::RenderItem::Segment(Segment::line()));
        }

        // Determine column count and layout
        let col_count = if let Some(fixed_width) = self.width {
            let with_padding = fixed_width + width_padding;
            if with_padding > 0 {
                (max_width / with_padding).max(1)
            } else {
                1
            }
        } else {
            self.compute_column_count(max_width)
        };

        let col_width = if self.width.is_some() {
            self.width.unwrap()
        } else if self.equal {
            // Equal width: use max of all renderable widths
            let max_w = self
                .renderables
                .iter()
                .map(|r| {
                    if let Some(m) = r.measure(options) {
                        m.maximum
                    } else {
                        0
                    }
                })
                .max()
                .unwrap_or(0);
            ((max_width.saturating_sub(width_padding * (col_count.saturating_sub(1)) * 2))
                / col_count)
                .min(max_w)
        } else {
            (max_width.saturating_sub(width_padding * (col_count.saturating_sub(1)) * 2))
                / col_count
        };

        // Build index mapping for column-first or right-to-left ordering
        let indices: Vec<usize> = if self.column_first {
            // Top-to-bottom column filling
            let row_count = (item_count + col_count - 1) / col_count;
            let mut grid = vec![vec![None; col_count]; row_count];
            let (mut r, mut c) = (0usize, 0usize);
            for idx in 0..item_count {
                grid[r][c] = Some(idx);
                r += 1;
                if r >= row_count || grid[r][c].is_some() {
                    r = 0;
                    c += 1;
                }
            }
            grid.into_iter().flatten().filter_map(|x| x).collect()
        } else {
            (0..item_count).collect()
        };

        // Group into rows of col_count items
        for chunk_start in (0..item_count).step_by(col_count) {
            let chunk_end = (chunk_start + col_count).min(item_count);
            let mut row_segments: Vec<Segment> = Vec::new();

            // Get the indices for this row
            let row_indices: Vec<usize> = if self.right_to_left {
                let mut inds: Vec<usize> = indices[chunk_start..chunk_end]
                    .iter()
                    .copied()
                    .collect();
                inds.reverse();
                inds
            } else {
                indices[chunk_start..chunk_end].to_vec()
            };

            for (i, &idx) in row_indices.iter().enumerate() {
                if i > 0 {
                    // Column separator padding
                    row_segments.push(Segment::new(" ".repeat(width_padding * 2)));
                }

                let renderable = &self.renderables[idx];
                let render_opts = options.update_width(col_width.max(1));
                let col_result = renderable.render(&render_opts);

                // Apply alignment if set
                if let Some(align) = self.align {
                    let rendered_width = col_result
                        .lines
                        .first()
                        .map(|l| l.iter().map(|s| s.cell_length()).sum::<usize>())
                        .unwrap_or(0);
                    if rendered_width < col_width {
                        let pad = col_width - rendered_width;
                        match align {
                            AlignMethod::Right => {
                                row_segments.push(Segment::new(" ".repeat(pad)));
                            }
                            AlignMethod::Center => {
                                let left_pad = pad / 2;
                                row_segments.push(Segment::new(" ".repeat(left_pad)));
                            }
                            _ => {}
                        }
                        for line in &col_result.lines {
                            for seg in line {
                                row_segments.push(seg.clone());
                            }
                        }
                        if align == AlignMethod::Right || align == AlignMethod::Center {
                            let remaining = pad - match align {
                                AlignMethod::Center => pad / 2,
                                _ => pad,
                            };
                            if remaining > 0 {
                                row_segments.push(Segment::new(" ".repeat(remaining)));
                            }
                        }
                    } else {
                        for line in &col_result.lines {
                            for seg in line {
                                row_segments.push(seg.clone());
                            }
                        }
                    }
                } else {
                    for line in &col_result.lines {
                        for seg in line {
                            row_segments.push(seg.clone());
                        }
                    }
                }
            }

            if !row_segments.is_empty() {
                row_segments.push(Segment::line());
                lines.push(row_segments);
            }
        }

        RenderResult { lines, items }
    }

    fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
        if self.renderables.is_empty() {
            return Some(Measurement::new(0, 0));
        }

        let (_, right, _, left) = self.padding;
        let width_padding = left.max(right);

        let max_item_width = self
            .renderables
            .iter()
            .map(|r| {
                if let Some(m) = r.measure(options) {
                    m.maximum
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);

        let total = max_item_width + width_padding * 2;
        Some(Measurement::new(total.min(1), total))
    }
}

impl Default for Columns {
    fn default() -> Self {
        Self::new()
    }
}
