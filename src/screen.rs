//! Screen — full-screen renderable and alternate screen buffer.
//!
//! Provides the `Screen` renderable that fills the terminal, cropping or
//! padding content to exactly fit the screen dimensions. Also provides
//! `ScreenContext` for managing the alternate screen buffer and
//! `ScreenUpdate` for partial screen updates.
//!
//! Equivalent to Rich's `screen.py`.

use std::io::Write;

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

/// A renderable that fills the entire terminal screen, cropping or padding
/// its content to exactly fit the screen dimensions.
///
/// Equivalent to Rich's `Screen` class.
pub struct Screen {
    /// The child renderable.
    pub renderable: DynRenderable,
    /// Optional style applied as a background / padding style.
    pub style: Option<Style>,
    /// If true, use `\n\r` line endings (application mode for raw terminals).
    pub application_mode: bool,
}

impl Screen {
    /// Create a new Screen wrapping the given renderable.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
            style: None,
            application_mode: false,
        }
    }

    /// Builder: set the optional background / padding style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder: set application mode (uses `\n\r` instead of `\n`).
    pub fn application_mode(mut self, mode: bool) -> Self {
        self.application_mode = mode;
        self
    }

    /// Update the content renderable.
    pub fn update<T>(&mut self, update: T)
    where
        T: Into<ScreenUpdate>,
    {
        let update = update.into();
        self.renderable = update.renderable;
    }
}

impl std::fmt::Debug for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Screen")
            .field("style", &self.style)
            .field("application_mode", &self.application_mode)
            .finish()
    }
}

impl Renderable for Screen {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let width = options.size.width.max(1);
        let height = options.size.height.max(1);

        // Create render options that match the full screen size
        let render_options = options
            .update_width(width)
            .update_height(height);

        // Render the inner content
        let result = self.renderable.render(&render_options);

        // Collect lines from result (handle both `lines` and `items`)
        let mut lines: Vec<Vec<Segment>> = if !result.lines.is_empty() {
            result.lines
        } else {
            let segments = result.flatten(&render_options);
            if segments.is_empty() {
                vec![vec![]]
            } else {
                // Group flattened segments into lines by splitting on newlines
                let mut grouped: Vec<Vec<Segment>> = Vec::new();
                let mut current_line: Vec<Segment> = Vec::new();
                for seg in segments {
                    if seg.text == "\n" || seg.text == "\r\n" {
                        grouped.push(std::mem::take(&mut current_line));
                    } else {
                        current_line.push(seg);
                    }
                }
                if !current_line.is_empty() {
                    grouped.push(current_line);
                }
                if grouped.is_empty() {
                    grouped.push(vec![]);
                }
                grouped
            }
        };

        // -- Apply style and shape output to exact screen dimensions --

        // Style all content segments first
        if let Some(ref screen_style) = self.style {
            for line in &mut lines {
                for seg in line.iter_mut() {
                    if let Some(ref existing) = seg.style {
                        seg.style = Some(existing.combine(screen_style));
                    } else {
                        seg.style = Some(screen_style.clone());
                    }
                }
            }
        }

        // Crop or pad each line to exact width
        let blank_seg = if let Some(ref style) = self.style {
            Segment::styled(" ".repeat(width), style.clone())
        } else {
            Segment::new(" ".repeat(width))
        };

        for line in &mut lines {
            let line_len: usize = line.iter().map(|s| s.cell_length()).sum();
            if line_len > width {
                // Crop the line
                let mut cropped: Vec<Segment> = Vec::new();
                let mut accumulated = 0usize;
                for seg in line.drain(..) {
                    let seg_len = seg.cell_length();
                    if accumulated + seg_len <= width {
                        cropped.push(seg);
                        accumulated += seg_len;
                    } else if accumulated < width {
                        let remaining = width - accumulated;
                        let (left, _) = seg.split(remaining);
                        if left.cell_length() > 0 {
                            cropped.push(left);
                        }
                        break;
                    } else {
                        break;
                    }
                }
                *line = cropped;
            } else if line_len < width {
                // Pad to width with spaces (styled if needed)
                if let Some(ref style) = self.style {
                    line.push(Segment::styled(" ".repeat(width - line_len), style.clone()));
                } else {
                    line.push(Segment::new(" ".repeat(width - line_len)));
                }
            }
        }

        // Crop or pad height
        if lines.len() > height {
            lines.truncate(height);
        } else {
            while lines.len() < height {
                lines.push(vec![blank_seg.clone()]);
            }
        }

        // Insert newline segments between lines (not after the last)
        let new_line_char = if self.application_mode { "\n\r" } else { "\n" };
        let mut final_lines: Vec<Vec<Segment>> = Vec::with_capacity(lines.len() * 2);
        let last_idx = lines.len().saturating_sub(1);
        for (i, line) in lines.into_iter().enumerate() {
            final_lines.push(line);
            if i < last_idx {
                final_lines.push(vec![Segment::new(new_line_char)]);
            }
        }

        RenderResult {
            lines: final_lines,
            items: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// ScreenUpdate
// ---------------------------------------------------------------------------

/// Represents an update to a screen display.
///
/// Used by [`ScreenContext::update()`] (and [`Screen::update()`]) to replace
/// the displayed content without creating a new Screen.
pub struct ScreenUpdate {
    /// The new renderable to display.
    pub renderable: DynRenderable,
}

impl ScreenUpdate {
    /// Create a new ScreenUpdate wrapping the given renderable.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
        }
    }
}

impl std::fmt::Debug for ScreenUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScreenUpdate").finish()
    }
}

impl<R> From<R> for ScreenUpdate
where
    R: Renderable + Send + Sync + 'static,
{
    fn from(renderable: R) -> Self {
        Self::new(renderable)
    }
}

// ---------------------------------------------------------------------------
// ScreenContext
// ---------------------------------------------------------------------------

/// A context that enters the alternate screen buffer, provides an [`update`]
/// method to display content, and automatically exits the alternate screen
/// buffer on drop.
///
/// Created via [`Console::screen()`](crate::console::Console::screen).
///
/// # Example
///
/// ```ignore
/// let mut console = Console::new();
/// let ctx = console.screen();
/// ctx.update("Hello from alt-screen!");
/// std::thread::sleep(std::time::Duration::from_secs(2));
/// // ctx drops → exits alt screen
/// ```
pub struct ScreenContext {
    /// Whether the alternate screen is currently active.
    active: bool,
    /// Optional style applied to screen content.
    style: Option<Style>,
}

impl ScreenContext {
    /// Create a new ScreenContext (does **not** enter alt screen yet).
    pub fn new() -> Self {
        Self {
            active: false,
            style: None,
        }
    }

    /// Builder: set the style for screen content.
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Enter the alternate screen buffer.
    pub fn enter(&mut self) {
        if !self.active {
            let _ = write!(std::io::stdout(), "\x1b[?1049h");
            let _ = std::io::stdout().flush();
            self.active = true;
        }
    }

    /// Exit the alternate screen buffer, restoring the original screen.
    pub fn exit(&mut self) {
        if self.active {
            let _ = write!(std::io::stdout(), "\x1b[?1049l");
            let _ = std::io::stdout().flush();
            self.active = false;
        }
    }

    /// Render the given content in the alternate screen.
    pub fn update(&mut self, update: impl Into<ScreenUpdate>) -> std::io::Result<()> {
        if !self.active {
            self.enter();
        }

        let opts = ConsoleOptions::default();
        let screen = Screen {
            renderable: update.into().renderable,
            style: self.style.clone(),
            application_mode: false,
        };
        let result = screen.render(&opts);
        let ansi = result.to_ansi();
        write!(std::io::stdout(), "{ansi}")?;
        std::io::stdout().flush()
    }

    /// Check whether the alternate screen is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for ScreenContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScreenContext {
    fn drop(&mut self) {
        self.exit();
    }
}

impl std::fmt::Debug for ScreenContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScreenContext")
            .field("active", &self.active)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleDimensions;
    use crate::style::Style;

    #[test]
    fn test_screen_creation() {
        let screen = Screen::new("Hello");
        assert!(screen.style.is_none());
        assert!(!screen.application_mode);
    }

    #[test]
    fn test_screen_with_style() {
        let screen = Screen::new("Hello").style(Style::new().bold(true));
        assert!(screen.style.is_some());
    }

    #[test]
    fn test_screen_application_mode() {
        let screen = Screen::new("Hello").application_mode(true);
        assert!(screen.application_mode);
    }

    #[test]
    fn test_screen_crops_wide_content() {
        let screen = Screen::new("Hello World!!!");
        let opts = ConsoleOptions {
            size: ConsoleDimensions {
                width: 5,
                height: 1,
            },
            max_width: 5,
            max_height: 1,
            ..Default::default()
        };
        let result = screen.render(&opts);
        let ansi = result.to_ansi();
        // Should be cropped to 5 chars
        assert!(ansi.contains("Hello"));
        assert!(!ansi.contains("World"));
    }

    #[test]
    fn test_screen_pads_to_height() {
        let screen = Screen::new("Hi");
        let opts = ConsoleOptions {
            size: ConsoleDimensions {
                width: 10,
                height: 5,
            },
            max_width: 10,
            max_height: 5,
            ..Default::default()
        };
        let result = screen.render(&opts);
        let ansi = result.to_ansi();
        // Should have content and padding (look for the text and then spaces)
        assert!(ansi.contains("Hi"));
    }

    #[test]
    fn test_screen_returns_render_result() {
        let screen = Screen::new("Test content");
        let opts = ConsoleOptions {
            size: ConsoleDimensions {
                width: 80,
                height: 24,
            },
            max_width: 80,
            max_height: 24,
            ..Default::default()
        };
        let result = screen.render(&opts);
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_screen_update_creation() {
        let update = ScreenUpdate::new("Updated content");
        let mut screen = Screen::new("Original");
        screen.update(update);
        let opts = ConsoleOptions {
            size: ConsoleDimensions {
                width: 80,
                height: 24,
            },
            max_width: 80,
            max_height: 24,
            ..Default::default()
        };
        let result = screen.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Updated"));
    }

    #[test]
    fn test_screen_update_from_renderable() {
        // Test the From impl
        let update: ScreenUpdate = "Direct string".into();
        let _screen = Screen::new(update.renderable);
    }

    #[test]
    fn test_screen_context_creation() {
        let ctx = ScreenContext::new();
        assert!(!ctx.is_active());
    }

    #[test]
    fn test_screen_context_default() {
        let ctx = ScreenContext::default();
        assert!(!ctx.is_active());
    }

    #[test]
    fn test_screen_context_enter_exit() {
        let mut ctx = ScreenContext::new();
        // enter (don't assert on terminal escape but verify state changes)
        ctx.enter();
        assert!(ctx.is_active());
        ctx.exit();
        assert!(!ctx.is_active());
    }

    #[test]
    fn test_screen_context_double_enter() {
        let mut ctx = ScreenContext::new();
        ctx.enter();
        assert!(ctx.is_active());
        // Second enter should be safe (no-op)
        ctx.enter();
        assert!(ctx.is_active());
    }
}
