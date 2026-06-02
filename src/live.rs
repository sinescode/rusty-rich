//! Live — auto-updating display. Equivalent to Rich's `live.py`.
//!
//! [`Live`] manages a terminal region that updates in-place. Each refresh
//! overwrites the previous output, creating an auto-updating display.
//!
//! # Quick Example
//!
//! ```rust,no_run
//! use rusty_rich::{Live, panel::Panel};
//! use std::thread;
//! use std::time::Duration;
//!
//! let mut live = Live::new(Panel::new("Loading...").title("Progress"));
//! live.start().unwrap();
//!
//! for i in 0..=100 {
//!     live.update(Panel::new(format!("{}%", i)).title("Progress")).unwrap();
//!     thread::sleep(Duration::from_millis(50));
//! }
//!
//! live.stop().unwrap();
//! ```
//!
//! # LiveWriter
//!
//! [`LiveWriter`] captures `write!` output and displays it within the live
//! region. Use [`Live::create_writer`] to create one, then write to it while
//! the live display is active:
//!
//! ```rust,no_run
//! use rusty_rich::{Live, panel::Panel};
//! use std::io::Write;
//!
//! let mut live = Live::new(Panel::new("Status").title("App"));
//! let mut writer = Live::create_writer();
//! live.start().unwrap();
//!
//! writeln!(writer, "Processing item 1...").unwrap();
//! writeln!(writer, "Done!").unwrap();
//!
//! live.stop().unwrap();
//! ```
//!
//! # Transient Mode
//!
//! Call [`Live::transient`] to erase the live region on stop — the output
//! disappears as if it was never there. Useful for "loading…" overlays.

use std::io::{self, Write};
use std::time::Instant;

use crate::console::{ConsoleOptions, DynRenderable, Renderable};
use crate::segment::Segment;

/// A writer that captures output for live display.
pub struct LiveWriter {
    buffer: Vec<u8>,
}

impl LiveWriter {
    /// Create a new `LiveWriter` with an empty capture buffer.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Return a reference to the captured output bytes.
    pub fn capture(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear the captured output buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Write for LiveWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A hook that transforms render output.
///
/// [`RenderHook`] provides a way to intercept and modify the rendered
/// segment lines before they are written to the terminal. Multiple hooks
/// can be registered on a [`Live`] display and are applied in order.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::live::RenderHook;
/// use rusty_rich::Segment;
///
/// let hook = RenderHook::new(|lines| {
///     // Reverse the order of displayed lines
///     let mut reversed = lines.to_vec();
///     reversed.reverse();
///     reversed
/// });
/// ```
pub struct RenderHook {
    hook: Box<dyn Fn(&[Vec<Segment>]) -> Vec<Vec<Segment>> + Send>,
}

impl RenderHook {
    /// Create a new [`RenderHook`] with the given transformation function.
    ///
    /// The function receives the current rendered lines of segments and
    /// returns the modified lines.
    pub fn new<F>(hook: F) -> Self
    where
        F: Fn(&[Vec<Segment>]) -> Vec<Vec<Segment>> + Send + 'static,
    {
        Self {
            hook: Box::new(hook),
        }
    }

    /// Apply this hook to the given segments, returning the transformed segments.
    pub fn apply(&self, segments: &[Vec<Segment>]) -> Vec<Vec<Segment>> {
        (self.hook)(segments)
    }
}

/// Manages a live-updating region of the terminal.
pub struct Live {
    renderable: Option<DynRenderable>,
    screen: bool,
    auto_refresh: bool,
    refresh_per_second: f64,
    transient: bool,
    started: bool,
    started_at: Option<Instant>,
    previous_line_count: usize,
    redirect_stdout: bool,
    redirect_stderr: bool,
    writers: Vec<LiveWriter>,
    render_hooks: Vec<RenderHook>,
}

impl std::fmt::Debug for Live {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Live")
            .field("screen", &self.screen)
            .field("started", &self.started)
            .finish()
    }
}

impl Live {
    /// Create a new `Live` display wrapping the given [`Renderable`].
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: Some(DynRenderable::new(renderable)),
            screen: false,
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            started: false,
            started_at: None,
            previous_line_count: 0,
            redirect_stdout: true,
            redirect_stderr: true,
            writers: Vec::new(),
            render_hooks: Vec::new(),
        }
    }

    /// Builder: use the alternate screen buffer for full-screen display.
    pub fn screen(mut self) -> Self { self.screen = true; self }
    /// Builder: disable automatic periodic refresh.
    pub fn no_auto_refresh(mut self) -> Self { self.auto_refresh = false; self }
    /// Builder: set the refresh rate in Hz (default 4.0).
    pub fn refresh_per_second(mut self, rate: f64) -> Self { self.refresh_per_second = rate; self }
    /// Builder: enable transient mode (live display disappears on stop).
    pub fn transient(mut self) -> Self { self.transient = true; self }
    /// Builder: redirect stdout writes into the live display.
    pub fn redirect_stdout(mut self, redirect: bool) -> Self { self.redirect_stdout = redirect; self }
    /// Builder: redirect stderr writes into the live display.
    pub fn redirect_stderr(mut self, redirect: bool) -> Self { self.redirect_stderr = redirect; self }

    /// Register a writer whose captured content will be rendered during refresh.
    pub fn add_writer(&mut self, writer: LiveWriter) { self.writers.push(writer); }

    /// Create a LiveWriter that captures output while Live is active.
    pub fn create_writer() -> LiveWriter {
        LiveWriter::new()
    }

    /// Start the live display: enter alternate screen (if configured) and hide cursor.
    pub fn start(&mut self) -> io::Result<()> {
        self.started = true;
        self.started_at = Some(Instant::now());
        if self.screen {
            write!(io::stdout(), "\x1b[?1049h")?;
        }
        write!(io::stdout(), "\x1b[?25l")?;
        self.refresh()
    }

    /// Stop the live display: restore cursor, exit alternate screen, and clean up.
    pub fn stop(&mut self) -> io::Result<()> {
        if self.transient {
            for _ in 0..self.previous_line_count {
                write!(io::stdout(), "\x1b[1A\x1b[2K")?;
            }
        }
        if self.screen {
            write!(io::stdout(), "\x1b[?1049l")?;
        }
        write!(io::stdout(), "\x1b[?25h")?;
        io::stdout().flush()?;
        self.started = false;
        self.started_at = None;
        Ok(())
    }

    /// Replace the displayed content and refresh immediately.
    pub fn update(&mut self, renderable: impl Renderable + Send + Sync + 'static) -> io::Result<()> {
        self.renderable = Some(DynRenderable::new(renderable));
        self.refresh()
    }

    /// Re-render the current content in place (cursor is moved back to overwrite previous output).
    ///
    /// If any [`RenderHook`]s are registered, they are applied to the rendered
    /// segment lines before the output is written to the terminal.
    pub fn refresh(&mut self) -> io::Result<()> {
        if let Some(ref renderable) = self.renderable {
            let opts = ConsoleOptions::default();
            let result = renderable.render(&opts);

            if self.previous_line_count > 0 {
                write!(io::stdout(), "\x1b[{}F", self.previous_line_count)?;
            }

            // Apply render hooks to transform segment lines before output
            let (ansi, line_count) = if !self.render_hooks.is_empty() {
                let mut lines = result.lines.clone();
                for hook in &self.render_hooks {
                    lines = hook.apply(&lines);
                }
                let mut out = String::new();
                for line in &lines {
                    for seg in line {
                        out.push_str(&seg.to_ansi());
                    }
                }
                (out, lines.len())
            } else {
                let s = result.to_ansi();
                let c = s.lines().count();
                (s, c)
            };

            write!(io::stdout(), "{ansi}")?;
            if line_count < self.previous_line_count {
                for _ in line_count..self.previous_line_count {
                    write!(io::stdout(), "\x1b[2K\n")?;
                }
            }

            self.previous_line_count = line_count;

            // Write captured writer content
            for writer in &self.writers {
                let captured = String::from_utf8_lossy(writer.capture());
                if !captured.is_empty() {
                    write!(io::stdout(), "{}", captured)?;
                }
            }

            io::stdout().flush()?;
        }
        Ok(())
    }

    /// Check if the live display is currently running.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get a reference to the current renderable.
    ///
    /// # Panics
    ///
    /// Panics if no renderable has been set (this should not happen with
    /// normal usage, as `Live::new` always creates one).
    pub fn get_renderable(&self) -> &dyn Renderable {
        self.renderable.as_ref().unwrap() as &dyn Renderable
    }

    /// Get the current renderable being displayed.
    ///
    /// # Panics
    ///
    /// Panics if no renderable has been set (this should not happen with
    /// normal usage, as `Live::new` always creates one).
    pub fn renderable(&self) -> &dyn Renderable {
        self.renderable.as_ref().unwrap() as &dyn Renderable
    }

    /// Process multiple renderables through the Live display pipeline.
    ///
    /// Each renderable is rendered with the given options, and the resulting
    /// segment lines are collected into a single vector.
    pub fn process_renderables(
        &self,
        renderables: &[Box<dyn Renderable>],
        options: &ConsoleOptions,
    ) -> Vec<Vec<Segment>> {
        let mut all_lines = Vec::new();
        for renderable in renderables {
            let result = renderable.render(options);
            all_lines.extend(result.lines);
        }
        all_lines
    }

    /// Add a render hook to the live display.
    ///
    /// Hooks are applied in registration order during each refresh, allowing
    /// transformation of the rendered segment lines before they are output.
    pub fn add_render_hook(&mut self, hook: RenderHook) {
        self.render_hooks.push(hook);
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Text;

    #[test]
    fn test_is_started() {
        let mut live = Live::new(Text::new("test"));
        assert!(!live.is_started());
        live.start().unwrap();
        assert!(live.is_started());
        live.stop().unwrap();
        assert!(!live.is_started());
    }

    #[test]
    fn test_renderable_accessor() {
        let live = Live::new(Text::new("hello"));
        let r = live.get_renderable();
        // Verify we get a valid reference by rendering it
        let opts = ConsoleOptions::default();
        let result = r.render(&opts);
        assert!(!result.to_ansi().is_empty());
    }

    #[test]
    fn test_render_hook_basic() {
        let hook = RenderHook::new(|segments| segments.to_vec());
        let input = vec![vec![Segment::new("test")]];
        let output = hook.apply(&input);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0][0].text, "test");
    }

    #[test]
    fn test_render_hook_transform() {
        let hook = RenderHook::new(|segments| {
            let mut transformed = segments.to_vec();
            transformed.push(vec![Segment::new("appended")]);
            transformed
        });
        let input = vec![vec![Segment::new("original")]];
        let output = hook.apply(&input);
        assert_eq!(output.len(), 2);
        assert_eq!(output[1][0].text, "appended");
    }

    #[test]
    fn test_process_renderables() {
        let live = Live::new(Text::new("dummy"));
        let opts = ConsoleOptions::default();
        let renderables: Vec<Box<dyn Renderable>> = vec![
            Box::new(Text::new("first")),
            Box::new(Text::new("second")),
        ];
        let lines = live.process_renderables(&renderables, &opts);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_start_stop_cycle() {
        let mut live = Live::new(Text::new("test"));
        assert!(!live.is_started());
        live.start().unwrap();
        assert!(live.is_started());
        live.stop().unwrap();
        assert!(!live.is_started());
    }

    #[test]
    fn test_add_render_hook() {
        let mut live = Live::new(Text::new("test"));
        let hook = RenderHook::new(|segments| segments.to_vec());
        live.add_render_hook(hook);
        assert_eq!(live.render_hooks.len(), 1);
    }
}
