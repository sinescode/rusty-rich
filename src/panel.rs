//! Panel — a bordered container. Equivalent to Rich's `panel.py`.

use crate::align::AlignMethod;
use crate::box_drawing::{get_safe_box, BoxStyle, BOX_ROUNDED};
use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// A renderable that draws a border around its contents.
#[derive(Clone)]
pub struct Panel {
    /// The content inside the panel.
    pub renderable: DynRenderable,
    /// The box style defining the border.
    pub box_style: BoxStyle,
    /// Optional title displayed in the top border.
    pub title: Option<String>,
    /// Alignment of the title.
    pub title_align: AlignMethod,
    /// Optional subtitle displayed in the bottom border.
    pub subtitle: Option<String>,
    /// Alignment of the subtitle.
    pub subtitle_align: AlignMethod,
    /// If true, expand to fill available width.
    pub expand: bool,
    /// Style for the content area.
    pub style: Style,
    /// Style for the border.
    pub border_style: Style,
    /// Optional fixed width.
    pub width: Option<usize>,
    /// Optional fixed height.
    pub height: Option<usize>,
    /// Padding (top, right, bottom, left).
    pub padding: (usize, usize, usize, usize),
    /// If true, highlight string titles.
    pub highlight: bool,
}

impl Panel {
    /// Create a new Panel with the given content.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
            box_style: BOX_ROUNDED.clone(),
            title: None,
            title_align: AlignMethod::Center,
            subtitle: None,
            subtitle_align: AlignMethod::Center,
            expand: true,
            style: Style::new(),
            border_style: Style::new(),
            width: None,
            height: None,
            padding: (0, 1, 0, 1), // top, right, bottom, left
            highlight: false,
        }
    }

    /// Builder: set the box style.
    pub fn box_style(mut self, bs: BoxStyle) -> Self {
        self.box_style = bs;
        self
    }

    /// Builder: set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Builder: set the subtitle.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Builder: set the border style.
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Builder: set the content style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder: set width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: set height.
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Builder: set padding.
    pub fn padding(mut self, top: usize, right: usize, bottom: usize, left: usize) -> Self {
        self.padding = (top, right, bottom, left);
        self
    }

    /// Builder: don't expand to fill width.
    pub fn fit(mut self) -> Self {
        self.expand = false;
        self
    }

    /// Builder: set title alignment.
    pub fn title_align(mut self, align: AlignMethod) -> Self {
        self.title_align = align;
        self
    }
}

impl std::fmt::Debug for Panel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Panel")
            .field("title", &self.title)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Renderable for Panel {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let box_style = get_safe_box(&self.box_style, options.ascii_only);
        let padding = self.padding;
        let has_edge = box_style.has_visible_edges();
        // Only reserve space for borders if the box actually draws them.
        let edge_width: usize = if has_edge { 2 } else { 0 };
        let inner_max_width = options
            .max_width
            .saturating_sub(edge_width + padding.1 + padding.3);

        // Render the content
        let inner_options = options.update_width(inner_max_width.max(1));
        let content = self.renderable.render(&inner_options);

        // Calculate content width and height
        let content_width: usize = content
            .lines
            .iter()
            .map(|line| line.iter().map(|s| s.cell_length()).sum::<usize>())
            .max()
            .unwrap_or(0);

        let panel_width = if self.expand {
            options.max_width
        } else {
            (content_width + edge_width + padding.1 + padding.3)
                .min(options.max_width)
                .max(3)
        };

        // Build the panel
        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let border = &box_style;
        let border_ansi = self.border_style.to_ansi();
        let border_reset = if border_ansi.is_empty() {
            ""
        } else {
            "\x1b[0m"
        };

        // Helper: create a border segment
        let bs = |ch: char| -> Segment {
            let text = format!("{border_ansi}{ch}{border_reset}");
            Segment::new(text)
        };

        // -- Edge-less mode: render title/subtitle as plain text, skip borders --
        if !has_edge {
            // Title as plain text
            if let Some(ref title) = self.title {
                let aligned = self.title_align.align_text(title, panel_width);
                lines.push(vec![Segment::new(&aligned), Segment::line()]);
            }
            // Top padding
            for _ in 0..padding.0 {
                lines.push(vec![Segment::new(" ".repeat(panel_width)), Segment::line()]);
            }
            // Content
            for content_line in &content.lines {
                let mut line: Vec<Segment> = Vec::new();
                if padding.3 > 0 {
                    line.push(Segment::new(" ".repeat(padding.3)));
                }
                let available = panel_width.saturating_sub(padding.1 + padding.3);
                let seg_width: usize = content_line.iter().map(|s| s.cell_length()).sum();
                line.extend(content_line.iter().take(seg_width.min(available)).cloned());
                let fill = available.saturating_sub(seg_width);
                if fill > 0 {
                    line.push(Segment::new(" ".repeat(fill)));
                }
                if padding.1 > 0 {
                    line.push(Segment::new(" ".repeat(padding.1)));
                }
                line.push(Segment::line());
                lines.push(line);
            }
            // Bottom padding
            for _ in 0..padding.2 {
                lines.push(vec![Segment::new(" ".repeat(panel_width)), Segment::line()]);
            }
            // Subtitle as plain text
            if let Some(ref subtitle) = self.subtitle {
                let aligned = self.subtitle_align.align_text(subtitle, panel_width);
                lines.push(vec![Segment::new(&aligned), Segment::line()]);
            }
            return RenderResult {
                lines,
                items: Vec::new(),
            };
        }

        // -- Bordered mode (original path) --
        // Top border (with optional title)
        let top_line = self.render_top_border(&box_style, panel_width, border_ansi.as_str(), border_reset);
        lines.push(top_line);

        // Pad top
        for _ in 0..padding.0 {
            let pad_line =
                self.render_pad_line(&box_style, panel_width, border_ansi.as_str(), border_reset);
            lines.push(pad_line);
        }

        // Content lines
        for content_line in &content.lines {
            let mut line: Vec<Segment> = Vec::new();
            // Left border
            line.push(bs(border.mid_left));
            // Left padding
            if padding.3 > 0 {
                line.push(Segment::new(" ".repeat(padding.3)));
            }

            // Content (possibly truncated to fit)
            let available = panel_width.saturating_sub(2 + padding.1 + padding.3);
            let seg_width: usize = content_line.iter().map(|s| s.cell_length()).sum();
            line.extend(content_line.iter().take(seg_width.min(available)).cloned());

            // Fill remaining space
            let fill = available.saturating_sub(seg_width);
            if fill > 0 {
                line.push(Segment::new(" ".repeat(fill)));
            }

            // Right padding
            if padding.1 > 0 {
                line.push(Segment::new(" ".repeat(padding.1)));
            }
            // Right border
            line.push(bs(border.mid_right));
            line.push(Segment::line());
            lines.push(line);
        }

        // Pad bottom
        for _ in 0..padding.2 {
            let pad_line =
                self.render_pad_line(&box_style, panel_width, border_ansi.as_str(), border_reset);
            lines.push(pad_line);
        }

        // Bottom border (with optional subtitle)
        let bottom_line =
            self.render_bottom_border(&box_style, panel_width, border_ansi.as_str(), border_reset);
        lines.push(bottom_line);

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

impl Panel {
    fn render_top_border(
        &self,
        b: &BoxStyle,
        width: usize,
        border_ansi: &str,
        border_reset: &str,
    ) -> Vec<Segment> {
        let mut line = Vec::new();
        let inner = width.saturating_sub(2);

        if let Some(ref title) = self.title {
            let title_w = unicode_width::UnicodeWidthStr::width(title.as_str());
            if title_w + 2 <= inner {
                let rem = inner - title_w - 2;
                let (left_w, right_w) = match self.title_align {
                    AlignMethod::Left => (1, rem - 1),
                    AlignMethod::Right => (rem - 1, 1),
                    AlignMethod::Center => {
                        let l = rem / 2;
                        (l, rem - l)
                    }
                    AlignMethod::Full => (1, rem - 1),
                };

                // Batch repeated horizontal chars under a single ANSI wrap
                let bl = format!("{border_ansi}{}{border_reset}", b.top_left);
                let br = format!("{border_ansi}{}{border_reset}", b.top_right);
                let bt_left = format!(
                    "{border_ansi}{}{border_reset}",
                    b.top.to_string().repeat(left_w)
                );
                let bt_right = format!(
                    "{border_ansi}{}{border_reset}",
                    b.top.to_string().repeat(right_w)
                );

                line.push(Segment::new(bl));
                line.push(Segment::new(bt_left));
                line.push(Segment::new(format!(" {title} ")));
                line.push(Segment::new(bt_right));
                line.push(Segment::new(br));
                line.push(Segment::line());
                return line;
            }
        }

        // No title, or title too long
        let bl = format!("{border_ansi}{}{border_reset}", b.top_left);
        let br = format!("{border_ansi}{}{border_reset}", b.top_right);
        let bt = format!(
            "{border_ansi}{}{border_reset}",
            b.top.to_string().repeat(inner)
        );

        line.push(Segment::new(bl));
        line.push(Segment::new(bt));
        line.push(Segment::new(br));
        line.push(Segment::line());
        line
    }

    fn render_bottom_border(
        &self,
        b: &BoxStyle,
        width: usize,
        border_ansi: &str,
        border_reset: &str,
    ) -> Vec<Segment> {
        let mut line = Vec::new();
        let inner = width.saturating_sub(2);

        if let Some(ref subtitle) = self.subtitle {
            let sub_w = unicode_width::UnicodeWidthStr::width(subtitle.as_str());
            if sub_w + 2 <= inner {
                let rem = inner - sub_w - 2;
                let (left_w, right_w) = match self.subtitle_align {
                    AlignMethod::Left => (1, rem - 1),
                    AlignMethod::Right => (rem - 1, 1),
                    AlignMethod::Center => {
                        let l = rem / 2;
                        (l, rem - l)
                    }
                    AlignMethod::Full => (1, rem - 1),
                };

                let bl = format!("{border_ansi}{}{border_reset}", b.bottom_left);
                let br = format!("{border_ansi}{}{border_reset}", b.bottom_right);
                let bb_left = format!(
                    "{border_ansi}{}{border_reset}",
                    b.bottom.to_string().repeat(left_w)
                );
                let bb_right = format!(
                    "{border_ansi}{}{border_reset}",
                    b.bottom.to_string().repeat(right_w)
                );

                line.push(Segment::new(bl));
                line.push(Segment::new(bb_left));
                line.push(Segment::new(format!(" {subtitle} ")));
                line.push(Segment::new(bb_right));
                line.push(Segment::new(br));
                line.push(Segment::line());
                return line;
            }
        }

        let bl = format!("{border_ansi}{}{border_reset}", b.bottom_left);
        let br = format!("{border_ansi}{}{border_reset}", b.bottom_right);
        let bb = format!(
            "{border_ansi}{}{border_reset}",
            b.bottom.to_string().repeat(inner)
        );

        line.push(Segment::new(bl));
        line.push(Segment::new(bb));
        line.push(Segment::new(br));
        line.push(Segment::line());
        line
    }

    fn render_pad_line(
        &self,
        b: &BoxStyle,
        width: usize,
        border_ansi: &str,
        border_reset: &str,
    ) -> Vec<Segment> {
        let inner = width.saturating_sub(2);
        let left = format!("{border_ansi}{}{border_reset}", b.mid_left);
        let right = format!("{border_ansi}{}{border_reset}", b.mid_right);
        vec![
            Segment::new(left),
            Segment::new(" ".repeat(inner)),
            Segment::new(right),
            Segment::line(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_panel_creation() {
        let panel = Panel::new("Hello");
        assert!(panel.title.is_none());
    }

    #[test]
    fn test_panel_with_title() {
        let panel = Panel::new("Content").title("My Title");
        let opts = ConsoleOptions::default();
        let result = panel.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("My Title"));
    }
}
