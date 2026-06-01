//! Console — the central rendering engine. Equivalent to Rich's `console.py`.
//!
//! The `Console` is the main entry point for rendering. It manages terminal
//! detection, color system support, and dispatching renderables to produce
//! styled output.

use std::fmt;
use std::io::{self, Write};
use std::sync::Arc;

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
    pub fn update_width(&self, max_width: usize) -> Self {
        let mut opts = self.clone();
        opts.max_width = max_width;
        opts
    }

    /// Update the height.
    pub fn update_height(&self, height: usize) -> Self {
        let mut opts = self.clone();
        opts.height = Some(height);
        opts
    }

    /// Shrink the max width by an amount (for padding).
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
    fn from(s: Segment) -> Self { Self::Segment(s) }
}

impl From<DynRenderable> for RenderItem {
    fn from(r: DynRenderable) -> Self { Self::Nested(r) }
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

impl RenderResult {
    /// Create an empty [`RenderResult`].
    pub fn new() -> Self {
        Self { lines: Vec::new(), items: Vec::new() }
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
        let items: Vec<RenderItem> = segments.iter().map(|s| RenderItem::Segment(s.clone())).collect();
        Self { lines: vec![segments], items }
    }

    /// Create a [`RenderResult`] from pre-computed lines of [`Segment`]s.
    pub fn from_lines(lines: Vec<Vec<Segment>>) -> Self {
        Self { lines, items: Vec::new() }
    }

    /// Create a [`RenderResult`] from [`RenderItem`]s for recursive flattening.
    pub fn from_items(items: Vec<RenderItem>) -> Self {
        Self { lines: Vec::new(), items }
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
        let lines: Vec<Vec<Segment>> = rendered
            .lines()
            .map(|l| vec![Segment::new(l)])
            .collect();
        RenderResult { lines, items: Vec::new() }
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
}

/// A renderable that renders multiple children one after another (vertically).
///
/// Equivalent to Python Rich's `Group`.
#[derive(Debug, Clone)]
pub struct Group {
    /// The child renderables to render in sequence.
    pub children: Vec<DynRenderable>,
}

impl Group {
    /// Create an empty [`Group`].
    pub fn new() -> Self {
        Self { children: Vec::new() }
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
        RenderResult { lines: all_lines, items: Vec::new() }
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
}

impl Console {
    /// Create a new Console writing to stdout.
    pub fn new() -> Self {
        let is_terminal = atty::is(atty::Stream::Stdout);
        let color_system = detect_color_system();

        let size = ConsoleDimensions::detect();

        Self {
            file: Box::new(io::stdout()) as Box<dyn Write + Send>,
            color_system,
            theme: crate::theme::default_theme(),
            options: ConsoleOptions {
                size,
                is_terminal,
                max_width: size.width,
                max_height: size.height,
                ..Default::default()
            },
            width: None,
            height: None,
            is_terminal,
            quiet: false,
            soft_wrap: false,
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
                size: ConsoleDimensions { width: 80, height: 25 },
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
        self.theme
            .get(name)
            .cloned()
            .or_else(|| {
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
        if self.quiet { return; }
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
    pub fn println(&mut self, renderable: &dyn Renderable) {
        if self.quiet { return; }
        let result = renderable.render(&self.options);
        let ansi = result.to_ansi();
        let _ = writeln!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Print a plain string (supports markup by default when `markup` is
    /// enabled).
    pub fn print_str(&mut self, text: &str) {
        if self.quiet { return; }
        let ansi = if self.options.markup {
            let parsed = crate::markup::render(text);
            parsed.render()
        } else {
            text.to_string()
        };
        let _ = write!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Print formatted JSON.
    pub fn print_json(&mut self, data: &serde_json::Value) {
        if self.quiet { return; }
        let formatted = crate::json::render_json(data);
        let result = formatted.render(&self.options);
        let ansi = result.to_ansi();
        let _ = writeln!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Clear the screen.
    pub fn clear(&mut self) {
        if self.quiet { return; }
        let _ = write!(self.file, "\x1b[2J\x1b[H");
        let _ = self.file.flush();
    }

    /// Show the cursor.
    pub fn show_cursor(&mut self) {
        let _ = write!(self.file, "\x1b[?25h");
        let _ = self.file.flush();
    }

    /// Hide the cursor.
    pub fn hide_cursor(&mut self) {
        let _ = write!(self.file, "\x1b[?25l");
        let _ = self.file.flush();
    }

    /// Set the terminal window title.
    pub fn set_window_title(&mut self, title: &str) {
        let _ = write!(self.file, "\x1b]0;{title}\x07");
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
    pub fn measure(&self, renderable: &dyn Renderable, options: &ConsoleOptions) -> crate::measure::Measurement {
        if let Some(m) = renderable.measure(options) {
            return m;
        }
        let segments = self.render(renderable, options);
        let max_w = segments.iter()
            .map(|s| s.cell_length())
            .max()
            .unwrap_or(0);
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
        if self.quiet { return; }
        let mut rule = crate::rule::Rule::new().title(title);
        if let Some(chars) = characters { rule = rule.characters(chars); }
        if let Some(st) = style { rule = rule.style(st); }
        if let Some(a) = align { rule = rule.align(a); }
        let result = rule.render(&self.options);
        let ansi = result.to_ansi();
        let _ = write!(self.file, "{ansi}");
        let _ = self.file.flush();
    }

    /// Output a bell character.
    pub fn bell(&mut self) {
        if self.quiet { return; }
        let _ = write!(self.file, "\x07");
        let _ = self.file.flush();
    }

    /// Output blank lines.
    pub fn line(&mut self, count: usize) {
        if self.quiet { return; }
        for _ in 0..count {
            let _ = writeln!(self.file);
        }
        let _ = self.file.flush();
    }

    /// Output a log entry with timestamp, caller info.
    pub fn log(&mut self, objects: &[&dyn Renderable]) {
        if self.quiet { return; }
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
    pub fn export_html(&self, renderable: &dyn Renderable) -> String {
        let result = renderable.render(&self.options);
        let ansi = result.to_ansi();
        crate::export::export_html(&crate::export::ExportHtmlOptions {
            code: crate::export::strip_ansi_escapes(&ansi),
            ..Default::default()
        })
    }

    /// Save rendered output as an HTML file.
    pub fn save_html(&self, path: impl AsRef<std::path::Path>, renderable: &dyn Renderable) -> std::io::Result<()> {
        let html = self.export_html(renderable);
        crate::export::save_html(path, &crate::export::ExportHtmlOptions {
            code: html,
            ..Default::default()
        })
    }

    /// Export the current console output as an SVG document.
    pub fn export_svg(&self, renderable: &dyn Renderable) -> String {
        let result = renderable.render(&self.options);
        let ansi = result.to_ansi();
        crate::export::export_svg(&crate::export::ExportSvgOptions {
            code: crate::export::strip_ansi_escapes(&ansi),
            ..Default::default()
        })
    }

    /// Save rendered output as an SVG file.
    pub fn save_svg(&self, path: impl AsRef<std::path::Path>, renderable: &dyn Renderable) -> std::io::Result<()> {
        let svg = self.export_svg(renderable);
        crate::export::save_svg(path, &crate::export::ExportSvgOptions {
            code: svg,
            ..Default::default()
        })
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
    pub fn save_text(&self, path: impl AsRef<std::path::Path>, renderable: &dyn Renderable) -> std::io::Result<()> {
        let text = self.export_text(renderable);
        crate::export::save_text(path, &crate::export::ExportTextOptions {
            text,
            strip_ansi: false,
        })
    }

    // -- Context manager equivalent -----------------------------------------

    /// Enter a capture context. Returns the Console back so it can be
    /// used inside a block. Call `end_capture()` to get the captured text.
    pub fn begin_capture(&mut self) {
        // In Rust, capture would need to swap the file with a buffer.
        // For now, this is a no-op placeholder.
    }

    /// End capture and return captured text.
    pub fn end_capture(&mut self) -> String {
        String::new() // placeholder
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

                loop {
                    match handle.read_exact(&mut buf) {
                        Ok(()) => match buf[0] {
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
                        },
                        Err(_) => break,
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
    /// escape sequences (`\x1b[?1049h` / `\x1b[?1049l`).
    pub fn set_alt_screen(&mut self, enable: bool) {
        if enable {
            let _ = write!(self.file, "\x1b[?1049h");
        } else {
            let _ = write!(self.file, "\x1b[?1049l");
        }
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
            .finish()
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
    if atty::is(atty::Stream::Stdout) {
        ColorSystem::TrueColor
    } else {
        ColorSystem::Standard
    }
}

// ---------------------------------------------------------------------------
// Global console instance (like Rich's `get_console()`)
// ---------------------------------------------------------------------------

use std::sync::Mutex;
use once_cell::sync::Lazy;

static GLOBAL_CONSOLE: Lazy<Mutex<Console>> = Lazy::new(|| {
    Mutex::new(Console::new())
});

/// Get a reference to the global Console.
pub fn get_console() -> std::sync::MutexGuard<'static, Console> {
    GLOBAL_CONSOLE.lock().unwrap()
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
        assert_eq!(detected, atty::is(atty::Stream::Stdout));
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
}
