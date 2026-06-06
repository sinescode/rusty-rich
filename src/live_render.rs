//! Live render helper — equivalent to Rich's `live_render.py`.
//!
//! [`LiveRender`] manages cursor positioning and vertical overflow for
//! auto-updating terminal regions. It tracks the last-rendered shape
//! (width × height) so it can reposition the cursor to overwrite previous
//! output on each refresh.
//!
//! # Example
//!
//! ```rust,no_run
//! use rusty_rich::live_render::{LiveRender, VerticalOverflow};
//! use rusty_rich::Text;
//!
//! let mut lr = LiveRender::new(Text::new("Loading..."));
//! lr.set_vertical_overflow(VerticalOverflow::Ellipsis);
//! ```

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::control::Control;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// VerticalOverflow
// ---------------------------------------------------------------------------

/// How to handle content that exceeds the available terminal height.
///
/// Equivalent to Python Rich's `VerticalOverflowMethod` literal type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalOverflow {
    /// Crop content at the boundary — lines beyond the limit are discarded.
    Crop,
    /// Crop content and show "…" on the last visible line.
    Ellipsis,
    /// Let content overflow past the region (visible below).
    Visible,
}

// ---------------------------------------------------------------------------
// LiveRender
// ---------------------------------------------------------------------------

/// Tracks the rendering state for a live-updating terminal region.
///
/// `LiveRender` wraps a renderable and remembers the shape of the last
/// render so it can emit the correct cursor-movement control codes to
/// overwrite previous output in-place.
///
/// This is the Rust equivalent of Python Rich's `LiveRender` class.
#[derive(Debug, Clone)]
pub struct LiveRender {
    /// The current renderable being displayed.
    pub renderable: DynRenderable,
    /// Optional style to apply to all output.
    pub style: Style,
    /// How to handle vertical overflow.
    pub vertical_overflow: VerticalOverflow,
    /// The shape of the last render: `(width, height)` in cells.
    /// `None` if nothing has been rendered yet. Updated by [`Live`](crate::Live)
    /// after each refresh via [`set_shape`](Self::set_shape).
    last_shape: Option<(usize, usize)>,
}

impl LiveRender {
    /// Create a new `LiveRender` wrapping the given renderable.
    ///
    /// Defaults to [`VerticalOverflow::Ellipsis`] and no additional style.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
            style: Style::new(),
            vertical_overflow: VerticalOverflow::Ellipsis,
            last_shape: None,
        }
    }

    /// Update the last-rendered shape. Called by [`Live`](crate::Live) after
    /// each refresh so cursor-control methods can reposition correctly.
    pub fn set_shape(&mut self, shape: Option<(usize, usize)>) {
        self.last_shape = shape;
    }

    /// Builder: set the style applied to the live region.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the vertical overflow method.
    pub fn vertical_overflow(mut self, method: VerticalOverflow) -> Self {
        self.vertical_overflow = method;
        self
    }

    /// Replace the displayed renderable.
    pub fn set_renderable(&mut self, renderable: impl Renderable + Send + Sync + 'static) {
        self.renderable = DynRenderable::new(renderable);
    }

    /// Set the vertical overflow method.
    pub fn set_vertical_overflow(&mut self, method: VerticalOverflow) {
        self.vertical_overflow = method;
    }

    /// Return the height of the last render (0 if nothing rendered yet).
    pub fn last_render_height(&self) -> usize {
        self.last_shape.map(|(_, h)| h).unwrap_or(0)
    }

    /// Return the last rendered shape as `(width, height)`, if any.
    pub fn last_shape(&self) -> Option<(usize, usize)> {
        self.last_shape
    }

    // ------------------------------------------------------------------
    // Cursor control helpers
    // ------------------------------------------------------------------

    /// Produce a [`Control`] that moves the cursor to the beginning of the
    /// live region, ready to overwrite the previous output.
    ///
    /// This is called before rendering new content during a refresh.
    ///
    /// Equivalent to Python Rich's `LiveRender.position_cursor()` which emits
    /// `(CURSOR_UP, ERASE_IN_LINE) * (height - 1)` pairs.
    pub fn position_cursor(&self) -> Control {
        if let Some((_, height)) = self.last_shape {
            if height > 1 {
                // Build: CARRIAGE_RETURN then (CURSOR_UP, ERASE_LINE) × (height-1)
                // Python Rich does exactly this sequence
                let mut seqs: Vec<String> = Vec::with_capacity(1 + (height - 1) * 2);
                seqs.push(Control::carriage_return().to_ansi());
                for _ in 1..height {
                    seqs.push(Control::cursor_up(1).to_ansi());
                    seqs.push(Control::erase_line().to_ansi());
                }
                let refs: Vec<&str> = seqs.iter().map(|s| s.as_str()).collect();
                return Control::new(&refs);
            } else if height == 1 {
                return Control::carriage_return();
            }
        }
        Control::new(&[])
    }

    /// Produce a [`Control`] that clears the rendered region and restores
    /// the cursor to its position before the live display started.
    ///
    /// This is used when stopping a transient live display — the output
    /// disappears as if it was never there.
    ///
    /// Equivalent to Python Rich's `LiveRender.restore_cursor()` which emits
    /// `CARRIAGE_RETURN, (CURSOR_UP, ERASE_IN_LINE) * height`.
    pub fn restore_cursor(&self) -> Control {
        if let Some((_, height)) = self.last_shape {
            if height > 0 {
                let mut seqs: Vec<String> = Vec::with_capacity(1 + height * 2);
                seqs.push(Control::carriage_return().to_ansi());
                for _ in 0..height {
                    seqs.push(Control::cursor_up(1).to_ansi());
                    seqs.push(Control::erase_line().to_ansi());
                }
                let refs: Vec<&str> = seqs.iter().map(|s| s.as_str()).collect();
                return Control::new(&refs);
            }
        }
        Control::new(&[])
    }
}

// ---------------------------------------------------------------------------
// Renderable impl
// ---------------------------------------------------------------------------

impl Renderable for LiveRender {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let result = self.renderable.render(options);
        let mut lines = result.lines;

        // Compute the shape from the rendered lines
        let width: usize = lines
            .iter()
            .map(|line| line.iter().map(|s| s.cell_length()).sum::<usize>())
            .max()
            .unwrap_or(0);
        let height = lines.len();

        // Handle vertical overflow
        let max_height = options.max_height;
        if height > max_height && max_height > 0 {
            match self.vertical_overflow {
                VerticalOverflow::Crop => {
                    lines.truncate(max_height);
                }
                VerticalOverflow::Ellipsis => {
                    if max_height > 0 {
                        lines.truncate(max_height.saturating_sub(1));
                        // Append ellipsis line
                        let ellipsis_text = Text::new("...")
                            .style(Style::new().bold(true).color(
                                crate::color::Color::parse("red").unwrap_or_default(),
                            ))
                            .justify(crate::align::AlignMethod::Center);
                        let ellipsis_result = ellipsis_text.render(options);
                        if let Some(ellipsis_line) = ellipsis_result.lines.into_iter().next() {
                            lines.push(ellipsis_line);
                        }
                    }
                }
                VerticalOverflow::Visible => {
                    // Don't truncate — let content overflow
                }
            }
        }

        // Shape will be stored by the caller (Live) after rendering, so
        // cursor-control methods can reference it on subsequent refreshes.
        let mut items = result.items;
        let new_line = Segment::line();

        // Add newlines between lines (matching Python Rich behavior)
        let mut out_lines: Vec<Vec<Segment>> = Vec::with_capacity(lines.len());
        for (i, line) in lines.into_iter().enumerate() {
            out_lines.push(line);
            if i + 1 < height {
                out_lines.push(vec![new_line.clone()]);
            }
        }

        RenderResult {
            lines: out_lines,
            items,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Text;

    #[test]
    fn test_vertical_overflow_enum() {
        assert_eq!(VerticalOverflow::Crop, VerticalOverflow::Crop);
        assert_ne!(VerticalOverflow::Crop, VerticalOverflow::Ellipsis);
        assert_ne!(VerticalOverflow::Ellipsis, VerticalOverflow::Visible);
    }

    #[test]
    fn test_live_render_new() {
        let lr = LiveRender::new(Text::new("hello"));
        assert_eq!(lr.last_render_height(), 0);
        assert!(lr.last_shape().is_none());
        assert_eq!(lr.vertical_overflow, VerticalOverflow::Ellipsis);
    }

    #[test]
    fn test_live_render_builder() {
        let lr = LiveRender::new(Text::new("test"))
            .style(Style::new().bold(true))
            .vertical_overflow(VerticalOverflow::Crop);
        assert_eq!(lr.vertical_overflow, VerticalOverflow::Crop);
    }

    #[test]
    fn test_set_renderable() {
        let mut lr = LiveRender::new(Text::new("first"));
        lr.set_renderable(Text::new("second"));
        // Verify it doesn't panic and the renderable was replaced
        let opts = ConsoleOptions::default();
        let result = lr.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("second"));
    }

    #[test]
    fn test_position_cursor_no_render() {
        let lr = LiveRender::new(Text::new("unrendered"));
        let ctrl = lr.position_cursor();
        // No previous shape → empty control
        assert!(ctrl.to_ansi().is_empty());
    }

    #[test]
    fn test_restore_cursor_no_render() {
        let lr = LiveRender::new(Text::new("unrendered"));
        let ctrl = lr.restore_cursor();
        assert!(ctrl.to_ansi().is_empty());
    }

    #[test]
    fn test_vertical_overflow_crop() {
        let lr = LiveRender::new(Text::new("line1\nline2\nline3\nline4\nline5"))
            .vertical_overflow(VerticalOverflow::Crop);
        let opts = ConsoleOptions {
            max_height: 3,
            ..ConsoleOptions::default()
        };
        let result = lr.render(&opts);
        // Should be cropped to at most 3 content lines
        let line_count = result
            .lines
            .iter()
            .filter(|l| l.iter().any(|s| !s.text.trim().is_empty()))
            .count();
        assert!(line_count <= 3);
    }

    #[test]
    fn test_vertical_overflow_visible() {
        let lr = LiveRender::new(Text::new("line1\nline2\nline3\nline4\nline5\nline6"))
            .vertical_overflow(VerticalOverflow::Visible);
        let opts = ConsoleOptions {
            max_height: 3,
            ..ConsoleOptions::default()
        };
        let result = lr.render(&opts);
        // Visible mode should NOT crop
        let line_count = result
            .lines
            .iter()
            .filter(|l| l.iter().any(|s| !s.text.trim().is_empty()))
            .count();
        assert!(line_count > 3);
    }
}
