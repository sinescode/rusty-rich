//! # rusty-rich
//!
//! **Rich text and beautiful formatting in the terminal** — a Rust port of
//! the popular Python [Rich](https://github.com/Textualize/rich) library.
//!
//! ## Features
//!
//! - 🎨 **Style**: foreground/background colors, bold, italic, underline,
//!   dim, blink, reverse, strikethrough
//! - 📝 **Console markup**: `[bold red]text[/bold red]` for inline styling
//! - 📊 **Table**: tabular data with headers, footers, column alignment
//! - 🌲 **Tree**: hierarchical tree rendering
//! - 📦 **Panel**: bordered containers with optional titles
//! - ➖ **Rule**: horizontal dividers with optional titles
//! - 📐 **Padding & Align**: spacing and alignment helpers
//! - 📋 **Columns**: side-by-side layout
//! - 🗂️ **Layout**: split-pane layout system
//! - ⏳ **Progress**: multi-task progress bars
//! - 🔄 **Spinner**: animated spinners
//! - 📌 **Status**: status messages with spinner
//! - 🔄 **Live**: auto-updating live displays
//! - 🖥️ **Screen**: full-screen rendering and alternate screen buffer
//! - 🌈 **Syntax highlighting**: powered by syntect (like Pygments)
//! - 📝 **Markdown**: rich markdown rendering
//! - 📋 **JSON**: pretty-printed, syntax-highlighted JSON
//! - 🔍 **Logging**: Rich-formatted log records
//! - 🖼️ **Box drawing**: 12 box styles (rounded, square, heavy, double, etc.)
//! - 🎯 **TrueColor / 256 / 16** color support with automatic detection
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rusty_rich::{
//!     Console, Panel, Table, Column, Rule, Tree,
//!     Style, Color, AlignMethod, Padding,
//! };
//!
//! fn main() {
//!     let mut console = Console::new();
//!
//!     // Print with markup
//!     console.print_str("[bold green]Hello, [red]World![/red][/bold green]");
//!
//!     // Create a panel
//!     let panel = Panel::new("Hello inside a rounded box!")
//!         .title("My Panel")
//!         .border_style(Style::new().color(Color::parse("cyan").unwrap()));
//!     console.println(&panel);
//!
//!     // Create a table
//!     let mut table = Table::new();
//!     table.add_column(Column::new("Name").justify(AlignMethod::Left));
//!     table.add_column(Column::new("Age").justify(AlignMethod::Right));
//!     table.add_row(vec!["Alice".into(), "30".into()]);
//!     table.add_row(vec!["Bob".into(), "25".into()]);
//!     console.println(&table);
//!
//!     // Create a tree
//!     let mut tree = Tree::new("Root");
//!     tree.add("Child 1").add("Grandchild");
//!     tree.add("Child 2");
//!     console.println(&tree);
//! }
//! ```

// -- Core modules -----------------------------------------------------------
pub mod cells;
pub mod color;
pub mod style;
pub mod segment;
pub mod text;
pub mod theme;
pub mod measure;
pub mod align;
pub mod markup;
pub mod ratio;
pub mod highlighter;
pub mod console;
pub mod box_drawing;

// -- Renderable components --------------------------------------------------
pub mod panel;
pub mod table;
pub mod tree;
pub mod rule;
pub mod padding;
pub mod columns;
pub mod layout;

// -- Dynamic / animated components ------------------------------------------
pub mod prompt;
pub mod progress;
pub mod progress_columns;
pub mod spinner;
pub mod status;
pub mod live;
pub mod screen;

// -- Content rendering ------------------------------------------------------
pub mod syntax;
pub mod markdown;
pub mod json;
pub mod logging;
pub mod traceback;

// -- Export -----------------------------------------------------------------
pub mod export;

// -- Re-exports for convenience ---------------------------------------------

// Core types
pub use color::Color;
pub use color::ColorSystem;
pub use color::ColorType;

pub use style::Style;
pub use style::StyleStack;

pub use segment::Segment;
pub use segment::Segments;

pub use text::Text;
pub use text::Span;

pub use theme::Theme;

pub use align::AlignMethod;
pub use align::VerticalAlignMethod;
pub use align::Align;

pub use measure::Measurement;

// Console
pub use console::Console;
pub use console::ConsoleOptions;
pub use console::ConsoleDimensions;
pub use console::OverflowMethod;
pub use console::Renderable;
pub use console::RenderResult;
pub use console::RenderItem;
pub use console::DynRenderable;
pub use console::Group;
pub use console::get_console;
pub use console::print_objects as print;
pub use console::print_str;
pub use console::print_json_val as print_json;

// Box drawing
pub use box_drawing::BoxStyle;
pub use box_drawing::{
    BOX_ROUNDED, BOX_SQUARE, BOX_HEAVY, BOX_HEAVY_EDGE, BOX_HEAVY_HEAD,
    BOX_DOUBLE, BOX_DOUBLE_EDGE, BOX_SIMPLE, BOX_SIMPLE_HEAVY,
    BOX_MINIMAL, BOX_MINIMAL_HEAVY, BOX_ASCII, BOX_ASCII2,
    BOX_SQUARE_DOUBLE_HEAD, BOX_MINIMAL_DOUBLE_HEAD, BOX_SIMPLE_HEAD,
    BOX_ASCII_DOUBLE_HEAD,
};

// Renderables
pub use panel::Panel;
pub use table::{Table, Column, Cell};
pub use tree::Tree;
pub use rule::Rule;
pub use padding::{Padding, PaddingDimensions};
pub use columns::Columns;
pub use layout::{Layout, LayoutNode, Direction, Region};

pub use progress::{Progress, ProgressBar, ProgressFile, Task, TrackIterator};
pub use progress_columns::{
    BarColumn, DownloadColumn, FileSizeColumn, MofNCompleteColumn,
    ProgressColumn, SpinnerColumn, TaskProgressColumn, TextColumn,
    TimeElapsedColumn, TimeRemainingColumn, TotalFileSizeColumn,
    TransferSpeedColumn, format_size, format_speed,
};
pub use spinner::{
    Spinner, SpinnerFrames, DEFAULT_SPINNER, get_spinner,
    SPINNER_ARC, SPINNER_ARROW, SPINNER_ARROW2, SPINNER_ARROW3,
    SPINNER_BOUNCING_BAR, SPINNER_BOUNCING_BALL,
    SPINNER_CHRISTMAS, SPINNER_CIRCLE, SPINNER_CLOCK,
    SPINNER_EARTH, SPINNER_GRENADE,
    SPINNER_GROW_HORIZONTAL, SPINNER_GROW_VERTICAL,
    SPINNER_HAMBURGER, SPINNER_HEARTS, SPINNER_MONKEY,
    SPINNER_NOISE, SPINNER_PONG, SPINNER_RUNNER, SPINNER_SHARK,
    SPINNER_TOGGLE, SPINNER_TRIANGLE, SPINNER_VERTICAL_BARS,
    SPINNERS,
};
pub use prompt::{
    Prompt, PromptBase, PromptError, IntPrompt, FloatPrompt, Confirm, Select,
};
pub use status::Status;
pub use live::{Live, LiveWriter};
pub use screen::Screen;
pub use screen::ScreenContext;
pub use screen::ScreenUpdate;

pub use syntax::Syntax;
pub use markdown::render_markdown;
pub use markdown::MarkdownRender;
pub use json::render_json;
pub use json::JsonRender;

pub use logging::RichHandler;
pub use traceback::{Traceback, Trace, Stack, Frame, install};
pub use highlighter::{Highlighter, ReprHighlighter, NullHighlighter, RegexHighlighter};

pub use export::{
    export_html, save_html, ExportHtmlOptions,
    export_svg, save_svg, ExportSvgOptions,
    export_text, save_text, ExportTextOptions,
    ExportTheme,
    EXPORT_THEME_MONOKAI, EXPORT_THEME_DIMMED_MONOKAI,
    EXPORT_THEME_NIGHT_OWLISH, EXPORT_THEME_SVG,
    segments_to_html, escape_html, strip_ansi_escapes,
    CONSOLE_HTML_FORMAT, CONSOLE_SVG_FORMAT,
};

pub use markup::{render as render_markup, escape as escape_markup};
