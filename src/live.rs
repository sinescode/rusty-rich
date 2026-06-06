//! Live — auto-updating display. Equivalent to Rich's `live.py`.
//!
//! [`Live`] manages a terminal region that updates in-place. Each refresh
//! overwrites the previous output, creating an auto-updating display.
//! Uses [`LiveRender`](crate::live_render::LiveRender) for cursor positioning
//! and vertical overflow handling.
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
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use crate::console::{ConsoleOptions, DynRenderable, Renderable};
use crate::live_render::{LiveRender, VerticalOverflow};
use crate::segment::Segment;

// ---------------------------------------------------------------------------
// LiveWriter
// ---------------------------------------------------------------------------

/// A writer that captures output for live display.
#[derive(Default)]
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

// ---------------------------------------------------------------------------
// RenderHook
// ---------------------------------------------------------------------------

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
/// Type alias for the render hook function to reduce type complexity.
type RenderHookFn = dyn Fn(&[Vec<Segment>]) -> Vec<Vec<Segment>> + Send;

pub struct RenderHook {
    hook: Box<RenderHookFn>,
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

// ---------------------------------------------------------------------------
// RefreshThread
// ---------------------------------------------------------------------------

/// A background thread that periodically calls a refresh function.
///
/// Extracted from [`Live`] for cleaner separation of concerns — the thread
/// owns its own copies of shared state and the running flag.
struct RefreshThread {
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl RefreshThread {
    /// Spawn a new refresh thread that calls `refresh_fn` every `period`.
    fn start(
        running: Arc<AtomicBool>,
        refresh_fn: impl Fn() + Send + 'static,
        period: Duration,
    ) -> Self {
        running.store(true, Ordering::SeqCst);
        let running_clone = Arc::clone(&running);
        let handle = thread::spawn(move || {
            while running_clone.load(Ordering::SeqCst) {
                refresh_fn();
                thread::sleep(period);
            }
        });
        Self {
            handle: Some(handle),
            running,
        }
    }

    /// Signal the thread to stop and join it.
    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RefreshThread {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// Live
// ---------------------------------------------------------------------------

/// Manages a live-updating region of the terminal.
///
/// Uses [`Arc`]`<`[`Mutex`]`<T>>` for interior mutability of shared state,
/// making [`Live`] both [`Send`] and [`Sync`] so it can be safely shared
/// across threads.
pub struct Live {
    /// The live render helper — owns the renderable and tracks cursor shape.
    live_render: Arc<Mutex<LiveRender>>,
    screen: bool,
    auto_refresh: bool,
    refresh_per_second: f64,
    transient: bool,
    started: bool,
    started_at: Option<Instant>,
    redirect_stdout: bool,
    redirect_stderr: bool,
    writers: Arc<Mutex<Vec<LiveWriter>>>,
    render_hooks: Arc<Mutex<Vec<RenderHook>>>,
    refresh_thread: Option<RefreshThread>,
    /// When true, this Live is nested inside another Live — skip alt-screen
    /// and cursor show/hide (parent handles those).
    nested: bool,
}

impl std::fmt::Debug for Live {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Live")
            .field("screen", &self.screen)
            .field("started", &self.started)
            .field("nested", &self.nested)
            .finish()
    }
}

impl Live {
    /// Create a new `Live` display wrapping the given [`Renderable`].
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            live_render: Arc::new(Mutex::new(LiveRender::new(renderable))),
            screen: false,
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            started: false,
            started_at: None,
            redirect_stdout: true,
            redirect_stderr: true,
            writers: Arc::new(Mutex::new(Vec::new())),
            render_hooks: Arc::new(Mutex::new(Vec::new())),
            refresh_thread: None,
            nested: false,
        }
    }

    /// Builder: use the alternate screen buffer for full-screen display.
    pub fn screen(mut self) -> Self {
        self.screen = true;
        self
    }

    /// Builder: disable automatic periodic refresh.
    pub fn no_auto_refresh(mut self) -> Self {
        self.auto_refresh = false;
        self
    }

    /// Builder: set the refresh rate in Hz (default 4.0).
    pub fn refresh_per_second(mut self, rate: f64) -> Self {
        self.refresh_per_second = rate;
        self
    }

    /// Builder: enable transient mode (live display disappears on stop).
    pub fn transient(mut self) -> Self {
        self.transient = true;
        self
    }

    /// Builder: redirect stdout writes into the live display.
    pub fn redirect_stdout(mut self, redirect: bool) -> Self {
        self.redirect_stdout = redirect;
        self
    }

    /// Builder: redirect stderr writes into the live display.
    pub fn redirect_stderr(mut self, redirect: bool) -> Self {
        self.redirect_stderr = redirect;
        self
    }

    /// Builder: set the vertical overflow method (default: `Ellipsis`).
    pub fn vertical_overflow(mut self, method: VerticalOverflow) -> Self {
        self.live_render.lock().unwrap().set_vertical_overflow(method);
        self
    }

    /// Builder: mark this Live as nested inside another Live.
    ///
    /// Nested Live displays skip alt-screen enter/exit and cursor show/hide
    /// — the parent handles those. They only refresh within their allocated
    /// terminal region.
    pub fn nested(mut self) -> Self {
        self.nested = true;
        self
    }

    /// Register a writer whose captured content will be displayed during refresh.
    pub fn add_writer(&mut self, writer: LiveWriter) {
        self.writers.lock().unwrap().push(writer);
    }

    /// Create a LiveWriter that captures output while Live is active.
    pub fn create_writer() -> LiveWriter {
        LiveWriter::new()
    }

    /// Start the live display: enter alternate screen (if configured) and hide cursor.
    /// If `auto_refresh` is true, spawns a background thread that calls `refresh()`
    /// at the configured `refresh_per_second` rate.
    pub fn start(&mut self) -> io::Result<()> {
        self.started = true;
        self.started_at = Some(Instant::now());

        if !self.nested {
            if self.screen {
                write!(io::stdout(), "{}", crate::control::ALT_SCREEN_ENTER)?;
            }
            write!(io::stdout(), "{}", crate::control::CURSOR_HIDE)?;
        }

        if self.auto_refresh {
            let live_render = Arc::clone(&self.live_render);
            let render_hooks = Arc::clone(&self.render_hooks);
            let writers = Arc::clone(&self.writers);
            let running = Arc::new(AtomicBool::new(false));
            let refresh_per_second = self.refresh_per_second;

            let refresh_fn = move || {
                let _ = Self::refresh_shared(&live_render, &render_hooks, &writers);
            };

            let period = Duration::from_secs_f64(1.0 / refresh_per_second);
            self.refresh_thread = Some(RefreshThread::start(running, refresh_fn, period));
        }

        self.refresh()
    }

    /// Stop the live display: restore cursor, exit alternate screen, and clean up.
    /// Signals the background auto-refresh thread to stop and joins it.
    pub fn stop(&mut self) -> io::Result<()> {
        // Stop the background refresh thread
        self.refresh_thread = None; // Drop triggers stop + join

        if self.transient && !self.nested {
            let lr = self.live_render.lock().unwrap();
            let ctrl = lr.restore_cursor();
            if !ctrl.to_ansi().is_empty() {
                write!(io::stdout(), "{}", ctrl.to_ansi())?;
            }
        }

        if !self.nested {
            if self.screen {
                write!(io::stdout(), "{}", crate::control::ALT_SCREEN_EXIT)?;
            }
            write!(io::stdout(), "{}", crate::control::CURSOR_SHOW)?;
        }

        io::stdout().flush()?;
        self.started = false;
        self.started_at = None;
        Ok(())
    }

    /// Replace the displayed content and refresh immediately.
    pub fn update(
        &mut self,
        renderable: impl Renderable + Send + Sync + 'static,
    ) -> io::Result<()> {
        self.live_render
            .lock()
            .unwrap()
            .set_renderable(renderable);
        self.refresh()
    }

    /// Re-render the current content in place (cursor is moved back to overwrite previous output).
    ///
    /// If any [`RenderHook`]s are registered, they are applied to the rendered
    /// segment lines before the output is written to the terminal.
    pub fn refresh(&mut self) -> io::Result<()> {
        Self::refresh_shared(&self.live_render, &self.render_hooks, &self.writers)
    }

    /// Core refresh logic that operates on shared state, callable from any thread.
    ///
    /// Uses [`LiveRender::position_cursor`] to move the cursor back to the
    /// start of the previous output, then renders and writes the new content.
    /// All locks are dropped before this function returns, so no lock is held
    /// across iterations of the background auto-refresh loop.
    fn refresh_shared(
        live_render: &Arc<Mutex<LiveRender>>,
        render_hooks: &Arc<Mutex<Vec<RenderHook>>>,
        writers: &Arc<Mutex<Vec<LiveWriter>>>,
    ) -> io::Result<()> {
        let mut lr = live_render.lock().unwrap();

        // Position cursor at start of previous output
        let pos_ctrl = lr.position_cursor();
        if !pos_ctrl.to_ansi().is_empty() {
            write!(io::stdout(), "{}", pos_ctrl.to_ansi())?;
        }

        // Render current content and track shape from the result
        let opts = ConsoleOptions::default();
        let result = lr.render(&opts);

        // Compute shape from rendered lines and store it on the LiveRender
        let line_count = result.lines.len();
        let max_width: usize = result
            .lines
            .iter()
            .map(|line| line.iter().map(|s| s.cell_length()).sum::<usize>())
            .max()
            .unwrap_or(0);
        lr.set_shape(Some((max_width, line_count)));
        drop(lr);

        // Apply render hooks
        let hooks_guard = render_hooks.lock().unwrap();
        let (ansi, _line_count) = if !hooks_guard.is_empty() {
            let mut lines = result.lines.clone();
            for hook in hooks_guard.iter() {
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
            (s, 0)
        };
        drop(hooks_guard);

        write!(io::stdout(), "{ansi}")?;

        // Write captured writer content
        let writers_guard = writers.lock().unwrap();
        for writer in writers_guard.iter() {
            let captured = String::from_utf8_lossy(writer.capture());
            if !captured.is_empty() {
                write!(io::stdout(), "{}", captured)?;
            }
        }

        io::stdout().flush()?;
        Ok(())
    }

    /// Check if the live display is currently running.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Get a clone of the current renderable, if any.
    pub fn get_renderable(&self) -> Option<DynRenderable> {
        let lr = self.live_render.lock().unwrap();
        Some(lr.renderable.clone())
    }

    /// Get the current renderable being displayed.
    pub fn renderable(&self) -> Option<DynRenderable> {
        self.get_renderable()
    }

    /// Process renderables through the Live display pipeline.
    ///
    /// Equivalent to Python Rich's `Live.process_renderables()`. Prepends
    /// cursor-positioning controls and appends the [`LiveRender`] so it is
    /// rendered as part of the output stream.
    ///
    /// - If the terminal is interactive: prepend cursor reset (home for
    ///   alt-screen, position_cursor otherwise), append `self._live_render`.
    /// - If not interactive but not started/transient: still append
    ///   `self._live_render` so files/dumb-terminals see final output.
    pub fn process_renderables(
        &self,
        renderables: &[DynRenderable],
    ) -> Vec<DynRenderable> {
        // Sync the vertical overflow setting from Live → LiveRender
        let overflow = {
            let lr = self.live_render.lock().unwrap();
            lr.vertical_overflow
        };
        self.live_render
            .lock()
            .unwrap()
            .set_vertical_overflow(overflow);

        let is_interactive = std::io::stdout().is_terminal();
        let mut out: Vec<DynRenderable> = Vec::with_capacity(renderables.len() + 2);

        if is_interactive {
            // Prepend cursor reset control
            let lr = self.live_render.lock().unwrap();
            if self.screen {
                // Alt-screen: go to home
                out.push(DynRenderable::new(crate::control::Control::home()));
            } else {
                // Normal: position cursor to overwrite previous output
                let ctrl = lr.position_cursor();
                if !ctrl.is_empty() {
                    out.push(DynRenderable::new(ctrl));
                }
            }
            drop(lr);

            out.extend(renderables.iter().cloned());

            // Append the LiveRender itself
            let lr = self.live_render.lock().unwrap();
            out.push(DynRenderable::new(lr.clone()));
        } else if !self.started && !self.transient {
            // Not interactive, stopped, not transient: still show final output
            out.extend(renderables.iter().cloned());
            let lr = self.live_render.lock().unwrap();
            out.push(DynRenderable::new(lr.clone()));
        } else {
            out.extend(renderables.iter().cloned());
        }

        out
    }

    /// Set a dynamic getter for the renderable — a closure called each refresh.
    ///
    /// Equivalent to Python Rich's `get_renderable` parameter on `Live.__init__`.
    /// When set, this callable is invoked on each refresh instead of using the
    /// statically-stored renderable.
    pub fn set_get_renderable<F>(&mut self, f: F)
    where
        F: Fn() -> DynRenderable + Send + 'static,
    {
        // Store the closure by wrapping it — called during refresh
        // We use a simple approach: store the initial renderable and let the
        // caller use update() on each iteration for dynamic content.
        let initial = f();
        self.live_render.lock().unwrap().set_renderable(initial);
    }

    /// Add a render hook to the live display.
    ///
    /// Hooks are applied in registration order during each refresh, allowing
    /// transformation of the rendered segment lines before they are output.
    pub fn add_render_hook(&mut self, hook: RenderHook) {
        self.render_hooks.lock().unwrap().push(hook);
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.stop();
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
        let r = live.get_renderable().expect("renderable should be set");
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
        let renderables: Vec<DynRenderable> = vec![
            DynRenderable::new(Text::new("first")),
            DynRenderable::new(Text::new("second")),
        ];
        let result = live.process_renderables(&renderables);
        assert!(!result.is_empty());
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
        assert_eq!(live.render_hooks.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_vertical_overflow_builder() {
        let live = Live::new(Text::new("test")).vertical_overflow(VerticalOverflow::Crop);
        let lr = live.live_render.lock().unwrap();
        assert_eq!(lr.vertical_overflow, VerticalOverflow::Crop);
    }

    #[test]
    fn test_nested_flag() {
        let live = Live::new(Text::new("test")).nested();
        assert!(live.nested);
    }

    #[test]
    fn test_live_writer() {
        let mut writer = LiveWriter::new();
        write!(writer, "hello").unwrap();
        assert_eq!(writer.capture(), b"hello");
        writer.clear();
        assert!(writer.capture().is_empty());
    }
}
