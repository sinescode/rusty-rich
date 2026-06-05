//! Console — the central rendering engine. Equivalent to Rich's `console.py`.
//!
//! The `Console` is the main entry point for rendering. It manages terminal
//! detection, color system support, and dispatching renderables to produce
//! styled output.

use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::sync::{Arc, Mutex};

use crate::align::AlignMethod;
use crate::color::{Color, ColorSystem};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// ConsoleDimensions
// ---------------------------------------------------------------------------

/// Size of the terminal in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleDimensions {
    pub width: usize,
    pub height: usize,
}

impl ConsoleDimensions {
    /// Detect the terminal size, falling back to 80x25 if detection fails.
    pub fn detect() -> Self {
        if let Some((w, h)) = terminal_size::terminal_size() {
            Self {
                width: w.0 as usize,
                height: h.0 as usize,
            }
        } else {
            Self {
                width: 80,
                height: 25,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// OverflowMethod
// ---------------------------------------------------------------------------

/// How to handle text that overflows the available width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowMethod {
    /// Wrap text onto the next line.
    Fold,
    /// Crop text at the boundary.
    Crop,
    /// Crop and append "…".
    Ellipsis,
    /// Let text overflow (don't clip).
    Ignore,
}

// ---------------------------------------------------------------------------
// ConsoleOptions
// ---------------------------------------------------------------------------

/// Options passed to renderables during rendering.
#[derive(Debug, Clone)]
pub struct ConsoleOptions {
    /// Terminal size.
    pub size: ConsoleDimensions,
    /// True if output is a terminal.
    pub is_terminal: bool,
    /// The encoding (almost always UTF-8).
    pub encoding: String,
    /// Minimum render width.
    pub min_width: usize,
    /// Maximum render width.
    pub max_width: usize,
    /// Maximum height.
    pub max_height: usize,
    /// Override for text justification.
    pub justify: Option<AlignMethod>,
    /// Override for overflow handling.
    pub overflow: Option<OverflowMethod>,
    /// Disable text wrapping.
    pub no_wrap: bool,
    /// If true, use ASCII-only box characters.
    pub ascii_only: bool,
    /// If true, enable markup interpretation.
    pub markup: bool,
    /// If true, enable syntax highlighting of strings.
    pub highlight: bool,
    /// Optional fixed height for the renderable.
    pub height: Option<usize>,
    /// For legacy Windows console.
    pub legacy_windows: bool,
}

impl Default for ConsoleOptions {
    fn default() -> Self {
        Self {
            size: ConsoleDimensions::detect(),
            is_terminal: true,
            encoding: "utf-8".into(),
            min_width: 1,
            max_width: 80,
            max_height: 25,
            justify: None,
            overflow: None,
            no_wrap: false,
            ascii_only: false,
            markup: true,
            highlight: true,
            height: None,
            legacy_windows: false,
        }
    }
}

impl ConsoleOptions {
    /// Update the max width.
    #[must_use]
    pub fn update_width(&self, max_width: usize) -> Self {
        let mut opts = self.clone();
        opts.max_width = max_width;
        opts
    }

    /// Update the height.
    #[must_use]
    pub fn update_height(&self, height: usize) -> Self {
        let mut opts = self.clone();
        opts.height = Some(height);
        opts
    }

    /// Shrink the max width by an amount (for padding).
    #[must_use]
    pub fn shrink_width(&self, amount: usize) -> Self {
        let mut opts = self.clone();
        opts.max_width = opts.max_width.saturating_sub(amount);
        opts
    }
}

// ---------------------------------------------------------------------------
// Renderable trait
// ---------------------------------------------------------------------------

/// A single item in a render result — either a final `Segment` or a nested
/// renderable that will be recursively flattened by `Console::render()`.
///
/// Equivalent to Python Rich's `RenderResult = Iterable[Union[Segment, RenderableType]]`.
#[derive(Clone)]
pub enum RenderItem {
    /// A fully-rendered [`Segment`].
    Segment(Segment),
    /// A nested [`DynRenderable`] that will be recursively flattened.
    Nested(DynRenderable),
}

impl fmt::Debug for RenderItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Segment(s) => write!(f, "Segment({})", &s.text),
            Self::Nested(_) => write!(f, "Nested(...)"),
        }
    }
}

impl From<Segment> for RenderItem {
    fn from(s: Segment) -> Self {
        Self::Segment(s)
    }
}

impl From<DynRenderable> for RenderItem {
    fn from(r: DynRenderable) -> Self {
        Self::Nested(r)
    }
}

/// The result of rendering: a list of lines, each line being a list of
/// segments.  Also carries an optional `items` list for recursive rendering.
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Flat line-oriented segments (backward-compatible).
    pub lines: Vec<Vec<Segment>>,
    /// Optional render items for recursive flattening. When present,
    /// `Console::render()` recurses into nested renderables.
    pub items: Vec<RenderItem>,
}

impl Default for RenderResult {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderResult {
    /// Create an empty [`RenderResult`].
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            items: Vec::new(),
        }
    }

    /// Create a [`RenderResult`] from a plain text string.
    ///
    /// The text becomes a single line with one segment.
    pub fn from_text(text: &str) -> Self {
        Self {
            lines: vec![vec![Segment::new(text)]],
            items: vec![RenderItem::Segment(Segment::new(text))],
        }
    }

    /// Create a [`RenderResult`] from a list of [`Segment`]s on a single line.
    pub fn from_segments(segments: Vec<Segment>) -> Self {
        let items: Vec<RenderItem> = segments
            .iter()
            .map(|s| RenderItem::Segment(s.clone()))
            .collect();
        Self {
            lines: vec![segments],
            items,
        }
    }

    /// Create a [`RenderResult`] from pre-computed lines of [`Segment`]s.
    pub fn from_lines(lines: Vec<Vec<Segment>>) -> Self {
        Self {
            lines,
            items: Vec::new(),
        }
    }

    /// Create a [`RenderResult`] from [`RenderItem`]s for recursive flattening.
    pub fn from_items(items: Vec<RenderItem>) -> Self {
        Self {
            lines: Vec::new(),
            items,
        }
    }

    /// Push a segment item.
    pub fn push_item(&mut self, item: impl Into<RenderItem>) {
        self.items.push(item.into());
    }

    /// Push a nested renderable for recursive flattening.
    pub fn push_renderable(&mut self, r: impl Renderable + Send + Sync + 'static) {
        self.items.push(RenderItem::Nested(DynRenderable::new(r)));
    }

    /// Recursively flatten items into segments using the given options.
    /// This is called by `Console::render()` to resolve nested renderables.
    pub fn flatten(&self, options: &ConsoleOptions) -> Vec<Segment> {
        let mut out: Vec<Segment> = Vec::new();
        flatten_items(&self.items, options, &mut out);
        // Also flatten lines for backward compat
        if out.is_empty() {
            for line in &self.lines {
                for seg in line {
                    out.push(seg.clone());
                }
            }
        }
        out
    }

    /// Flatten all segments into a single ANSI string.
    pub fn to_ansi(&self) -> String {
        let mut out = String::new();
        // Use items if present, otherwise fall back to lines
        if !self.items.is_empty() {
            let flat = self.flatten(&ConsoleOptions::default());
            for seg in &flat {
                out.push_str(&seg.to_ansi());
            }
        } else {
            for line in &self.lines {
                for seg in line {
                    out.push_str(&seg.to_ansi());
                }
            }
        }
        out
    }
}

/// Recursively flatten `RenderItem`s into a `Vec<Segment>`.
fn flatten_items(items: &[RenderItem], options: &ConsoleOptions, out: &mut Vec<Segment>) {
    for item in items {
        match item {
            RenderItem::Segment(seg) => out.push(seg.clone()),
            RenderItem::Nested(renderable) => {
                let nested = renderable.render(options);
                flatten_items(&nested.items, options, out);
            }
        }
    }
}

/// Trait for anything that can be rendered to the console.
///
/// Equivalent to `__rich_console__` in Python Rich.
pub trait Renderable {
    /// Render this object into a [`RenderResult`] using the provided options.
    ///
    /// Implementing types produce [`Segment`]s or nested [`Renderable`]s
    /// that are recursively flattened by [`Console::render`].
    fn render(&self, options: &ConsoleOptions) -> RenderResult;

    /// Optional width-measurement hook (equivalent to `__rich_measure__`).
    /// Override to provide min/max width constraints for layout.
    fn measure(&self, _options: &ConsoleOptions) -> Option<crate::measure::Measurement> {
        None
    }
}

// -- Implementations for common types ---------------------------------------

/// Allows a [`String`] to be used as a renderable.
impl Renderable for String {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        self.as_str().render(options)
    }
}

/// Allows a [`&str`] to be used as a renderable (rendered as plain text).
impl Renderable for &str {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        RenderResult::from_text(self)
    }
}

/// Allows a [`Text`] object to be used as a renderable.
impl Renderable for Text {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        let rendered = self.render();
        // Simple: just treat the rendered ANSI string as one segment per line
        let lines: Vec<Vec<Segment>> = rendered.lines().map(|l| vec![Segment::new(l)]).collect();
        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

/// A wrapper that provides `Clone` + `Debug` for trait-object renderables.
///
/// [`DynRenderable`] boxes any [`Renderable`] behind an [`Arc`] so it can be
/// stored in collections like [`Group`] and [`Panel`](crate::Panel).
#[derive(Clone)]
pub struct DynRenderable {
    inner: Arc<dyn Renderable + Send + Sync>,
}

impl DynRenderable {
    /// Wrap a [`Renderable`] in a [`DynRenderable`].
    pub fn new(r: impl Renderable + Send + Sync + 'static) -> Self {
        Self { inner: Arc::new(r) }
    }
}

impl fmt::Debug for DynRenderable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynRenderable").finish()
    }
}

/// Delegates rendering to the inner trait object.
impl Renderable for DynRenderable {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        self.inner.render(options)
    }

    fn measure(&self, options: &ConsoleOptions) -> Option<crate::measure::Measurement> {
        self.inner.measure(options)
    }
}

/// A renderable that renders multiple children one after another (vertically).
///
/// Equivalent to Python Rich's `Group`.
#[derive(Debug, Clone)]
pub struct Group {
    /// The child renderables to render in sequence.
    pub children: Vec<DynRenderable>,
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

impl Group {
    /// Create an empty [`Group`].
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a renderable child to the group.
    pub fn add(&mut self, renderable: impl Renderable + Send + Sync + 'static) {
        self.children.push(DynRenderable::new(renderable));
    }
}

/// Renders each child sequentially and concatenates their output lines.
impl Renderable for Group {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut all_lines: Vec<Vec<Segment>> = Vec::new();
        for child in &self.children {
            let result = child.render(options);
            all_lines.extend(result.lines);
        }
        RenderResult {
            lines: all_lines,
            items: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Capture system — redirect console output to a buffer
// ---------------------------------------------------------------------------

/// Private writer that captures output into a shared buffer.
struct CaptureWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut data = self.buf.lock().unwrap();
        data.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Captured console output. Created by [`Console::end_capture`].
#[derive(Debug)]
pub struct Capture {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Capture {
    /// Create an empty Capture (not connected to any console).
    pub fn new(_console: &Console) -> Self {
        Self {
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the captured text.
    pub fn get(&self) -> String {
        let data = self.buf.lock().unwrap();
        String::from_utf8_lossy(&data).to_string()
    }
}

// Re-export pager types from the dedicated pager module
pub use crate::pager::{Pager, PagerContext, SystemPager};

// ---------------------------------------------------------------------------
// CaptureError
// ---------------------------------------------------------------------------

/// Error type for capture operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    /// Capture is already in progress.
    AlreadyCapturing,
    /// No capture is currently active.
    NotCapturing,
    /// The captured output could not be decoded as UTF-8.
    InvalidUtf8,
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyCapturing => write!(f, "capture already in progress"),
            Self::NotCapturing => write!(f, "no capture active"),
            Self::InvalidUtf8 => write!(f, "captured output is not valid UTF-8"),
        }
    }
}

impl std::error::Error for CaptureError {}

// ---------------------------------------------------------------------------
// NewLine / NoChange renderables
// ---------------------------------------------------------------------------

/// A renderable that outputs a single newline.
pub struct NewLine;

impl Renderable for NewLine {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        RenderResult::from_text("\n")
    }
}

/// A renderable that outputs nothing (used as a sentinel).
pub struct NoChange;

impl Renderable for NoChange {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        RenderResult::new()
    }
}

// ---------------------------------------------------------------------------
// RenderHook — modify render output before display
// ---------------------------------------------------------------------------

/// Type alias for the render hook closure.
pub type RenderHookFn = dyn Fn(&[Vec<Segment>]) -> Vec<Vec<Segment>> + Send;

/// A hook that can modify render output before display.
pub struct RenderHook {
    hook: Box<RenderHookFn>,
}

impl RenderHook {
    /// Create a new RenderHook from a closure.
    pub fn new<F: Fn(&[Vec<Segment>]) -> Vec<Vec<Segment>> + Send + 'static>(f: F) -> Self {
        Self { hook: Box::new(f) }
    }

    /// Apply the hook to a set of rendered lines.
    pub fn apply(&self, lines: &[Vec<Segment>]) -> Vec<Vec<Segment>> {
        (self.hook)(lines)
    }
}

impl fmt::Debug for RenderHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderHook").finish()
    }
}

// ---------------------------------------------------------------------------
// ThemeContext — temporarily switch themes with RAII restoration
// ---------------------------------------------------------------------------

/// A RAII guard that restores a previous theme when dropped.
///
/// Created by [`Console::use_theme`]. While alive, the console uses the new
/// theme. When the context is dropped, the original theme is restored.
// SAFETY: The PhantomData<'a mut Console> ensures the compiler enforces that
// Console outlives ThemeContext. The raw pointer is valid because:
// 1. ThemeContext is not Send or Sync (raw pointer prevents auto-derive)
// 2. The pointer comes from a &'a mut reference
// 3. PhantomData links the borrow lifetime to ThemeContext's lifetime
pub struct ThemeContext<'a> {
    _phantom: std::marker::PhantomData<&'a mut Console>,
    console_ptr: *mut Console,
    previous_theme: Theme,
    // Explicitly prevents Send + Sync auto-derive (VULN-004)
    _not_send_sync: std::marker::PhantomData<*const ()>,
}

// SAFETY: ThemeContext is not Send or Sync because of the raw pointer.
// It must only be used on the same thread as the Console.
// The pointer is valid because Console creates ThemeContext and outlives it.

impl<'a> ThemeContext<'a> {
    /// Create a new ThemeContext (internal — use [`Console::use_theme`]).
    pub(crate) fn new(console: &'a mut Console, previous_theme: Theme) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            console_ptr: console as *mut Console,
            previous_theme,
            _not_send_sync: std::marker::PhantomData,
        }
    }
}

impl<'a> Drop for ThemeContext<'a> {
    fn drop(&mut self) {
        unsafe {
            (*self.console_ptr).theme = std::mem::take(&mut self.previous_theme);
        }
    }
}

// ---------------------------------------------------------------------------
// Console
// ---------------------------------------------------------------------------

/// The main console for rendering rich output.
pub struct Console {
    /// The output writer.
    pub file: Box<dyn Write + Send>,
    /// Detected color system.
    pub color_system: ColorSystem,
    /// Current theme.
    pub theme: Theme,
    /// Default options.
    pub options: ConsoleOptions,
    /// Current width (may be overridden).
    width: Option<usize>,
    /// Current height (may be overridden).
    height: Option<usize>,
    /// Is this output a terminal?
    is_terminal: bool,
    /// If true, suppress all output.
    pub quiet: bool,
    /// If true, text wraps at word boundaries.
    pub soft_wrap: bool,
    /// Is the alternate screen active?
    alt_screen: bool,
    /// Is the cursor visible?
    cursor_visible: bool,
    /// Number of spaces per tab (default 8).
    pub tab_size: usize,
    /// If true, use ASCII-safe box characters (for legacy terminals).
    pub safe_box: bool,
    /// Active render hooks that modify output before display.
    render_hooks: Vec<RenderHook>,
    /// Captured output buffer (active when capturing).
    capture_buf: Option<Arc<Mutex<Vec<u8>>>>,
    /// Original file writer saved during capture.
    saved_file: Option<Box<dyn Write + Send>>,
}

impl Console {
    /// Create a new Console writing to stdout.
    pub fn new() -> Self {
        let is_terminal = std::io::stdout().is_terminal();
        let color_system = detect_color_system();

        let size = ConsoleDimensions::detect();
        // Subtract 1 column from the width to prevent line wrapping in
        // terminals where output that exactly fills the last column causes
        // the cursor to advance and wrap visually before the newline.
        let render_width = size.width.saturating_sub(1);

        Self {
            file: Box::new(io::stdout()) as Box<dyn Write + Send>,
            color_system,
            theme: crate::theme::default_theme(),
            options: ConsoleOptions {
                size,
                is_terminal,
                max_width: render_width,
                max_height: size.height,
                ..Default::default()
            },
            width: None,
            height: None,
            is_terminal,
            quiet: false,
            soft_wrap: false,
            alt_screen: false,
            cursor_visible: true,
            tab_size: 8,
            safe_box: true,
            render_hooks: Vec::new(),
            capture_buf: None,
            saved_file: None,
        }
    }

    /// Create a Console that writes to a file.
    pub fn with_file(file: Box<dyn Write + Send>) -> Self {
        let _is_terminal = false;
        Self {
            file,
            color_system: ColorSystem::Standard,
            theme: crate::theme::default_theme(),
            options: ConsoleOptions {
                size: ConsoleDimensions {
                    width: 80,
                    height: 25,
                },
                is_terminal: false,
                max_width: 80,
                max_height: 25,
                ..Default::default()
            },
            width: None,
            height: None,
            is_terminal: false,
            quiet: false,
            soft_wrap: false,
            alt_screen: false,
            cursor_visible: true,
            tab_size: 8,
            safe_box: true,
            render_hooks: Vec::new(),
            capture_buf: None,
            saved_file: None,
        }
    }

    /// Set the console width (overrides auto-detected terminal width).
    pub fn set_width(&mut self, width: usize) {
        self.width = Some(width);
        self.options.max_width = width;
    }

    /// Set the console height.
    pub fn set_height(&mut self, height: usize) {
        self.height = Some(height);
        self.options.max_height = height;
    }

    /// Get the effective width.
    pub fn width(&self) -> usize {
        self.width.unwrap_or(self.options.size.width)
    }

    /// Get the effective height.
    pub fn height(&self) -> usize {
        self.height.unwrap_or(self.options.size.height)
    }

    /// Render a renderable and return the segment lines.
    pub fn render_lines(
        &self,
        renderable: &dyn Renderable,
        options: &ConsoleOptions,
        style: Option<&Style>,
        _pad: bool,
    ) -> Vec<Vec<Segment>> {
        let result = renderable.render(options);

        if let Some(st) = style {
            result
                .lines
                .into_iter()
                .map(|line| {
                    line.into_iter()
                        .map(|seg| {
                            let new_style = if let Some(ref s) = seg.style {
                                s.combine(st)
                            } else {
                                st.clone()
                            };
                            Segment::styled(seg.text, new_style)
                        })
                        .collect()
                })
                .collect()
        } else {
            result.lines
        }
    }

    /// Look up a style by name from the theme.
    pub fn get_style(&self, name: &str, default: &str) -> Option<Style> {
        self.theme.get(name).cloned().or_else(|| {
            if !default.is_empty() {
                Some(Style::from_str(default))
            } else {
                None
            }
        })
    }

    /// Render a string (with optional style).
    pub fn render_str(&self, text: &str, style: &str) -> Text {
        let st = self.get_style(style, "");
        let mut t = Text::new(text);
        if let Some(s) = st {
            t = t.style(s);
        }
        t
    }

    // -----------------------------------------------------------------------
    // print / log methods
    // -----------------------------------------------------------------------

    /// Print one or more renderable objects, separated by `sep`, ending with
    /// `end`.
    pub fn print(&mut self, objects: &[&dyn Renderable], sep: &str, end: &str) {
        if self.quiet {
            return;
        }
        let mut first = true;
        for obj in objects {
            if !first {
                let _ = write!(self.file, "{sep}");
            }
            first = false;
            let result = obj.render(&self.options);
            let ansi = result.to_ansi();
            let _ = write!(self.file, "{ansi}");
        }
        let _ = write!(self.file, "{end}");
        let _ = self.file.flush();
    }

    /// Print a single renderable followed by a newline.
    ///
    /// Re-detects the terminal size on each call so that the output
    /// adapts when the user resizes the terminal window.
    pub fn println(&mut self, renderable: &dyn Renderable) {
        if self.quiet {
            return;
        }
        self.refresh_size();
        let result = renderable.render(&self.options);
        let ansi = result.to_ansi();
        let _ = writeln!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Update `max_width` / `max_height` from the current terminal size.
    fn refresh_size(&mut self) {
        if self.is_terminal {
            let size = ConsoleDimensions::detect();
            self.options.size = size;
            // Subtract 1 column to prevent edge-of-screen line wrapping.
            self.options.max_width = size.width.saturating_sub(1);
            self.options.max_height = size.height;
        }
    }

    /// Print a plain string (supports markup by default when `markup` is
    /// enabled).
    pub fn print_str(&mut self, text: &str) {
        if self.quiet {
            return;
        }
        let ansi = if self.options.markup {
            let parsed = crate::markup::render(text);
            parsed.render()
        } else {
            // Strip raw ANSI escapes from plain text to prevent injection
            crate::export::strip_ansi_escapes(text)
        };
        let _ = write!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Print formatted JSON.
    pub fn print_json(&mut self, data: &serde_json::Value) {
        if self.quiet {
            return;
        }
        let formatted = crate::json::render_json(data);
        let result = formatted.render(&self.options);
        let ansi = result.to_ansi();
        let _ = writeln!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Clear the screen.
    pub fn clear(&mut self) {
        if self.quiet {
            return;
        }
        let _ = write!(self.file, "{}", crate::control::CLEAR_HOME);
        let _ = self.file.flush();
    }

    /// Show the cursor.
    pub fn show_cursor(&mut self) {
        self.cursor_visible = true;
        let _ = write!(self.file, "{}", crate::control::CURSOR_SHOW);
        let _ = self.file.flush();
    }

    /// Hide the cursor.
    pub fn hide_cursor(&mut self) {
        self.cursor_visible = false;
        let _ = write!(self.file, "{}", crate::control::CURSOR_HIDE);
        let _ = self.file.flush();
    }

    /// Set the terminal window title.
    pub fn set_window_title(&mut self, title: &str) {
        let _ = write!(
            self.file,
            "{}{}{}",
            crate::control::OSC,
            title,
            crate::control::ST
        );
        let _ = self.file.flush();
    }

    /// Get the ANSI escape string for a given color as this console supports.
    pub fn color_ansi(&self, color: &Color) -> String {
        let downgraded = color.downgrade(self.color_system);
        downgraded.to_string()
    }

    // -- Recursive rendering ------------------------------------------------

    /// Render a renderable by recursively flattening nested items into
    /// segments.  This is equivalent to Python Rich's `Console.render()`.
    /// It handles `Group` composition and any renderable that yields other
    /// renderables.
    pub fn render(&self, renderable: &dyn Renderable, options: &ConsoleOptions) -> Vec<Segment> {
        let result = renderable.render(options);
        result.flatten(options)
    }

    /// Measure a renderable's width constraints.
    /// Equivalent to Python Rich's `Measurement.get(console, options, renderable)`.
    pub fn measure(
        &self,
        renderable: &dyn Renderable,
        options: &ConsoleOptions,
    ) -> crate::measure::Measurement {
        if let Some(m) = renderable.measure(options) {
            return m;
        }
        let segments = self.render(renderable, options);
        let max_w = segments.iter().map(|s| s.cell_length()).max().unwrap_or(0);
        crate::measure::Measurement::new(max_w, options.max_width)
    }

    // -- Convenience render methods -----------------------------------------

    /// Render a rule with the given title.
    /// Equivalent to `Console.rule()`.
    pub fn rule(
        &mut self,
        title: impl Into<String>,
        characters: Option<&str>,
        style: Option<Style>,
        align: Option<AlignMethod>,
    ) {
        if self.quiet {
            return;
        }
        let mut rule = crate::rule::Rule::new().title(title);
        if let Some(chars) = characters {
            rule = rule.characters(chars);
        }
        if let Some(st) = style {
            rule = rule.style(st);
        }
        if let Some(a) = align {
            rule = rule.align(a);
        }
        let result = rule.render(&self.options);
        let ansi = result.to_ansi();
        let _ = write!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Output a bell character.
    pub fn bell(&mut self) {
        if self.quiet {
            return;
        }
        let _ = write!(self.file, "\x07");
        let _ = self.file.flush();
    }

    /// Output blank lines.
    pub fn line(&mut self, count: usize) {
        if self.quiet {
            return;
        }
        for _ in 0..count {
            let _ = writeln!(self.file);
        }
        let _ = self.file.flush();
    }

    /// Output a log entry with timestamp, caller info.
    pub fn log(&mut self, objects: &[&dyn Renderable]) {
        if self.quiet {
            return;
        }
        let now = chrono::Local::now();
        let time_str = format!("[{}]", now.format("%H:%M:%S"));
        let _ = write!(self.file, "{} ", Style::new().dim(true).to_ansi());
        let _ = write!(self.file, "{time_str} ");
        let _ = write!(self.file, "{}", Style::new().reset_ansi());
        self.print(objects, " ", "\n");
    }

    // -- Theme stack --------------------------------------------------------

    /// Push a theme onto the stack.
    pub fn push_theme(&mut self, theme: Theme) {
        let mut new_theme = theme.clone();
        new_theme.inherit = Some(Box::new(self.theme.clone()));
        self.theme = new_theme;
    }

    /// Pop the current theme, restoring the previous one.
    pub fn pop_theme(&mut self) {
        if let Some(ref inherit) = self.theme.inherit {
            self.theme = *inherit.clone();
        }
    }

    // -- Export methods ------------------------------------------------------

    /// Export the current console output as an HTML document.
    ///
    /// Renders the given renderable and wraps it in a styled HTML page.
    /// Export the current console output as an HTML document.
    ///
    /// Renders the given renderable to segments, converts to styled HTML spans,
    /// and wraps in a full HTML document. Colors and styles are preserved.
    pub fn export_html(&self, renderable: &dyn Renderable) -> String {
        let segments = self.render(renderable, &self.options);
        let code =
            crate::export::segments_to_html(&segments, &crate::export::ExportTheme::default());
        crate::export::export_html(&crate::export::ExportHtmlOptions {
            code,
            ..Default::default()
        })
    }

    /// Save rendered output as an HTML file.
    pub fn save_html(
        &self,
        path: impl AsRef<std::path::Path>,
        renderable: &dyn Renderable,
    ) -> std::io::Result<()> {
        let html = self.export_html(renderable);
        std::fs::write(path.as_ref(), html)
    }

    /// Export the current console output as an SVG document.
    /// Export the current console output as an SVG document.
    ///
    /// Renders the given renderable to segments, converts to styled SVG
    /// `<tspan>` elements, and wraps in a full SVG document.
    pub fn export_svg(&self, renderable: &dyn Renderable) -> String {
        let segments = self.render(renderable, &self.options);
        let code =
            crate::export::segments_to_svg(&segments, &crate::export::ExportTheme::default());
        crate::export::export_svg(&crate::export::ExportSvgOptions {
            code,
            ..Default::default()
        })
    }

    /// Save rendered output as an SVG file.
    pub fn save_svg(
        &self,
        path: impl AsRef<std::path::Path>,
        renderable: &dyn Renderable,
    ) -> std::io::Result<()> {
        let svg = self.export_svg(renderable);
        crate::export::save_svg(
            path,
            &crate::export::ExportSvgOptions {
                code: svg,
                ..Default::default()
            },
        )
    }

    /// Export the current console output as plain text (strips ANSI).
    pub fn export_text(&self, renderable: &dyn Renderable) -> String {
        let result = renderable.render(&self.options);
        let ansi = result.to_ansi();
        crate::export::export_text(&crate::export::ExportTextOptions {
            text: ansi,
            strip_ansi: true,
        })
    }

    /// Save rendered output as a plain text file.
    pub fn save_text(
        &self,
        path: impl AsRef<std::path::Path>,
        renderable: &dyn Renderable,
    ) -> std::io::Result<()> {
        let text = self.export_text(renderable);
        crate::export::save_text(
            path,
            &crate::export::ExportTextOptions {
                text,
                strip_ansi: false,
            },
        )
    }

    // -- Quiet / Soft-wrap setters ------------------------------------------

    /// Set the quiet flag (suppress all output when true).
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    /// Builder-style setter for quiet.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Set the soft-wrap flag (wrap text at word boundaries when true).
    pub fn set_soft_wrap(&mut self, soft_wrap: bool) {
        self.soft_wrap = soft_wrap;
    }

    /// Builder-style setter for soft_wrap.
    pub fn soft_wrap(mut self, soft_wrap: bool) -> Self {
        self.soft_wrap = soft_wrap;
        self
    }

    // -- Input --------------------------------------------------------------

    /// Read a line of input from the user.
    ///
    /// Writes `prompt` to the console, then reads a line from stdin.
    /// When `password` is true, the input is masked with `*` characters
    /// (using raw terminal mode via crossterm).
    pub fn input(&mut self, prompt: &str, password: bool) -> String {
        let _ = write!(self.file, "{prompt}");
        let _ = self.file.flush();

        if password {
            self.read_password()
        } else {
            let mut input = String::new();
            let _ = io::stdin().read_line(&mut input);
            input.trim().to_string()
        }
    }

    /// Read a password from stdin with character masking.
    fn read_password(&mut self) -> String {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
        use std::io::Read;

        match enable_raw_mode() {
            Ok(()) => {
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                let mut buf = [0u8; 1];
                let mut password = String::new();

                while let Ok(()) = handle.read_exact(&mut buf) {
                    match buf[0] {
                        b'\r' | b'\n' => {
                            let _ = writeln!(self.file);
                            let _ = self.file.flush();
                            break;
                        }
                        b'\x03' => {
                            // Ctrl+C — break and return what we have
                            let _ = writeln!(self.file);
                            let _ = self.file.flush();
                            break;
                        }
                        b'\x7f' | b'\x08' => {
                            // Backspace
                            password.pop();
                        }
                        c => {
                            password.push(c as char);
                            let _ = write!(self.file, "*");
                            let _ = self.file.flush();
                        }
                    }
                }
                let _ = disable_raw_mode();
                password
            }
            Err(_) => {
                // Fallback: read without masking
                let mut input = String::new();
                let _ = io::stdin().read_line(&mut input);
                input.trim().to_string()
            }
        }
    }

    // -- Screen / alternate screen ------------------------------------------

    /// Create a [`ScreenContext`](crate::screen::ScreenContext) that enters the
    /// alternate screen buffer. The context automatically exits the alternate
    /// screen when dropped.
    pub fn screen(&mut self) -> crate::screen::ScreenContext {
        let mut ctx = crate::screen::ScreenContext::new();
        ctx.enter();
        ctx
    }

    /// Enter or exit the alternate screen buffer by writing the corresponding
    /// escape sequences [`crate::control::ALT_SCREEN_ENTER`] / [`crate::control::ALT_SCREEN_EXIT`].
    pub fn set_alt_screen(&mut self, enable: bool) {
        self.alt_screen = enable;
        let seq = if enable {
            crate::control::ALT_SCREEN_ENTER
        } else {
            crate::control::ALT_SCREEN_EXIT
        };
        let _ = write!(self.file, "{seq}");
        let _ = self.file.flush();
    }

    /// Get whether the output is a terminal.
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Set the terminal size (overrides auto-detected dimensions).
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = Some(width);
        self.height = Some(height);
        self.options.max_width = width;
        self.options.max_height = height;
        self.options.size = crate::console::ConsoleDimensions { width, height };
    }

    /// Handle broken pipe errors gracefully.
    ///
    /// In Rust, `write()` returns `ErrorKind::BrokenPipe` instead of raising
    /// `SIGPIPE`, so broken pipes are not fatal. The Console already uses
    /// `let _ = write!(...)` throughout, which silently discards all write
    /// errors including EPIPE. This method is provided for API compatibility
    /// with Python Rich and as a documentation point.
    pub fn on_broken_pipe(&self) {
        // No-op: Rust handles EPIPE via ErrorKind, not signals.
        // All Console write operations use `let _ = write!()` which
        // already discards BrokenPipe errors without panicking.
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Console {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Console")
            .field("color_system", &self.color_system)
            .field("width", &self.width())
            .field("height", &self.height())
            .field("is_terminal", &self.is_terminal)
            .field("alt_screen", &self.alt_screen)
            .field("cursor_visible", &self.cursor_visible)
            .field("quiet", &self.quiet)
            .field("soft_wrap", &self.soft_wrap)
            .finish()
    }
}

// ===========================================================================
// New feature methods (Capture, Pager, Terminal Control, Hooks, etc.)
// ===========================================================================

impl Console {
    // -- Capture System ------------------------------------------------------

    /// Start capturing all output. All subsequent writes to this console are
    /// redirected to an internal buffer. Call [`end_capture`](Self::end_capture)
    /// to stop capturing and retrieve the captured content.
    pub fn begin_capture(&mut self) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = Box::new(CaptureWriter { buf: buf.clone() });
        self.saved_file = Some(std::mem::replace(&mut self.file, writer));
        self.capture_buf = Some(buf);
    }

    /// End capture mode and return the [`Capture`] containing all output written
    /// while capturing was active. The console's output is restored to its
    /// original destination.
    pub fn end_capture(&mut self) -> Result<Capture, CaptureError> {
        let buf = self.capture_buf.take().ok_or(CaptureError::NotCapturing)?;
        if let Some(saved) = self.saved_file.take() {
            self.file = saved;
        }
        Ok(Capture { buf })
    }

    /// Run the given closure with output captured, returning the captured text.
    ///
    /// This is the most ergonomic way to capture output in Rust:
    ///
    /// ```rust,no_run
    /// # use rusty_rich::Console;
    /// let mut console = Console::new();
    /// let output = console.capture(|c| {
    ///     c.print_str("Hello, world!");
    /// }).unwrap();
    /// assert_eq!(output, "Hello, world!");
    /// ```
    pub fn capture<F: FnOnce(&mut Self)>(&mut self, f: F) -> Result<String, CaptureError> {
        self.begin_capture();
        f(self);
        self.end_capture().map(|cap| cap.get())
    }

    // -- Pager System --------------------------------------------------------

    /// Get a [`PagerContext`]. Content rendered while the context is alive is
    /// collected and displayed through the system pager (`$PAGER` or `less`)
    /// when the context is dropped.
    ///
    /// `styles` controls whether ANSI styles are preserved when paging.
    pub fn pager(&mut self, styles: bool) -> PagerContext {
        PagerContext::new(Pager::new().color(styles))
    }

    // -- Input with Renderable prompt ----------------------------------------

    /// Display a [`Renderable`] prompt and read a line of input from stdin.
    ///
    /// The prompt is rendered through the console's current options and theme.
    pub fn input_renderable(&mut self, prompt: &dyn Renderable) -> String {
        if !self.quiet {
            let result = prompt.render(&self.options);
            let ansi = result.to_ansi();
            let _ = write!(self.file, "{ansi}");
            let _ = self.file.flush();
        }
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        input.trim().to_string()
    }

    // -- Exception / Traceback -----------------------------------------------

    /// Print the current exception as a rich traceback.
    ///
    /// In Rust, this is a best-effort rendering; it captures the current
    /// thread's panic info if available. `width` overrides the output width,
    /// and `extra_lines` controls how many lines of source context to show
    /// around each frame.
    pub fn print_exception(&mut self, _width: Option<usize>, _extra_lines: usize) {
        if self.quiet {
            return;
        }
        // Note: Rust does not have Python's sys.exc_info(). A full traceback
        // renderer would need std::panic::catch_unwind or custom error capture.
        // This method provides the API surface; for actual panic tracebacks
        // see crate::traceback::install().
        let msg = "[bold red]Exception[/bold red]: No current exception info. ";
        let msg_text = crate::text::Text::from_markup(msg);
        let result = msg_text.render();
        let _ = writeln!(self.file, "{result}");
        let _ = self.file.flush();
    }

    // -- JSON pretty-print (string overload) -----------------------------------

    /// Pretty-print a JSON string. Parses the string and renders it with
    /// syntax highlighting.
    pub fn print_json_str(&mut self, json: &str) {
        if self.quiet {
            return;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
            self.print_json(&value);
        } else {
            let _ = writeln!(self.file, "[invalid JSON]");
            let _ = self.file.flush();
        }
    }

    // -- Render lines (simple version) ----------------------------------------

    /// Render a renderable to a vector of segment lines.
    ///
    /// This is the lower-level render entry point, returning raw lines instead
    /// of an ANSI string. Compare with [`render`](Self::render) which returns
    /// flat segments.
    pub fn render_to_lines(
        &self,
        renderable: &dyn Renderable,
        options: &ConsoleOptions,
    ) -> Vec<Vec<Segment>> {
        let result = renderable.render(options);
        let has_items = !result.items.is_empty();
        let mut lines = if result.lines.is_empty() && has_items {
            let flat = result.flatten(options);
            if flat.is_empty() {
                Vec::new() // also empty after flatten — keep empty
            } else {
                vec![flat]
            }
        } else {
            result.lines
        };
        // Apply any render hooks
        if !self.render_hooks.is_empty() {
            for hook in &self.render_hooks {
                lines = hook.apply(&lines);
            }
        }
        lines
    }

    // -- Render ANSI string ---------------------------------------------------

    /// Render a plain string to ANSI text, applying the current theme and
    /// styles. Returns the ANSI-formatted string.
    pub fn render_ansi(&self, text: &str) -> String {
        let t = self.render_str(text, "");
        t.render()
    }

    // -- Export SVG with options ----------------------------------------------

    /// Export the console output as an SVG document with explicit options.
    ///
    /// This delegates to [`crate::export::export_svg`] with the given
    /// [`ExportSvgOptions`](crate::export::ExportSvgOptions).
    pub fn export_svg_opts(&self, options: &crate::export::ExportSvgOptions) -> String {
        crate::export::export_svg(options)
    }

    // -- Console Properties ---------------------------------------------------

    /// Get the terminal size as [`ConsoleDimensions`].
    pub fn size(&self) -> ConsoleDimensions {
        ConsoleDimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    /// Check if the terminal is a "dumb" terminal (no color support).
    pub fn is_dumb_terminal(&self) -> bool {
        std::env::var("TERM").is_ok_and(|t| t == "dumb")
    }

    /// Check if the console is currently in the alternate screen buffer.
    pub fn is_alt_screen(&self) -> bool {
        self.alt_screen
    }

    // -- Terminal Control ----------------------------------------------------

    /// Show or hide the cursor based on the boolean parameter.
    ///
    /// `true` shows the cursor, `false` hides it. Tracks the current state
    /// so it can be queried via internal fields.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        let seq = if visible {
            crate::control::CURSOR_SHOW
        } else {
            crate::control::CURSOR_HIDE
        };
        let _ = write!(self.file, "{seq}");
        let _ = self.file.flush();
    }

    /// Temporarily switch to a different theme. Returns a [`ThemeContext`]
    /// that restores the original theme when dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rusty_rich::{Console, Theme};
    /// let mut console = Console::new();
    /// let custom = Theme::new();
    /// {
    ///     let _ctx = console.use_theme(custom);
    ///     // console uses custom theme here
    /// }
    /// // original theme restored here
    /// ```
    pub fn use_theme(&mut self, theme: Theme) -> ThemeContext<'_> {
        let prev = std::mem::replace(&mut self.theme, theme);
        ThemeContext::new(self, prev)
    }

    /// Clear the live display region. When in alt-screen mode, this clears
    /// the entire alternate screen. Otherwise, it's equivalent to
    /// [`clear`](Self::clear).
    pub fn clear_live(&mut self) {
        let _ = write!(self.file, "{}", crate::control::CLEAR_HOME);
        let _ = self.file.flush();
    }

    /// Set the active live display. Stores a reference to the
    /// [`Live`](crate::live::Live) renderer for integration with the
    /// console's rendering pipeline.
    ///
    /// Note: [`Live`](crate::live::Live) manages its own refresh cycle;
    /// this method is primarily for API compatibility with Python Rich.
    pub fn set_live(&mut self, _live: &crate::live::Live) {
        // Live manages its own refresh cycle; this method provides the
        // API surface for attaching a live display to the console.
    }

    /// Update the full screen (enter alt-screen, render content, exit).
    ///
    /// Clears the screen and renders the given renderable. If `options` is
    /// `None`, the console's current options are used.
    pub fn update_screen(&mut self, renderable: &dyn Renderable, options: Option<&ConsoleOptions>) {
        let opts = options.unwrap_or(&self.options);
        let segments = self.render(renderable, opts);
        let mut output = String::new();
        for seg in &segments {
            output.push_str(&seg.to_ansi());
        }
        let _ = write!(self.file, "{}{output}", crate::control::CLEAR_HOME);
        let _ = self.file.flush();
    }

    /// Update the screen from pre-rendered lines of segments.
    ///
    /// Takes already-rendered lines and displays them as the full screen
    /// content, clearing existing content first.
    pub fn update_screen_lines(
        &mut self,
        lines: &[Vec<Segment>],
        options: Option<&ConsoleOptions>,
    ) {
        let _ = options;
        let mut output = String::new();
        for line in lines {
            for seg in line {
                output.push_str(&seg.to_ansi());
            }
            output.push('\n');
        }
        let _ = write!(self.file, "{}{output}", crate::control::CLEAR_HOME);
        let _ = self.file.flush();
    }

    // -- Render Hooks --------------------------------------------------------

    /// Add a [`RenderHook`] to the console. Hooks are applied in order and
    /// can modify the rendered lines before they are displayed.
    pub fn push_render_hook(&mut self, hook: RenderHook) {
        self.render_hooks.push(hook);
    }

    /// Remove and return the most recently added [`RenderHook`], if any.
    pub fn pop_render_hook(&mut self) -> Option<RenderHook> {
        self.render_hooks.pop()
    }
}

// ---------------------------------------------------------------------------
// Color system detection
// ---------------------------------------------------------------------------

/// Detect the terminal color system from environment variables.
///
/// Checks `COLORTERM`, `TERM`, `NO_COLOR`, and `CLICOLOR` to determine
/// whether the terminal supports true color, 8-bit, or standard 16 colors.
fn detect_color_system() -> ColorSystem {
    // Check common env vars
    if let Ok(val) = std::env::var("COLORTERM") {
        if val == "truecolor" || val == "24bit" {
            return ColorSystem::TrueColor;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") {
            return ColorSystem::EightBit;
        }
        if term == "xterm-kitty" {
            return ColorSystem::TrueColor;
        }
    }
    // Check NO_COLOR / CLICOLOR
    if std::env::var("NO_COLOR").is_ok() {
        return ColorSystem::Standard;
    }
    // Default to true color on modern terminals
    if std::io::stdout().is_terminal() {
        ColorSystem::TrueColor
    } else {
        ColorSystem::Standard
    }
}

// ---------------------------------------------------------------------------
// Global console instance (like Rich's `get_console()`)
// ---------------------------------------------------------------------------

use std::sync::LazyLock;

static GLOBAL_CONSOLE: LazyLock<Mutex<Console>> = LazyLock::new(|| Mutex::new(Console::new()));

/// Get a reference to the global Console.
pub fn get_console() -> std::sync::MutexGuard<'static, Console> {
    GLOBAL_CONSOLE.lock().unwrap_or_else(|e| e.into_inner())
}

// ---------------------------------------------------------------------------
// Convenience functions (like Rich's `print()`)
// ---------------------------------------------------------------------------

/// Print objects using the global console.
pub fn print_objects(objects: &[&dyn Renderable]) {
    let mut console = GLOBAL_CONSOLE.lock().unwrap();
    console.print(objects, " ", "\n");
}

/// Print a string with markup support.
pub fn print_str(text: &str) {
    let mut console = GLOBAL_CONSOLE.lock().unwrap();
    console.print_str(text);
}

/// Print formatted JSON.
pub fn print_json_val(data: &serde_json::Value) {
    let mut console = GLOBAL_CONSOLE.lock().unwrap();
    console.print_json(data);
}

// ---------------------------------------------------------------------------
// Reconfigure global console
// ---------------------------------------------------------------------------

/// Reconfigure the global Console singleton with new dimensions and/or
/// color system. This updates the shared global console instance used by
/// [`print_objects`], [`print_str`], and [`print_json_val`].
///
/// # Parameters
///
/// * `width` — New terminal width (None to keep current).
/// * `height` — New terminal height (None to keep current).
/// * `color_system` — New color system level (None to keep current).
pub fn reconfigure(width: Option<usize>, height: Option<usize>, color_system: Option<ColorSystem>) {
    let mut console = GLOBAL_CONSOLE.lock().unwrap();
    if let Some(w) = width {
        console.set_width(w);
    }
    if let Some(h) = height {
        console.set_height(h);
    }
    if let Some(cs) = color_system {
        console.color_system = cs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_result_from_text() {
        let r = RenderResult::from_text("hello");
        assert_eq!(r.lines.len(), 1);
        assert_eq!(r.lines[0][0].text, "hello");
    }

    #[test]
    fn test_console_options_default() {
        let opts = ConsoleOptions::default();
        assert!(opts.markup);
    }

    #[test]
    fn test_console_quiet_default() {
        let console = Console::new();
        assert!(!console.quiet);
    }

    #[test]
    fn test_console_quiet_setter() {
        let mut console = Console::new();
        console.set_quiet(true);
        assert!(console.quiet);
    }

    #[test]
    fn test_console_quiet_builder() {
        let console = Console::new().quiet(true);
        assert!(console.quiet);
    }

    #[test]
    fn test_console_quiet_suppresses_print() {
        let mut console = Console::new();
        console.quiet = true;
        // Should not panic
        console.print(&[], " ", "\n");
        console.println(&"test");
        console.print_str("test");
    }

    #[test]
    fn test_console_soft_wrap_default() {
        let console = Console::new();
        assert!(!console.soft_wrap);
    }

    #[test]
    fn test_console_soft_wrap_setter() {
        let mut console = Console::new();
        console.set_soft_wrap(true);
        assert!(console.soft_wrap);
    }

    #[test]
    fn test_console_soft_wrap_builder() {
        let console = Console::new().soft_wrap(true);
        assert!(console.soft_wrap);
    }

    #[test]
    fn test_console_is_terminal() {
        let console = Console::new();
        // is_terminal depends on whether stdout is a terminal
        let detected = console.is_terminal();
        assert_eq!(detected, std::io::stdout().is_terminal());
    }

    #[test]
    fn test_console_set_size() {
        let mut console = Console::new();
        console.set_size(120, 30);
        assert_eq!(console.width(), 120);
        assert_eq!(console.height(), 30);
        assert_eq!(console.options.max_width, 120);
        assert_eq!(console.options.max_height, 30);
    }

    #[test]
    fn test_console_set_alt_screen() {
        let mut console = Console::new();
        // Just ensure it doesn't panic
        console.set_alt_screen(true);
        console.set_alt_screen(false);
    }

    #[test]
    fn test_console_on_broken_pipe() {
        let console = Console::new();
        console.on_broken_pipe(); // no-op
    }

    #[test]
    fn test_console_input_normal() {
        // We can't easily test stdin in unit tests, but we can verify
        // the method signature compiles and matches.
        let _console = Console::new();
        // input() cannot be meaningfully tested without actual stdin.
    }

    #[test]
    fn test_console_debug() {
        let console = Console::new();
        let debug = format!("{:?}", console);
        assert!(debug.contains("Console"));
    }

    #[test]
    fn test_console_with_file_has_no_terminal() {
        let console = Console::with_file(Box::new(std::io::sink()));
        assert!(!console.is_terminal());
    }

    // -- New feature tests ---------------------------------------------------

    #[test]
    fn test_newline_renderable() {
        let nl = NewLine;
        let result = nl.render(&ConsoleOptions::default());
        let ansi = result.to_ansi();
        assert_eq!(ansi, "\n");
    }

    #[test]
    fn test_nochange_renderable() {
        let nc = NoChange;
        let result = nc.render(&ConsoleOptions::default());
        assert!(result.lines.is_empty());
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_capture_begin_end() {
        let mut console = Console::with_file(Box::new(std::io::sink()));
        console.begin_capture();
        let _ = write!(console.file, "captured text");
        let cap = console.end_capture().unwrap();
        assert_eq!(cap.get(), "captured text");
    }

    #[test]
    fn test_capture_with_closure() {
        let mut console = Console::with_file(Box::new(std::io::sink()));
        let output = console
            .capture(|c| {
                let _ = write!(c.file, "hello from capture");
            })
            .unwrap();
        assert_eq!(output, "hello from capture");
    }

    #[test]
    fn test_capture_new_empty() {
        let console = Console::new();
        let cap = Capture::new(&console);
        assert_eq!(cap.get(), "");
    }

    #[test]
    fn test_end_capture_not_capturing() {
        let mut console = Console::new();
        let result = console.end_capture();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CaptureError::NotCapturing);
    }

    #[test]
    fn test_system_pager_default() {
        let pager = SystemPager::new();
        // SystemPager should be constructable and show() should not panic
        // when called with empty content (even if pager command doesn't exist)
        let _ = pager.show("");
    }

    #[test]
    fn test_pager_enabled() {
        let pager = Pager::new();
        assert!(pager.is_enabled());
        let disabled = pager.enabled(false);
        assert!(!disabled.is_enabled());
    }

    #[test]
    fn test_render_hook() {
        let hook = RenderHook::new(|lines| {
            // Add a bold "HOOKED" segment to every line
            let hooked: Vec<Vec<Segment>> = lines
                .iter()
                .map(|line| {
                    let mut new_line = line.clone();
                    new_line.push(Segment::styled("HOOKED", Style::new().bold(true)));
                    new_line
                })
                .collect();
            hooked
        });
        let lines = vec![vec![Segment::new("test")]];
        let result = hook.apply(&lines);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[0][1].text, "HOOKED");
    }

    #[test]
    fn test_console_size() {
        let mut console = Console::new();
        console.set_size(100, 40);
        let dims = console.size();
        assert_eq!(dims.width, 100);
        assert_eq!(dims.height, 40);
    }

    #[test]
    fn test_console_is_dumb_terminal() {
        let console = Console::new();
        // In test environment, TERM is typically not "dumb"
        // Just verify it doesn't panic and returns a bool
        let _ = console.is_dumb_terminal();
    }

    #[test]
    fn test_console_is_alt_screen() {
        let mut console = Console::new();
        assert!(!console.is_alt_screen());
        console.alt_screen = true;
        assert!(console.is_alt_screen());
    }

    #[test]
    fn test_console_render_ansi() {
        let console = Console::new();
        let ansi = console.render_ansi("test");
        // Should return plain text if no style applied
        assert!(ansi.contains("test") || ansi.contains("\x1b["));
    }

    #[test]
    fn test_console_render_to_lines() {
        let console = Console::new();
        let opts = ConsoleOptions::default();
        let lines = console.render_to_lines(&"hello", &opts);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0][0].text, "hello");
    }

    #[test]
    fn test_console_input_renderable() {
        // input_renderable reads from stdin, which is hard to test
        // Verify the method signature compiles
    }

    #[test]
    fn test_console_print_exception_noop() {
        let mut console = Console::new();
        // Should not panic
        console.print_exception(None, 3);
    }

    #[test]
    fn test_console_render_hooks_push_pop() {
        let mut console = Console::new();
        let hook = RenderHook::new(|lines| lines.to_vec());
        console.push_render_hook(hook);
        assert_eq!(console.render_hooks.len(), 1);
        let popped = console.pop_render_hook();
        assert!(popped.is_some());
        assert!(console.render_hooks.is_empty());
    }

    #[test]
    fn test_console_reconfigure() {
        // Test that reconfigure doesn't panic
        reconfigure(Some(120), Some(40), None);
        reconfigure(None, None, Some(ColorSystem::Standard));
        // Reset
        reconfigure(None, None, None);
    }

    #[test]
    fn test_pager_context_write() {
        let pager = Pager::new().enabled(false);
        let mut ctx = PagerContext::new(pager);
        ctx.feed("test content");
        // Drop should not panic since pager is disabled
    }

    #[test]
    fn test_theme_context() {
        let mut console = Console::new();
        let custom_theme = Theme::new();
        let original = console.theme.clone();
        {
            let _ctx = console.use_theme(custom_theme);
            // Theme should be the custom one now
        }
        // After ctx drops, original theme should be restored
        assert_eq!(console.theme.styles.len(), original.styles.len());
    }
}
