//! Table — tabular data with columns. Equivalent to Rich's `table.py`.


use crate::align::{AlignMethod, VerticalAlignMethod};
use crate::box_drawing::{get_safe_box, BoxStyle, BOX_HEAVY_HEAD};
use crate::console::{ConsoleOptions, OverflowMethod, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

// ---------------------------------------------------------------------------
// Cell
// ---------------------------------------------------------------------------

/// A single cell in a table row, with optional styling and spanning.
#[derive(Debug, Clone)]
pub struct Cell {
    /// The text content of the cell.
    pub content: String,
    /// Optional per-cell style.
    pub style: Option<Style>,
    /// Number of columns this cell spans (default 1).
    pub colspan: usize,
    /// Number of rows this cell spans (default 1).
    pub rowspan: usize,
}

impl Cell {
    /// Create a new Cell with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Cell {
            content: content.into(),
            style: None,
            colspan: 1,
            rowspan: 1,
        }
    }

    /// Builder: set style.
    pub fn style(mut self, s: Style) -> Self { self.style = Some(s); self }
    /// Builder: set colspan.
    pub fn colspan(mut self, c: usize) -> Self { self.colspan = c; self }
    /// Builder: set rowspan.
    pub fn rowspan(mut self, r: usize) -> Self { self.rowspan = r; self }
}

impl From<String> for Cell {
    fn from(s: String) -> Self { Cell::new(s) }
}

impl From<&str> for Cell {
    fn from(s: &str) -> Self { Cell::new(s) }
}

// ---------------------------------------------------------------------------
// Column
// ---------------------------------------------------------------------------

/// Defines a column within a Table.
#[derive(Debug, Clone)]
pub struct Column {
    /// The header text / renderable.
    pub header: String,
    /// The footer text / renderable.
    pub footer: String,
    /// Header style.
    pub header_style: Style,
    /// Footer style.
    pub footer_style: Style,
    /// Default style for cells in this column.
    pub style: Style,
    /// Horizontal justification.
    pub justify: AlignMethod,
    /// Vertical alignment.
    pub vertical: VerticalAlignMethod,
    /// Overflow method.
    pub overflow: OverflowMethod,
    /// Fixed width, if set.
    pub width: Option<usize>,
    /// Minimum width.
    pub min_width: Option<usize>,
    /// Maximum width.
    pub max_width: Option<usize>,
    /// Ratio weight for flexible distribution.
    pub ratio: Option<usize>,
    /// Number of columns this header spans (default 1).
    pub colspan: usize,
}

impl Column {
    /// Create a new column with the given header.
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            footer: String::new(),
            header_style: Style::new().bold(true),
            footer_style: Style::new(),
            style: Style::new(),
            justify: AlignMethod::Left,
            vertical: VerticalAlignMethod::Top,
            overflow: OverflowMethod::Ellipsis,
            width: None,
            min_width: None,
            max_width: None,
            ratio: None,
            colspan: 1,
        }
    }

    /// Builder: set justify.
    pub fn justify(mut self, j: AlignMethod) -> Self { self.justify = j; self }
    /// Builder: set width.
    pub fn width(mut self, w: usize) -> Self { self.width = Some(w); self }
    /// Builder: set min width.
    pub fn min_width(mut self, w: usize) -> Self { self.min_width = Some(w); self }
    /// Builder: set max width.
    pub fn max_width(mut self, w: usize) -> Self { self.max_width = Some(w); self }
    /// Builder: set style.
    pub fn style(mut self, s: Style) -> Self { self.style = s; self }
    /// Builder: set header style.
    pub fn header_style(mut self, s: Style) -> Self { self.header_style = s; self }
    /// Builder: set ratio.
    pub fn ratio(mut self, r: usize) -> Self { self.ratio = Some(r); self }
    /// Builder: set overflow.
    pub fn overflow(mut self, o: OverflowMethod) -> Self { self.overflow = o; self }
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

/// A renderable for tabular data.
#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<Cell>>,
    /// Title above the table.
    pub title: Option<String>,
    /// Caption below the table.
    pub caption: Option<String>,
    /// Box style.
    pub box_style: BoxStyle,
    /// Show the header row.
    pub show_header: bool,
    /// Show the footer row.
    pub show_footer: bool,
    /// Show outer edge border.
    pub show_edge: bool,
    /// Show lines between every row.
    pub show_lines: bool,
    /// Padding per cell (top, right, bottom, left).
    pub padding: (usize, usize, usize, usize),
    /// Collapse padding between rows.
    pub collapse_padding: bool,
    /// Default style for the table.
    pub style: Style,
    /// Border style.
    pub border_style: Style,
    /// Title style.
    pub title_style: Style,
    /// Caption style.
    pub caption_style: Style,
    /// Title justification.
    pub title_justify: AlignMethod,
    /// Caption justification.
    pub caption_justify: AlignMethod,
    /// If true, highlight cell strings.
    pub highlight: bool,
    /// Optional fixed width.
    pub width: Option<usize>,
    /// Row styles (alternating).
    pub row_styles: Vec<Style>,
    /// Number of blank lines between rows.
    pub leading: usize,
    /// Active rowspan counts per column (tracked during rendering).
    pub rowspans: Vec<usize>,
    /// Row indices that have a section separator before them.
    pub section_rows: HashSet<usize>,
}

impl Table {
    /// Create a new Table.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            caption: None,
            box_style: BOX_HEAVY_HEAD.clone(),
            show_header: true,
            show_footer: false,
            show_edge: true,
            show_lines: false,
            padding: (0, 1, 0, 1),
            collapse_padding: false,
            style: Style::new(),
            border_style: Style::new(),
            title_style: Style::new().bold(true),
            caption_style: Style::new().dim(true),
            title_justify: AlignMethod::Center,
            caption_justify: AlignMethod::Center,
            highlight: false,
            width: None,
            row_styles: Vec::new(),
            leading: 0,
            rowspans: Vec::new(),
            section_rows: HashSet::new(),
        }
    }

    /// Add a column.
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Add a row from Cell objects (supports colspan/rowspan).
    pub fn add_row(&mut self, row: Vec<Cell>) {
        self.rows.push(row);
    }

    /// Add a row from strings (backward-compatible, converts to Cells).
    pub fn add_row_str(&mut self, row: Vec<String>) {
        let cells: Vec<Cell> = row.into_iter().map(Cell::new).collect();
        self.rows.push(cells);
    }

    /// Builder: add a column and return self.
    pub fn column(mut self, col: Column) -> Self { self.add_column(col); self }

    /// Builder: add a row of Cells and return self.
    pub fn row(mut self, row: Vec<Cell>) -> Self { self.add_row(row); self }

    /// Builder: add a row of strings and return self.
    pub fn row_str(mut self, row: Vec<String>) -> Self { self.add_row_str(row); self }

    /// Builder: set title.
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }

    /// Builder: set caption.
    pub fn caption(mut self, t: impl Into<String>) -> Self { self.caption = Some(t.into()); self }

    /// Builder: set box style.
    pub fn box_style(mut self, bs: BoxStyle) -> Self { self.box_style = bs; self }

    /// Builder: set border style.
    pub fn border_style(mut self, s: Style) -> Self { self.border_style = s; self }

    /// Builder: hide the header.
    pub fn hide_header(mut self) -> Self { self.show_header = false; self }

    /// Builder: show lines.
    pub fn show_lines(mut self) -> Self { self.show_lines = true; self }

    /// Builder: set leading (blank lines between rows).
    pub fn leading(mut self, l: usize) -> Self { self.leading = l; self }

    /// Create a grid table (no outer border, no header, no footer).
    /// Equivalent to `Table.grid()`.
    pub fn grid() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            caption: None,
            box_style: crate::box_drawing::BOX_SIMPLE.clone(),
            show_header: false,
            show_footer: false,
            show_edge: false,
            show_lines: false,
            padding: (0, 1, 0, 1),
            collapse_padding: false,
            style: Style::new(),
            border_style: Style::new(),
            title_style: Style::new().bold(true),
            caption_style: Style::new().dim(true),
            title_justify: AlignMethod::Center,
            caption_justify: AlignMethod::Center,
            highlight: false,
            width: None,
            row_styles: Vec::new(),
            leading: 0,
            rowspans: Vec::new(),
            section_rows: HashSet::new(),
        }
    }

    /// Add a section separator before the next row.
    /// The next row added will have a horizontal rule above it.
    pub fn add_section(&mut self) {
        self.section_rows.insert(self.rows.len());
    }

    /// Get the row count.
    pub fn row_count(&self) -> usize { self.rows.len() }
}

impl Renderable for Table {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        if self.columns.is_empty() {
            return RenderResult::new();
        }

        let box_style = get_safe_box(&self.box_style, options.ascii_only);
        let available_width = self.width.unwrap_or(options.max_width);
        let col_count = self.columns.len();

        // Calculate column widths
        let col_widths = self.calculate_column_widths(available_width);

        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let b = &box_style;

        // Helper: make a border segment
        let bs = |ch: char| -> Segment {
            let ansi = self.border_style.to_ansi();
            let reset = if ansi.is_empty() { "" } else { "\x1b[0m" };
            Segment::new(format!("{ansi}{ch}{reset}"))
        };

        // -- Title --
        if let Some(ref title) = self.title {
            let _tw = UnicodeWidthStr::width(title.as_str());
            let centered = self.title_justify.align_text(title, available_width.saturating_sub(2));
            lines.push(vec![bs(b.top_left), Segment::new(&centered[1..centered.len()-1]), bs(b.top_right), Segment::line()]);
        }

        // -- Top border --
        if self.show_edge {
            let mut top_line = vec![bs(b.top_left)];
            for (i, w) in col_widths.iter().enumerate() {
                top_line.push(Segment::new(b.top.to_string().repeat(*w)));
                if i < col_count - 1 {
                    top_line.push(bs(b.top_divider));
                }
            }
            top_line.push(bs(b.top_right));
            top_line.push(Segment::line());
            lines.push(top_line);
        }

        // -- Header --
        if self.show_header && self.columns.iter().any(|c| !c.header.is_empty()) {
            // Top padding
            let (pt, _pr, _pb, _pl) = self.padding;
            for _ in 0..pt {
                lines.push(self.render_row_line(&col_widths, &[], &b, available_width, false));
            }

            let header_cells: Vec<String> = self.columns.iter()
                .map(|c| c.header.clone())
                .collect();
            lines.push(self.render_cell_line(&col_widths, &header_cells, &b, true));

            // Header separator
            let mut sep = vec![bs(b.head_row_left)];
            for (i, w) in col_widths.iter().enumerate() {
                sep.push(Segment::new(b.head_row_horizontal.to_string().repeat(*w)));
                if i < col_count - 1 {
                    sep.push(bs(b.head_row_cross));
                }
            }
            sep.push(bs(b.head_row_right));
            sep.push(Segment::line());
            lines.push(sep);
        }

        // -- Rows --
        let mut rowspan_remaining: Vec<usize> = vec![0; col_count];
        for (row_idx, row) in self.rows.iter().enumerate() {
            // Section separator
            if self.section_rows.contains(&row_idx) {
                let mut sep = vec![bs(b.head_row_left)];
                for (i, w) in col_widths.iter().enumerate() {
                    sep.push(Segment::new(b.head_row_horizontal.to_string().repeat(*w)));
                    if i < col_count - 1 {
                        sep.push(bs(b.head_row_cross));
                    }
                }
                sep.push(bs(b.head_row_right));
                sep.push(Segment::line());
                lines.push(sep);
            }

            // Leading blank lines between rows
            if row_idx > 0 {
                for _ in 0..self.leading {
                    lines.push(self.render_row_line(&col_widths, &[], &b, available_width, false));
                }
            }

            let (pt, _pr, _pb, _pl) = self.padding;
            for _ in 0..pt {
                lines.push(self.render_row_line(&col_widths, &[], &b, available_width, false));
            }

            let _style = if row_idx < self.row_styles.len() {
                Some(&self.row_styles[row_idx])
            } else if self.row_styles.len() == 2 {
                Some(&self.row_styles[row_idx % 2])
            } else {
                None
            };

            lines.push(self.render_cell_line_with_rowspan(
                &col_widths, row, &b, false, &mut rowspan_remaining,
            ));

            // Row separator
            if self.show_lines && row_idx < self.rows.len() - 1 {
                let mut sep = vec![bs(b.row_left)];
                for (i, w) in col_widths.iter().enumerate() {
                    sep.push(Segment::new(b.row_horizontal.to_string().repeat(*w)));
                    if i < col_count - 1 {
                        sep.push(bs(b.row_cross));
                    }
                }
                sep.push(bs(b.row_right));
                sep.push(Segment::line());
                lines.push(sep);
            }
        }

        // -- Footer --
        if self.show_footer && self.columns.iter().any(|c| !c.footer.is_empty()) {
            let mut sep = vec![bs(b.foot_row_left)];
            for (i, w) in col_widths.iter().enumerate() {
                sep.push(Segment::new(b.foot_row_horizontal.to_string().repeat(*w)));
                if i < col_count - 1 {
                    sep.push(bs(b.foot_row_cross));
                }
            }
            sep.push(bs(b.foot_row_right));
            sep.push(Segment::line());
            lines.push(sep);

            let footer_cells: Vec<String> = self.columns.iter()
                .map(|c| c.footer.clone())
                .collect();
            lines.push(self.render_cell_line(&col_widths, &footer_cells, &b, false));
        }

        // -- Bottom border --
        if self.show_edge {
            let mut bot_line = vec![bs(b.bottom_left)];
            for (i, w) in col_widths.iter().enumerate() {
                bot_line.push(Segment::new(b.bottom.to_string().repeat(*w)));
                if i < col_count - 1 {
                    bot_line.push(bs(b.bottom_divider));
                }
            }
            bot_line.push(bs(b.bottom_right));
            bot_line.push(Segment::line());
            lines.push(bot_line);
        }

        // -- Caption --
        if let Some(ref caption) = self.caption {
            let centered = self.caption_justify.align_text(caption, available_width.saturating_sub(2));
            lines.push(vec![Segment::new(&centered), Segment::line()]);
        }

        RenderResult { lines, items: Vec::new() }
    }
}

impl Table {
    fn calculate_column_widths(&self, available: usize) -> Vec<usize> {
        let col_count = self.columns.len();
        let total_pad = col_count.saturating_sub(1) + 2; // separators + edges
        let content_width = available.saturating_sub(total_pad);

        // If any column has a fixed width, respect it
        let mut widths: Vec<usize> = vec![0; col_count];
        let mut flex_cols: Vec<usize> = Vec::new();
        let mut used = 0usize;

        for (i, col) in self.columns.iter().enumerate() {
            if let Some(w) = col.width {
                widths[i] = w;
                used += w;
            } else {
                flex_cols.push(i);
            }
        }

        if flex_cols.is_empty() {
            return widths;
        }

        let remaining = content_width.saturating_sub(used);
        let _flex_count = flex_cols.len();

        // Distribute remaining width using ratios if available
        let total_ratio: usize = flex_cols.iter()
            .map(|&i| self.columns[i].ratio.unwrap_or(1))
            .sum();

        for &i in &flex_cols {
            let col = &self.columns[i];
            let ratio = col.ratio.unwrap_or(1);
            let mut w = (remaining * ratio) / total_ratio;
            if let Some(min_w) = col.min_width {
                w = w.max(min_w);
            }
            if let Some(max_w) = col.max_width {
                w = w.min(max_w);
            }
            w = w.max(3); // minimum usable width
            widths[i] = w;
        }

        // Adjust for rounding
        let total: usize = widths.iter().sum();
        if total < content_width && !flex_cols.is_empty() {
            let extra = content_width - total;
            widths[flex_cols[flex_cols.len() - 1]] += extra;
        }

        widths
    }

    fn render_cell_line(
        &self,
        widths: &[usize],
        values: &[String],
        b: &BoxStyle,
        is_header: bool,
    ) -> Vec<Segment> {
        let mut line = Vec::new();
        let bs = |ch: char| -> Segment {
            let ansi = self.border_style.to_ansi();
            let reset = if ansi.is_empty() { "" } else { "\x1b[0m" };
            Segment::new(format!("{ansi}{ch}{reset}"))
        };

        line.push(bs(b.mid_vertical));

        for (i, w) in widths.iter().enumerate() {
            let val = values.get(i).map(|s| s.as_str()).unwrap_or("");
            let col = self.columns.get(i);
            let justify = col.map(|c| c.justify).unwrap_or(AlignMethod::Left);
            let (_pt, pr, _pb, pl) = self.padding;

            // Pad left
            line.push(Segment::new(" ".repeat(pl)));

            // Align the text
            let disp = justify.align_text(val, w.saturating_sub(pl + pr));
            // Truncate if needed
            let disp_trunc = if UnicodeWidthStr::width(disp.as_str()) > *w {
                let mut truncated = disp.chars().take(
                    w.saturating_sub(1) // leave room for ellipsis
                ).collect::<String>();
                truncated.push('…');
                truncated
            } else {
                disp
            };

            // Apply header style if needed
            if is_header {
                let header_style = col.map(|c| &c.header_style);
                if let Some(hs) = header_style {
                    let ansi = hs.to_ansi();
                    let reset = hs.reset_ansi();
                    line.push(Segment::new(format!("{ansi}{disp_trunc}{reset}")));
                } else {
                    line.push(Segment::new(disp_trunc));
                }
            } else {
                line.push(Segment::new(disp_trunc));
            }

            // Pad right
            line.push(Segment::new(" ".repeat(pr)));

            if i < widths.len() - 1 {
                line.push(bs(b.mid_vertical));
            }
        }

        line.push(bs(b.mid_right));
        line.push(Segment::line());
        line
    }

    /// Render a row of Cells with colspan/rowspan support.
    /// `rowspan_remaining` is updated to track active rowspans.
    fn render_cell_line_with_rowspan(
        &self,
        widths: &[usize],
        cells: &[Cell],
        b: &BoxStyle,
        is_header: bool,
        rowspan_remaining: &mut [usize],
    ) -> Vec<Segment> {
        let mut line = Vec::new();
        let bs = |ch: char| -> Segment {
            let ansi = self.border_style.to_ansi();
            let reset = if ansi.is_empty() { "" } else { "\x1b[0m" };
            Segment::new(format!("{ansi}{ch}{reset}"))
        };

        line.push(bs(b.mid_vertical));

        let col_count = widths.len();
        let mut cell_idx = 0;
        let mut col: usize = 0;

        while col < col_count {
            // Check for active rowspan in this column
            if rowspan_remaining[col] > 0 {
                rowspan_remaining[col] -= 1;
                // Render an empty spanned cell for this column
                let w = widths[col];
                let (_pt, pr, _pb, pl) = self.padding;
                line.push(Segment::new(" ".repeat(pl + w + pr)));
                if col < col_count - 1 {
                    line.push(bs(b.mid_vertical));
                }
                col += 1;
                continue;
            }

            // No more cells — fill remaining columns as empty
            if cell_idx >= cells.len() {
                let w = widths[col];
                let (_pt, pr, _pb, pl) = self.padding;
                line.push(Segment::new(" ".repeat(pl + w + pr)));
                if col < col_count - 1 {
                    line.push(bs(b.mid_vertical));
                }
                col += 1;
                continue;
            }

            let cell = &cells[cell_idx];
            cell_idx += 1;

            let span_end = (col + cell.colspan).min(col_count);
            let span_width: usize = widths[col..span_end].iter().sum();
            let (_pt, pr, _pb, pl) = self.padding;
            let content_width = span_width.saturating_sub(pl + pr);

            let col_def = self.columns.get(col);
            let justify = col_def.map(|c| c.justify).unwrap_or(AlignMethod::Left);

            // Align and truncate content
            let disp_text = justify.align_text(&cell.content, content_width);
            let disp_trunc = if UnicodeWidthStr::width(disp_text.as_str()) > content_width {
                let mut truncated: String = disp_text.chars()
                    .take(content_width.saturating_sub(1))
                    .collect();
                truncated.push('…');
                truncated
            } else {
                disp_text
            };

            // Pad left
            line.push(Segment::new(" ".repeat(pl)));

            // Apply cell style, header style, or column style
            if let Some(ref cell_style) = cell.style {
                let ansi = cell_style.to_ansi();
                let reset = if ansi.is_empty() { "" } else { "\x1b[0m" };
                line.push(Segment::new(format!("{ansi}{disp_trunc}{reset}")));
            } else if is_header {
                if let Some(hs) = col_def.map(|c| &c.header_style) {
                    let ansi = hs.to_ansi();
                    let reset = hs.reset_ansi();
                    line.push(Segment::new(format!("{ansi}{disp_trunc}{reset}")));
                } else {
                    line.push(Segment::new(disp_trunc));
                }
            } else {
                // Apply column default style if it has ANSI
                let col_ansi = col_def.map(|c| c.style.to_ansi()).unwrap_or_default();
                if col_ansi.is_empty() {
                    line.push(Segment::new(disp_trunc));
                } else {
                    line.push(Segment::new(format!("{col_ansi}{disp_trunc}\x1b[0m")));
                }
            }

            // Pad right
            line.push(Segment::new(" ".repeat(pr)));

            // Set rowspan for future rows
            if cell.rowspan > 1 {
                for rc in col..span_end {
                    rowspan_remaining[rc] = cell.rowspan - 1;
                }
            }

            col = span_end;

            // Vertical separator after the span
            if col < col_count {
                line.push(bs(b.mid_vertical));
            }
        }

        line.push(bs(b.mid_right));
        line.push(Segment::line());
        line
    }

    fn render_row_line(
        &self,
        widths: &[usize],
        _values: &[String],
        b: &BoxStyle,
        _available_width: usize,
        _is_header: bool,
    ) -> Vec<Segment> {
        let mut line = Vec::new();
        let bs = |ch: char| -> Segment {
            let ansi = self.border_style.to_ansi();
            let reset = if ansi.is_empty() { "" } else { "\x1b[0m" };
            Segment::new(format!("{ansi}{ch}{reset}"))
        };

        line.push(bs(b.mid_vertical));
        for (i, w) in widths.iter().enumerate() {
            line.push(Segment::new(" ".repeat(*w)));
            if i < widths.len() - 1 {
                line.push(bs(b.mid_vertical));
            }
        }
        line.push(bs(b.mid_right));
        line.push(Segment::line());
        line
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_table() {
        let table = Table::new();
        let opts = ConsoleOptions::default();
        let result = table.render(&opts);
        assert!(result.lines.is_empty());
    }

    #[test]
    fn test_table_with_one_column() {
        let mut table = Table::new();
        table.add_column(Column::new("Name"));
        table.add_row_str(vec!["Alice".into()]);
        table.add_row_str(vec!["Bob".into()]);

        let opts = ConsoleOptions::default();
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Name"));
        assert!(ansi.contains("Alice"));
    }

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new("hello");
        assert_eq!(cell.content, "hello");
        assert_eq!(cell.colspan, 1);
        assert_eq!(cell.rowspan, 1);
        assert!(cell.style.is_none());

        let cell2 = Cell::new("world").colspan(2).rowspan(3);
        assert_eq!(cell2.content, "world");
        assert_eq!(cell2.colspan, 2);
        assert_eq!(cell2.rowspan, 3);
    }

    #[test]
    fn test_cell_from_string() {
        let cell: Cell = "test".into();
        assert_eq!(cell.content, "test");
    }

    #[test]
    fn test_column_colspan() {
        let col = Column::new("Header");
        assert_eq!(col.colspan, 1);
    }

    #[test]
    fn test_add_row_str() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_column(Column::new("B"));
        table.add_row_str(vec!["x".into(), "y".into()]);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn test_add_section() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_row_str(vec!["r1".into()]);
        table.add_section();
        table.add_row_str(vec!["r2".into()]);
        assert_eq!(table.row_count(), 2);
        assert!(table.section_rows.contains(&1));

        let opts = ConsoleOptions::default();
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("r1"));
        assert!(ansi.contains("r2"));
    }

    #[test]
    fn test_leading() {
        let table = Table::new()
            .column(Column::new("X"))
            .row_str(vec!["a".into()])
            .row_str(vec!["b".into()])
            .leading(1);
        assert_eq!(table.leading, 1);
    }

    #[test]
    fn test_cell_rowspan() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_column(Column::new("B"));
        let cell_a = Cell::new("span").rowspan(2);
        let cell_b = Cell::new("single");
        table.add_row(vec![cell_a, cell_b]);
        table.add_row_str(vec!["row2col2".into()]);

        let opts = ConsoleOptions::default();
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("span"));
    }

    #[test]
    fn test_cell_colspan() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_column(Column::new("B"));
        table.add_column(Column::new("C"));
        let cell = Cell::new("wide").colspan(2);
        table.add_row(vec![cell, Cell::new("c")]);
        table.add_row_str(vec!["a".into(), "b".into(), "c".into()]);

        let opts = ConsoleOptions::default();
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("wide"));
    }
}
