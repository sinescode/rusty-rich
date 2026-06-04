//! # rusty-rich
//!
//! **Rich text and beautiful formatting in the terminal** — a Rust port of
//! the popular Python [Rich](https://github.com/Textualize/rich) library.
//!
//! Bring stunning terminal output to your Rust CLI tools with minimal code.
//! Supports TrueColor/256/16-color terminals, produces ANSI escape sequences,
//! and exports to HTML and SVG.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rusty_rich::{Console, Panel, Table, Column, Style, Color};
//!
//! let mut console = Console::new();
//!
//! // Inline markup
//! console.print_str("[bold green]Hello, [red]World![/red][/bold green]");
//!
//! // Panel with title
//! let panel = Panel::new("Content inside a box")
//!     .title("My Panel")
//!     .border_style(Style::new().color(Color::parse("cyan").unwrap()));
//! console.println(&panel);
//!
//! // Table with columns
//! let mut table = Table::new();
//! table.add_column(Column::new("Name"));
//! table.add_column(Column::new("Age"));
//! table.add_row_str(vec!["Alice".into(), "30".into()]);
//! table.add_row_str(vec!["Bob".into(), "25".into()]);
//! console.println(&table);
//! ```
//!
//! ## Feature Overview
//!
//! ### Core Primitives
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`color`] | 256 named colors, TrueColor/8-bit/Standard, RGB↔ANSI conversion, blending |
//! | [`style`] | 13 text attributes (bold/italic/underline/strike/…), links, style combination |
//! | [`segment`] | Smallest output unit: text + style + control codes; 9 utility functions |
//! | [`text`] | [`Text`] with [`Span`]-based styling, markup parsing, truncation, wrapping |
//! | [`cells`] | Unicode cell width utilities for CJK and emoji |
//! | [`measure`] | Width measurement protocol for layout negotiation |
//! | [`align`] | Horizontal (Left/Center/Right/Full) and vertical (Top/Middle/Bottom) alignment |
//! | [`markup`] | BBCode-like parser: `[bold red]text[/bold red]` |
//! | [`highlighter`] | Regex/Repr/ISO8601/JSON/Path text highlighters |
//! | [`ratio`] | Proportional space distribution with minimums and maximums |
//!
//! ### Console & Rendering
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`console`] | Central rendering engine: [`Console`], [`Renderable`] trait, [`RenderResult`], [`Group`] |
//! | [`theme`] | 170+ named style maps with stack-based inheritance via [`Theme`] and `ThemeStack` |
//! | [`screen`] | Full-screen alt-buffer via [`Screen`], [`ScreenContext`] RAII guard, [`ScreenUpdate`] |
//!
//! ### Layout & Renderables
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`panel`] | Bordered container with title, subtitle, 17 box styles |
//! | [`table`] | Tabular data with [`Column`], [`Cell`], colspan/rowspan, sections, 17 border styles |
//! | [`tree`] | Hierarchical tree with Unicode/ASCII guides |
//! | [`rule`] | Horizontal divider with optional centered title |
//! | [`columns`] | Side-by-side column layout with equal/expand options |
//! | [`layout`] | Recursive split-pane layout with ratio sizing and named regions |
//! | [`padding`] | CSS-style padding (1–4 values) |
//! | [`box_drawing`] | 17 box/border styles from ASCII to heavy double-lines |
//!
//! ### Dynamic Components
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`progress`] | Multi-task progress bars, [`TrackIterator`] for tracking iterables, [`ProgressFile`] |
//! | [`progress_columns`] | 11 column types: bar, text, spinner, time, file size, transfer speed, etc. |
//! | [`spinner`] | 55 animated spinners with case-insensitive name lookup via [`get_spinner`] |
//! | [`status`] | Animated spinner + status message with in-place refresh |
//! | [`live`] | Auto-updating region with alt-screen support and [`LiveWriter`] for output capture |
//!
//! ### Content Rendering
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`syntax`] | Syntax highlighting via syntect (100+ languages), lexer guessing, stylize_range |
//! | [`markdown`] | Markdown rendering via pulldown-cmark: headings, code, lists, blockquotes, tables, images |
//! | [`json`] | Pretty-printed JSON with syntax-highlighted keys/values |
//! | [`logging`] | [`RichHandler`] for the `log` crate with colored levels |
//! | [`log_render`] | Standalone [`LogRender`] formatter for log records as Rich tables |
//! | [`traceback`] | Rich exception tracebacks with locals, source code, frame suppression, panic hook |
//!
//! ### Interactive & Inspection
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`prompt`] | 5 prompt types: [`Prompt`], [`IntPrompt`], [`FloatPrompt`], [`Confirm`], [`Select`] |
//! | [`inspect`] | [`Inspect`] for structured object introspection with attribute/method tables |
//! | [`control`] | [`Control`] for composable terminal escape sequences (cursor, screen, title, bell) |
//!
//! ### Additional Renderables
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`pretty`] | [`Pretty`] printing with node tree traversal, [`pprint`], [`pretty_repr`] |
//! | [`emoji`] | 100+ `:shortcode:` → Unicode emoji replacement via [`Emoji`] |
//! | [`pager`] | System pager integration (`$PAGER`/`less`) with [`PagerContext`] RAII |
//! | [`bar`] | Horizontal [`BarChart`] with labels, colors, and auto-scaling |
//! | [`palette`] | [`Palette`] generation: gradient, rainbow, monochrome |
//! | [`ansi`] | [`AnsiDecoder`] — parse ANSI escape sequences into styled [`Text`] |
//! | [`constrain`] | [`Constrain`] — cap the maximum width of any renderable |
//! | [`styled`] | [`Styled`] — apply a style to all output of a renderable |
//! | [`containers`] | [`Lines`] and [`Renderables`] for grouping renderables |
//! | [`filesize`] | [`format_file_size`], [`format_transfer_speed`], `decimal` (SI units) |
//! | [`scope`] | [`render_scope`] / [`scope_summary`] for variable inspection |
//! | [`file_proxy`] | [`FileProxy`] — auto-refreshing file content display |
//! | [`diagnose`] | Error diagnostics — [`report`] and [`diagnose()`] |
//! | [`repr`] | [`RichRepr`] trait, [`repr_auto`] / [`rich_repr`] for custom pretty-printing |
//!
//! ### Export
//!
//! | Module | Provides |
//! |--------|----------|
//! | [`export`] | HTML, SVG, and text export with 4 preset terminal themes |
//!
//! ## Examples by Use Case
//!
//! ### Markdown with Tables
//!
//! ```rust,no_run
//! use rusty_rich::{render_markdown, Console};
//!
//! let md = render_markdown("# Report\n\n| Item | Qty |\n|------|-----|\n| A    | 10  |\n| B    | 5   |");
//! Console::new().println(&md);
//! ```
//!
//! ### Progress Bars
//!
//! ```rust,no_run
//! use rusty_rich::Progress;
//!
//! let mut progress = Progress::new();
//! let task = progress.add_task("Downloading...", Some(100.0));
//! // … in a loop: progress.update(task, n as f64);
//! println!("{}", progress.render(80));
//! ```
//!
//! ### Interactive Prompts
//!
//! ```rust,no_run
//! use rusty_rich::{Prompt, Confirm, IntPrompt, Select};
//!
//! let name = Prompt::ask_with("Enter name").unwrap();
//! let password = Prompt::new("Password").password(true).ask().unwrap();
//! let ok = Confirm::ask_with("Continue?", true).unwrap();
//! let age = IntPrompt::ask_with("Age").unwrap();
//! let choice = Select::new("Pick").choice("Red", "r").choice("Blue", "b").ask().unwrap();
//! ```
//!
//! ### Live Display with Writer
//!
//! ```rust,no_run
//! use rusty_rich::{Live, LiveWriter, Panel};
//! use std::io::Write;
//!
//! let mut live = Live::new(Panel::new("Starting...").title("Status"));
//! let mut writer = Live::create_writer();
//! live.start().unwrap();
//!
//! writeln!(writer, "Processing items...").unwrap();
//! live.update(Panel::new("Done!").title("Status")).unwrap();
//! live.stop().unwrap();
//! ```
//!
//! ### Traceback Panic Hook
//!
//! ```rust,no_run
//! use rusty_rich::traceback;
//!
//! // Install a global panic hook for rich tracebacks
//! traceback::install();
//! ```
//!
//! ### Object Inspection
//!
//! ```rust,no_run
//! use rusty_rich::Inspect;
//!
//! let value = vec![1, 2, 3];
//! let insp = Inspect::new(&value)
//!     .title("my_vec")
//!     .add_attr("len", "usize", "3")
//!     .add_method("push", "fn push(&mut self, value: T)")
//!     .methods(true);
//! ```
//!
//! ### Terminal Control Sequences
//!
//! ```rust,no_run
//! use rusty_rich::control::{Control, control_home, control_clear};
//!
//! let clear = Control::clear_home();
//! let title = Control::title("My App");
//! let bell = Control::bell();
//! ```
//!
//! ### Log Record Formatting
//!
//! ```rust,no_run
//! use rusty_rich::LogRender;
//!
//! let mut renderer = LogRender::new()
//!     .show_time(true)
//!     .show_level(true)
//!     .show_path(true);
//!
//! let record = renderer.render_log(
//!     Some("10:30:00"),
//!     "ERROR",
//!     "Connection refused",
//!     Some("src/main.rs"),
//!     Some(42),
//! );
//! ```
//!
//! ### Full-Screen Applications
//!
//! ```rust,no_run
//! use rusty_rich::{Console, Screen};
//!
//! let mut console = Console::new();
//! let mut screen = console.screen();
//! screen.enter();   // enters alt screen
//! // … render content …
//! screen.exit();    // restores terminal
//! ```
//!
//! ### HTML & SVG Export
//!
//! ```rust,no_run
//! use rusty_rich::{export_svg, ExportSvgOptions};
//!
//! let svg = export_svg(&ExportSvgOptions::default());
//! std::fs::write("output.svg", svg).unwrap();
//! ```
//!
//! ## Color & Style System
//!
//! 256 named colors via [`Color::parse`], plus hex/RGB constructors with automatic downgrade:
//!
//! ```rust
//! use rusty_rich::{Color, Style};
//!
//! // Named colors — 256 ANSI palette
//! let c = Color::parse("hot_pink").unwrap();
//! let c = Color::parse("steel_blue").unwrap();
//!
//! // Hex / RGB
//! let c = Color::from_hex("#FF6600").unwrap();
//! let c = Color::from_rgb(100, 200, 50);
//!
//! // Style with 13 attributes + links
//! let s = Style::new()
//!     .color(Color::parse("cyan").unwrap())
//!     .bgcolor(Color::parse("#1E1E2E").unwrap())
//!     .bold(true)
//!     .italic(true)
//!     .underline(true)
//!     .link("https://example.com");
//! ```
//!
//! ## Box Styles (17 built-in)
//!
//! ```text
//! BOX_ROUNDED        ╭─╮ │ │ ╰─╯     BOX_HEAVY       ┏━┓ ┃ ┃ ┗━┛
//! BOX_SQUARE         ┌─┐ │ │ └─┘     BOX_DOUBLE      ╔═╗ ║ ║ ╚═╝
//! BOX_ASCII          +-+ | | +-+     BOX_MARKDOWN    | Table style
//! ```
//!
//! ## Comparison with Python Rich
//!
//! rusty-rich achieves ~88% feature parity with Python Rich 14.x (475+ tests).
//!
//! ## Feature Flags
//!
//! No feature flags — all functionality is included by default. Dependencies are
//! carefully chosen for minimal compile times.
//!
//! ## Crate Organization
//!
//! The crate is organized into 6 module groups across 48 source files:
//!
//! - **Core** ([`color`], [`style`], [`segment`], [`text`], [`theme`],
//!   [`measure`], [`align`], [`markup`], [`ratio`], [`highlighter`],
//!   [`cells`], [`console`], [`box_drawing`])
//! - **Renderables** ([`panel`], [`table`], [`tree`], [`rule`],
//!   [`padding`], [`columns`], [`layout`], [`bar`], [`constrain`],
//!   [`styled`], [`containers`])
//! - **Dynamic** ([`prompt`], [`progress`], [`progress_columns`],
//!   [`spinner`], [`status`], [`live`], [`screen`], [`control`])
//! - **Content** ([`syntax`], [`markdown`], [`json`], [`logging`],
//!   [`log_render`], [`traceback`], [`pretty`], [`ansi`])
//! - **Inspection** ([`inspect`], [`scope`], [`repr`], [`diagnose()`])
//! - **Export & I/O** ([`export`], [`pager`], [`emoji`], [`palette`],
//!   [`filesize`], [`file_proxy`])
//!
//! Most commonly-used types are re-exported at the crate root for convenience.

// -- Core modules -----------------------------------------------------------

/// Unicode cell width utilities for CJK and emoji text measurement.
pub mod cells;
/// 256 named ANSI colors, TrueColor/8-bit/Standard, RGB↔ANSI conversion, blending.
pub mod color;
/// 13 text attributes (bold, italic, underline, …), links, style combination.
pub mod style;
/// Styled text unit with control codes — the smallest rendering primitive.
pub mod segment;
/// Styled text with `Span`-based markup and text manipulation utilities.
pub mod text;
/// Named style maps (170+ defaults) with stack-based inheritance.
pub mod theme;
/// Width measurement protocol for layout negotiation between renderables.
pub mod measure;
/// Horizontal and vertical alignment wrappers for renderables.
pub mod align;
/// BBCode-like markup parser: `[bold red]text[/bold red]`.
pub mod markup;
/// Proportional space distribution algorithms with minimums and maximums.
pub mod ratio;
/// Regex-based and repr-style text highlighters (requires `syntax-highlighting` feature).
#[cfg(feature = "syntax-highlighting")]
pub mod highlighter;
/// Central rendering engine — Console, Renderable trait, capture, export.
pub mod console;
/// 17 box/border drawing styles from ASCII to heavy Unicode double-lines.
pub mod box_drawing;

// -- Renderable components --------------------------------------------------

/// Bordered container with optional title, subtitle, and padding.
pub mod panel;
/// Tabular data with column definitions, colspan/rowspan, sections, and 17 border styles.
pub mod table;
/// Hierarchical tree with Unicode or ASCII branch guides.
pub mod tree;
/// Horizontal divider line with optional centered, left-, or right-aligned title.
pub mod rule;
/// CSS-style padding (1–4 side values) around any renderable.
pub mod padding;
/// Side-by-side column layout with equal-width and expand options.
pub mod columns;
/// Recursive split-pane layout engine with ratio sizing and named regions.
pub mod layout;

// -- Dynamic / animated components ------------------------------------------

/// Interactive prompts: string, int, float, confirm, select with password mode.
pub mod prompt;
/// Multi-task progress bars with configurable column layouts and iterable tracking.
pub mod progress;
/// 11 progress column types: bar, text, spinner, time, file size, transfer speed.
pub mod progress_columns;
/// 55 animated spinners with case-insensitive name-based lookup.
pub mod spinner;
/// Animated spinner with status message, in-place refresh via carriage return.
pub mod status;
/// Auto-updating display region with stdout/stderr capture via `LiveWriter`.
pub mod live;
/// Full-screen rendering, alternate screen buffer, and screen update helpers.
pub mod screen;

// -- Content rendering ------------------------------------------------------

/// Syntax highlighting via syntect (100+ languages, Sublime Text theme support).
#[cfg(feature = "syntax-highlighting")]
pub mod syntax;
/// Markdown rendering via pulldown-cmark: headings, code, lists, blockquotes, tables.
#[cfg(feature = "markdown")]
pub mod markdown;
/// Pretty-printed JSON with syntax-highlighted keys, strings, numbers, booleans.
pub mod json;
/// `RichHandler` for the `log` crate — colored log levels with file/line info.
pub mod logging;
/// Rich exception tracebacks with source code, locals, frame suppression, panic hook.
pub mod traceback;

// -- Additional renderables --------------------------------------------------

/// Pretty-printing for Rust data structures with tree traversal and syntax highlighting.
pub mod pretty;
/// Emoji shortcode replacement — `:smile:` → 😊.
pub mod emoji;
/// System pager integration — pipes output to `less` or `$PAGER`.
pub mod pager;
/// Constrain the maximum width of any renderable.
pub mod constrain;
/// Pre-styled renderable wrapper — applies a style to all output of a renderable.
pub mod styled;
/// Horizontal bar chart with labels, colors, and auto-scaling.
pub mod bar;
/// Human-readable file size and transfer speed formatting.
pub mod filesize;
/// Container renderables — `Lines` and `Renderables` for grouping output.
pub mod containers;
/// Color palette generation — gradients, rainbows, monochrome ramps.
pub mod palette;
/// Error diagnostics — rich-formatted error reporting.
pub mod diagnose;
/// ANSI escape sequence decoder — parse ANSI text into styled `Text`.
pub mod ansi;
/// Variable scope inspection — render name→value mappings as tables.
pub mod scope;
/// Auto-refreshing file content display — watches file for changes.
pub mod file_proxy;
/// Rich representation protocol — customizable pretty-printing for Rust types.
pub mod repr;
/// Terminal control sequence generation — cursor movement, screen, titles, bells.
pub mod control;
/// Object introspection — structured display of type info, attributes, and methods.
pub mod inspect;
/// Standalone log record formatter — renders log records as Rich tables.
pub mod log_render;

// -- Export -----------------------------------------------------------------

/// HTML, SVG, and plain-text export with 4 preset terminal color themes.
pub mod export;

// -- Re-exports for convenience ---------------------------------------------
// Most commonly-used types are re-exported at the crate root so you can write
// `use rusty_rich::Panel` instead of `use rusty_rich::panel::Panel`.

// -- Core types --------------------------------------------------------------

/// 256 named colors + TrueColor/8-bit support with automatic downgrade.
pub use color::Color;
/// What level of color the terminal supports (Standard, EightBit, TrueColor).
pub use color::ColorSystem;
/// How a [`Color`] is stored internally (Default, Standard, EightBit, TrueColor).
pub use color::ColorType;

/// Terminal text style — 13 attributes, fg/bg color, links, and combination logic.
pub use style::Style;
/// Stack of styles for nested markup — push/pop to track inheritance.
pub use style::StyleStack;

/// A piece of styled text with optional control codes — the smallest renderable unit.
pub use segment::Segment;
/// A collection of [`Segment`]s with convenience methods.
pub use segment::Segments;

/// Styled text with `Span` regions, markup support, and text manipulation methods.
pub use text::Text;
/// A styled region within a [`Text`] — defined by start/end offsets and a [`Style`].
pub use text::Span;

/// A named style map — look up styles by key (e.g. `"repr.number"`, `"markdown.h1"`).
pub use theme::Theme;

/// Horizontal alignment: `Left`, `Center`, `Right`, or `Full` (justified).
pub use align::AlignMethod;
/// Vertical alignment: `Top`, `Middle`, or `Bottom`.
pub use align::VerticalAlignMethod;
/// Wraps a renderable with horizontal and vertical alignment.
pub use align::Align;

/// Width measurement (minimum, maximum) returned by the layout protocol.
pub use measure::Measurement;

// -- Console -----------------------------------------------------------------

/// The central rendering engine — prints renderables, manages terminal state.
pub use console::Console;
/// Options passed to renderables during the rendering pass (width, height, overflow, etc.).
pub use console::ConsoleOptions;
/// Terminal size in character cells (width × height).
pub use console::ConsoleDimensions;
/// How text overflow is handled: `Fold`, `Crop`, `Ellipsis`, or `Ignore`.
pub use console::OverflowMethod;
/// Trait for types that can be rendered to terminal output.
pub use console::Renderable;
/// The result of rendering: lines of segments plus nested renderable items.
pub use console::RenderResult;
/// Either a [`Segment`] or a nested renderable — produced during rendering.
pub use console::RenderItem;
/// A type-erased, cloneable wrapper around any [`Renderable`].
pub use console::DynRenderable;
/// Renders multiple renderables sequentially as one unit.
pub use console::Group;
/// Returns a mutex-guarded reference to the global (singleton) [`Console`].
pub use console::get_console;
/// Drop-in replacement for `println!` that uses the global console.
pub use console::print_objects as print;
/// Print a markup string (e.g. `"[bold red]text[/bold red]"`) via the global console.
pub use console::print_str;
/// Pretty-print a `serde_json::Value` via the global console.
pub use console::print_json_val as print_json;

// -- Box drawing -------------------------------------------------------------

/// A box/border style defined by 32 corner and edge characters.
pub use box_drawing::BoxStyle;
#[doc(no_inline)]
pub use box_drawing::{
    BOX_ROUNDED, BOX_SQUARE, BOX_HEAVY, BOX_HEAVY_EDGE, BOX_HEAVY_HEAD,
    BOX_DOUBLE, BOX_DOUBLE_EDGE, BOX_SIMPLE, BOX_SIMPLE_HEAVY,
    BOX_MINIMAL, BOX_MINIMAL_HEAVY, BOX_ASCII, BOX_ASCII2,
    BOX_SQUARE_DOUBLE_HEAD, BOX_MINIMAL_DOUBLE_HEAD, BOX_SIMPLE_HEAD,
    BOX_ASCII_DOUBLE_HEAD,
};

// -- Renderables -------------------------------------------------------------

/// A bordered container with optional title, subtitle, border style, and padding.
pub use panel::Panel;
/// Tabular data widget with headers, footers, colspan/rowspan, and 17 box styles.
pub use table::Table;
/// A column definition for a [`Table`] — header, footer, width, alignment, ratio.
pub use table::Column;
/// A table cell with optional style, colspan, and rowspan.
pub use table::Cell;

/// A hierarchical tree with Unicode or ASCII branch guides.
pub use tree::Tree;
/// A horizontal divider line with optional centered, left-, or right-aligned title.
pub use rule::Rule;
/// CSS-style padding (1–4 side values) around any renderable.
pub use padding::Padding;
/// Padding dimension specification: 1, 2, or 4 values (like CSS).
pub use padding::PaddingDimensions;
/// Side-by-side column layout with equal-width and expand options.
pub use columns::Columns;
/// Recursive split-pane layout engine with ratio sizing.
pub use layout::Layout;
/// A node in the layout tree — either a `Split` or a `Leaf`.
pub use layout::LayoutNode;
/// Layout split direction: `Horizontal` (columns) or `Vertical` (rows).
pub use layout::Direction;
/// A screen region defined by x, y, width, and height.
pub use layout::Region;

// -- Progress ----------------------------------------------------------------

/// Multi-task progress display with configurable column layouts.
pub use progress::Progress;
/// A single progress bar — renders as a filled bar with percentage.
pub use progress::ProgressBar;
/// A file wrapper that tracks read progress via a task in [`Progress`].
pub use progress::ProgressFile;
/// A tracked task within a [`Progress`] display — holds description, total, completed.
pub use progress::Task;
/// An iterator wrapper that automatically advances a progress task on each iteration.
pub use progress::TrackIterator;

/// Trait for progress column types — renders one cell per task.
pub use progress_columns::ProgressColumn;
/// Shows a progress bar for each task.
pub use progress_columns::BarColumn;
/// Shows "completed/total" as file sizes with a separator.
pub use progress_columns::DownloadColumn;
/// Shows the completed file size for each task.
pub use progress_columns::FileSizeColumn;
/// Shows "completed/total" with raw numbers.
pub use progress_columns::MofNCompleteColumn;
/// Shows a spinner (animated during active, checkmark when finished).
pub use progress_columns::SpinnerColumn;
/// Shows the completion percentage for each task.
pub use progress_columns::TaskProgressColumn;
/// Shows a formatted text field from task metadata.
pub use progress_columns::TextColumn;
/// Shows the elapsed time for each task.
pub use progress_columns::TimeElapsedColumn;
/// Shows the estimated remaining time for each task.
pub use progress_columns::TimeRemainingColumn;
/// Shows the total file size for each task.
pub use progress_columns::TotalFileSizeColumn;
/// Shows the transfer speed for each task.
pub use progress_columns::TransferSpeedColumn;
/// Format a byte count as a human-readable size string.
pub use progress_columns::format_size;
/// Format a bytes-per-second rate as a human-readable speed string.
pub use progress_columns::format_speed;

// -- Spinners ----------------------------------------------------------------

/// An animated spinner — frames, interval, text, and style.
pub use spinner::Spinner;
/// A predefined spinner animation: slice of frame strings + frame interval.
pub use spinner::SpinnerFrames;
/// The default spinner (dots).
pub use spinner::DEFAULT_SPINNER;
/// Look up a spinner by name (case-insensitive).
pub use spinner::get_spinner;
/// All registered spinners as a slice of (name, spinner) pairs.
pub use spinner::SPINNERS;
#[doc(no_inline)]
pub use spinner::{
    SPINNER_ARC, SPINNER_ARROW, SPINNER_ARROW2, SPINNER_ARROW3,
    SPINNER_BOUNCING_BAR, SPINNER_BOUNCING_BALL,
    SPINNER_CHRISTMAS, SPINNER_CIRCLE, SPINNER_CLOCK,
    SPINNER_EARTH, SPINNER_GRENADE,
    SPINNER_GROW_HORIZONTAL, SPINNER_GROW_VERTICAL,
    SPINNER_HAMBURGER, SPINNER_HEARTS, SPINNER_MONKEY,
    SPINNER_NOISE, SPINNER_PONG, SPINNER_RUNNER, SPINNER_SHARK,
    SPINNER_TOGGLE, SPINNER_TRIANGLE, SPINNER_VERTICAL_BARS,
};

// -- Prompts -----------------------------------------------------------------

/// String input prompt with optional password mode and choice validation.
pub use prompt::Prompt;
/// Base configuration shared by all prompt types.
pub use prompt::PromptBase;
/// Error type for prompt operations: invalid response, I/O error, or cancellation.
pub use prompt::PromptError;
/// Integer input prompt — loops until a valid i64 is entered.
pub use prompt::IntPrompt;
/// Floating-point input prompt — loops until a valid f64 is entered.
pub use prompt::FloatPrompt;
/// Yes/no confirmation prompt with configurable default answer.
pub use prompt::Confirm;
/// Numbered-choice selection prompt — user picks from a list by number.
pub use prompt::Select;

/// An animated spinner with a status message, refreshed in-place via carriage return.
pub use status::Status;

/// Auto-updating display region — refreshs content at a configurable rate.
pub use live::Live;
/// A writer that captures output for display within a [`Live`] region.
pub use live::LiveWriter;

/// Full-screen renderable — fills the terminal and crops/pads to fit.
pub use screen::Screen;
/// RAII guard for the alternate screen buffer — enters on creation, exits on drop.
pub use screen::ScreenContext;
/// Wraps a renderable for screen updates at a specific x/y offset.
pub use screen::ScreenUpdate;

// -- Content rendering -------------------------------------------------------

/// Syntax-highlighted code block — language, theme, line numbers, word wrap.
#[cfg(feature = "syntax-highlighting")]
pub use syntax::Syntax;
/// Render a markdown string into a [`MarkdownRender`].
#[cfg(feature = "markdown")]
pub use markdown::render_markdown;
/// A markdown document renderable — headings, code, lists, blockquotes, tables.
#[cfg(feature = "markdown")]
pub use markdown::MarkdownRender;
/// Render a `serde_json::Value` into a [`JsonRender`].
pub use json::render_json;
/// Pretty-printed, syntax-highlighted JSON renderable.
pub use json::JsonRender;

/// A logging handler for the `log` crate — renders records with Rich styling.
pub use logging::RichHandler;

/// Rich exception traceback — box-drawn, with source code context and locals.
pub use traceback::Traceback;
/// A chain of exceptions — one or more [`Stack`]s of [`Frame`]s.
pub use traceback::Trace;
/// One exception level in a traceback — type, value, frames, notes.
pub use traceback::Stack;
/// A single stack frame — file, line number, function name, source line, locals.
pub use traceback::Frame;
/// Install a global panic hook that renders Rich-formatted tracebacks to stderr.
pub use traceback::install;

/// Trait for text highlighters — takes a [`Text`] and returns a styled [`Text`].
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::Highlighter;
/// Highlights Python-repr-like output: URLs, numbers, paths, quoted strings.
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::ReprHighlighter;
/// A no-op highlighter that returns text unchanged.
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::NullHighlighter;
/// Highlights text using regex patterns mapped to styles.
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::RegexHighlighter;

// -- Export ------------------------------------------------------------------

/// Export rendered output as a full HTML document.
pub use export::export_html;
/// Export rendered output as HTML and save to a file.
pub use export::save_html;
/// Options for HTML export: font, font size, line height, theme, code styling.
pub use export::ExportHtmlOptions;
/// Export rendered output as an SVG document with terminal chrome.
pub use export::export_svg;
/// Export rendered output as SVG and save to a file.
pub use export::save_svg;
/// Options for SVG export: font, font size, theme, dimensions, code styling.
pub use export::ExportSvgOptions;
/// Export rendered output as plain text (optionally strip ANSI escapes).
pub use export::export_text;
/// Export rendered output as plain text and save to a file.
pub use export::save_text;
/// Options for text export: text content and ANSI strip flag.
pub use export::ExportTextOptions;
/// A terminal color theme for HTML/SVG export (background, foreground, ANSI palette).
pub use export::ExportTheme;
/// Monokai export theme.
pub use export::EXPORT_THEME_MONOKAI;
/// Dimmed Monokai export theme.
pub use export::EXPORT_THEME_DIMMED_MONOKAI;
/// Night Owlish export theme.
pub use export::EXPORT_THEME_NIGHT_OWLISH;
/// SVG-optimized export theme.
pub use export::EXPORT_THEME_SVG;
/// Convert segments to HTML spans with inline CSS.
pub use export::segments_to_html;
/// Convert segments to SVG `<tspan>` elements with inline fill.
pub use export::segments_to_svg;
/// Escape text for safe HTML embedding.
pub use export::escape_html;
/// Strip ANSI escape sequences from a string.
pub use export::strip_ansi_escapes;
/// HTML document template string.
pub use export::CONSOLE_HTML_FORMAT;
/// SVG document template string.
pub use export::CONSOLE_SVG_FORMAT;

/// Parse a BBCode-like markup string and return a styled [`Text`].
pub use markup::render as render_markup;
/// Escape square brackets in a string so they are not interpreted as markup tags.
pub use markup::escape as escape_markup;

// -- New modules (Phase 2 additions) ------------------------------------------

// Pretty printing
pub use pretty::Pretty;
pub use pretty::Node as PrettyNode;
pub use pretty::pprint;
pub use pretty::pretty_repr;
pub use pretty::traverse;
pub use pretty::install as pretty_install;

// Emoji
pub use emoji::Emoji;
pub use emoji::NoEmoji;

// Pager
pub use pager::Pager;
pub use pager::PagerContext;
pub use pager::SystemPager;

// Constrain
pub use constrain::Constrain;

// Styled
pub use styled::Styled;

// Bar chart
pub use bar::Bar;
pub use bar::BarChart;

// File size
pub use filesize::format_size as format_file_size;
pub use filesize::format_speed as format_transfer_speed;
pub use filesize::pick_unit_and_suffix;

// Containers
pub use containers::Lines;
pub use containers::Renderables;

// Palette
pub use palette::Palette;

// Diagnose
pub use diagnose::report;
pub use diagnose::diagnose;

// ANSI decoder
pub use ansi::AnsiDecoder;

// Scope
pub use scope::render_scope;
pub use scope::scope_summary;

// File proxy
pub use file_proxy::FileProxy;

// Repr protocol
pub use repr::RichRepr;
pub use repr::auto as repr_auto;
pub use repr::rich_repr;
pub use repr::ReprOptions;
pub use repr::ReprError;

// Syntax additional exports
#[cfg(feature = "syntax-highlighting")]
pub use syntax::SyntaxTheme;
#[cfg(feature = "syntax-highlighting")]
pub use syntax::ANSISyntaxTheme;
#[cfg(feature = "syntax-highlighting")]
pub use syntax::get_lexer_by_name;
#[cfg(feature = "syntax-highlighting")]
pub use syntax::get_style_by_name;
#[cfg(feature = "syntax-highlighting")]
pub use syntax::guess_lexer_for_filename;

// Console additional exports
pub use console::Capture;
pub use console::NewLine;
pub use console::NoChange;
pub use console::RenderHook;
pub use console::ThemeContext;
pub use console::reconfigure;

// Layout additional exports
pub use layout::Splitter;
pub use layout::ColumnSplitter;
pub use layout::RowSplitter;
pub use layout::NoSplitter;

// Table additional exports
pub use table::Row;

// Progress additional exports
pub use progress::RenderableColumn;
pub use progress::track;
pub use progress::wrap_file;

// Highlighter additional exports
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::ISO8601Highlighter;
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::JSONHighlighter;
#[cfg(feature = "syntax-highlighting")]
pub use highlighter::PathHighlighter;

// Filesize additional exports
pub use filesize::decimal as format_size_decimal;

// Control exports
pub use control::Control;
pub use control::control_bell;
pub use control::control_home;
pub use control::control_clear;
pub use control::control_move_to;
pub use control::strip_control_codes;
pub use control::escape_control_codes;

// Inspect exports
pub use inspect::Inspect;
pub use inspect::inspect;
pub use inspect::inspect_str;

// LogRender exports
pub use log_render::LogRender;
pub use log_render::LogRecord;
pub use log_render::LogTable;
