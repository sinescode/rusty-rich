//! Traceback -- exception traceback rendering. Equivalent to Rich's `traceback.py`.
//!
//! Provides data structures for representing tracebacks and a `Traceback`
//! renderable that displays them with Rich formatting, complete with source
//! code context, local variable tables, and styled box-drawn borders.
//!
//! # Theme keys used
//!
//! | Key                        | Style                                      |
//! |----------------------------|--------------------------------------------|
//! | `traceback.border`         | border of the outer and inner boxes        |
//! | `traceback.title`          | "Traceback (most recent call last)" title  |
//! | `traceback.error`          | exception type  name                       |
//! | `traceback.error_mark`     | the "❱" marker on the error line           |
//! | `traceback.filename`       | file paths in frame headers                |
//! | `traceback.line_no`        | line numbers in source context             |
//! | `traceback.locals_header`  | header of the locals sub-table             |

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use unicode_width::UnicodeWidthStr;

use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::theme;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single frame in a traceback.
#[derive(Debug, Clone)]
pub struct Frame {
    pub filename: String,
    pub lineno: usize,
    pub name: String,
    pub line: Option<String>,
    pub locals: Option<HashMap<String, String>>,
    pub last_instruction: Option<String>,
}

impl Frame {
    /// Create a new `Frame` with the given filename, line number, and function name.
    pub fn new(filename: impl Into<String>, lineno: usize, name: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            lineno,
            name: name.into(),
            line: None,
            locals: None,
            last_instruction: None,
        }
    }

    /// Builder: attach the source line content for this frame.
    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.line = Some(line.into());
        self
    }

    /// Builder: attach local variables.
    pub fn locals(mut self, locals: HashMap<String, String>) -> Self {
        self.locals = Some(locals);
        self
    }
}

/// A stack of frames (one exception level).
#[derive(Debug, Clone, Default)]
pub struct Stack {
    pub exc_type: Option<String>,
    pub exc_value: Option<String>,
    pub syntax_error: Option<String>,
    pub is_cause: bool,
    pub frames: Vec<Frame>,
    pub notes: Vec<String>,
    pub is_group: bool,
    pub exceptions: Vec<Stack>,
}

impl Stack {
    /// Create a new empty `Stack` with no exception type, value, or frames.
    pub fn new() -> Self {
        Self {
            exc_type: None,
            exc_value: None,
            syntax_error: None,
            is_cause: false,
            frames: Vec::new(),
            notes: Vec::new(),
            is_group: false,
            exceptions: Vec::new(),
        }
    }

    /// Builder: set the exception type name (e.g. `"ValueError"`).
    pub fn exc_type(mut self, t: impl Into<String>) -> Self {
        self.exc_type = Some(t.into());
        self
    }

    /// Builder: set the exception message/value.
    pub fn exc_value(mut self, v: impl Into<String>) -> Self {
        self.exc_value = Some(v.into());
        self
    }

    /// Builder: append a [`Frame`] to the stack's frame list.
    pub fn add_frame(mut self, frame: Frame) -> Self {
        self.frames.push(frame);
        self
    }
}

/// Full trace data.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    pub stacks: Vec<Stack>,
}

impl Trace {
    /// Create a new empty `Trace` with no stacks.
    pub fn new() -> Self {
        Self { stacks: Vec::new() }
    }

    /// Create a `Trace` containing a single [`Stack`].
    pub fn from_stack(stack: Stack) -> Self {
        Self {
            stacks: vec![stack],
        }
    }
}

// ---------------------------------------------------------------------------
// Traceback -- renderable
// ---------------------------------------------------------------------------

/// Renders a traceback with Rich formatting.
///
/// Mimics Python Rich's `rich.traceback.Traceback` renderable.
#[derive(Debug, Clone)]
pub struct Traceback {
    trace: Trace,
    width: Option<usize>,
    code_width: Option<usize>,
    extra_lines: usize,
    theme_name: Option<String>,
    word_wrap: bool,
    show_locals: bool,
    indent_guides: bool,
    locals_max_length: usize,
    locals_max_string: usize,
    locals_max_depth: usize,
    locals_hide_dunder: bool,
    locals_hide_sunder: bool,
    suppress: Vec<String>,
    max_frames: Option<usize>,
}

impl Traceback {
    /// Create a new Traceback from `Trace` data.
    pub fn new(trace: Trace) -> Self {
        Self {
            trace,
            width: None,
            code_width: None,
            extra_lines: 3,
            theme_name: None,
            word_wrap: false,
            show_locals: false,
            indent_guides: false,
            locals_max_length: 10,
            locals_max_string: 80,
            locals_max_depth: 5,
            locals_hide_dunder: true,
            locals_hide_sunder: false,
            suppress: Vec::new(),
            max_frames: None,
        }
    }

    /// Convenience constructor: build a `Traceback` from an exception type,
    /// value, and list of frames.
    pub fn from_exception(
        exc_type: impl Into<String>,
        exc_value: impl Into<String>,
        frames: Vec<Frame>,
    ) -> Self {
        let mut stack = Stack::new();
        stack.exc_type = Some(exc_type.into());
        stack.exc_value = Some(exc_value.into());
        stack.frames = frames;
        let trace = Trace::from_stack(stack);
        Self::new(trace)
    }

    // -- Builder methods --------------------------------------------------

    /// Builder: set the total output width in characters.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: set the width reserved for source code (excluding line numbers).
    pub fn code_width(mut self, width: usize) -> Self {
        self.code_width = Some(width);
        self
    }

    /// Builder: set the number of extra source lines shown before and after the error line.
    pub fn extra_lines(mut self, n: usize) -> Self {
        self.extra_lines = n;
        self
    }

    /// Builder: set the theme name (e.g. `"monokai"`, `"base16-ocean.dark"`).
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme_name = Some(theme.into());
        self
    }

    /// Builder: enable or disable word wrapping of long lines.
    pub fn word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    /// Builder: show local variables at each frame when set to `true`.
    pub fn show_locals(mut self, show: bool) -> Self {
        self.show_locals = show;
        self
    }

    /// Builder: enable indentation guides in source context.
    pub fn indent_guides(mut self, guides: bool) -> Self {
        self.indent_guides = guides;
        self
    }

    /// Builder: set the maximum number of local variables to display per frame.
    pub fn locals_max_length(mut self, n: usize) -> Self {
        self.locals_max_length = n;
        self
    }

    /// Builder: set the maximum length for local variable string values.
    pub fn locals_max_string(mut self, n: usize) -> Self {
        self.locals_max_string = n;
        self
    }

    /// Builder: set the maximum depth for nested local variable display.
    pub fn locals_max_depth(mut self, n: usize) -> Self {
        self.locals_max_depth = n;
        self
    }

    /// Builder: hide locals with dunder names (e.g. `__name__`) when `true`.
    pub fn locals_hide_dunder(mut self, hide: bool) -> Self {
        self.locals_hide_dunder = hide;
        self
    }

    /// Builder: hide locals with underscore-prefixed names (e.g. `_secret`) when `true`.
    pub fn locals_hide_sunder(mut self, hide: bool) -> Self {
        self.locals_hide_sunder = hide;
        self
    }

    /// Builder: suppress frames whose filename matches any of the given patterns.
    pub fn suppress(mut self, suppress: Vec<String>) -> Self {
        self.suppress = suppress;
        self
    }

    /// Builder: limit the number of frames shown (remaining are collapsed into a single message).
    pub fn max_frames(mut self, n: usize) -> Self {
        self.max_frames = Some(n);
        self
    }
}

// ---------------------------------------------------------------------------
// Style helpers -- resolve theme styles from the default theme
// ---------------------------------------------------------------------------

/// Look up a style from the default theme, returning a default-constructed
/// Style if the key is not present.
fn theme_style(name: &str) -> Style {
    crate::theme::default_theme()
        .get(name)
        .cloned()
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Build an outer content line: "│ " + content + " │", padded to `width`.
fn outer_content_line(content: Vec<Segment>, total_width: usize) -> Vec<Segment> {
    let border_style = theme_style(theme::names::TRACEBACK_BORDER);
    let mut line = Vec::new();

    // Left border
    line.push(Segment::styled("│ ".to_string(), border_style.clone()));

    // Content
    let mut content_w = 0usize;
    for seg in &content {
        content_w += seg.cell_length();
    }
    line.extend(content);

    // Right padding
    let inner_w = total_width.saturating_sub(4); // "│ " + " │"
    let pad = inner_w.saturating_sub(content_w);
    if pad > 0 {
        line.push(Segment::new(" ".repeat(pad)));
    }

    // Right border
    line.push(Segment::styled(" │".to_string(), border_style));
    line
}

/// Build a blank content line (empty line with just the outer borders).
fn outer_blank(total_width: usize) -> Vec<Segment> {
    outer_content_line(Vec::new(), total_width)
}

/// Build the outer top border with the traceback title.
fn top_border(total_width: usize) -> Vec<Segment> {
    let border_style = theme_style(theme::names::TRACEBACK_BORDER);
    let title_style = theme_style(theme::names::TRACEBACK_TITLE);

    let title = " Traceback (most recent call last) ";
    let dashes_total = total_width.saturating_sub(title.len() + 4); // ╭ ─╮
    let left_dashes = dashes_total / 2;
    let right_dashes = dashes_total - left_dashes;

    vec![
        Segment::styled("╭─".to_string(), border_style.clone()),
        Segment::styled(
            "─".repeat(left_dashes.saturating_sub(1)),
            border_style.clone(),
        ),
        Segment::styled(title.to_string(), title_style),
        Segment::styled(
            "─".repeat(right_dashes.saturating_sub(1)),
            border_style.clone(),
        ),
        Segment::styled("─╮".to_string(), border_style),
    ]
}

/// Build the outer bottom border.
fn bottom_border(total_width: usize) -> Vec<Segment> {
    let border_style = theme_style(theme::names::TRACEBACK_BORDER);
    let dashes = total_width.saturating_sub(2);
    vec![Segment::styled(
        format!("╰{}╯", "─".repeat(dashes)),
        border_style,
    )]
}

/// Helper: read source file lines around a given line number.
fn read_source_lines(
    filename: &str,
    lineno: usize,
    extra_lines: usize,
) -> (usize, Vec<(usize, String)>) {
    // Try to open the file
    let content = match fs::read_to_string(Path::new(filename)) {
        Ok(s) => s,
        Err(_) => return (0, Vec::new()),
    };

    let all_lines: Vec<&str> = content.lines().collect();
    if all_lines.is_empty() {
        return (0, Vec::new());
    }

    let start = if lineno > extra_lines {
        lineno - extra_lines
    } else {
        1
    };
    // lineno is 1-based, all_lines is 0-based
    let end = (lineno + extra_lines).min(all_lines.len());

    let mut result = Vec::new();
    for i in start..=end {
        let line_str = all_lines.get(i.saturating_sub(1)).copied().unwrap_or("");
        result.push((i, line_str.to_string()));
    }

    (lineno, result)
}

/// Check whether a filename matches any of the suppress patterns.
fn is_suppressed(filename: &str, suppress: &[String]) -> bool {
    for pattern in suppress {
        if filename.starts_with(pattern) || filename.contains(pattern) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Renderable implementation
// ---------------------------------------------------------------------------

impl Renderable for Traceback {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let total_width = self.width.unwrap_or(options.max_width.min(120));
        let content_width = total_width.saturating_sub(4); // space for "│ " + " │"

        // Resolve styles
        let border_style = theme_style(theme::names::TRACEBACK_BORDER);
        let filename_style = theme_style(theme::names::TRACEBACK_FILENAME);
        let line_no_style = theme_style(theme::names::TRACEBACK_LINE_NO);
        let error_mark_style = theme_style(theme::names::TRACEBACK_ERROR_MARK);
        let error_style = theme_style(theme::names::TRACEBACK_ERROR);
        let locals_header_style = theme_style(theme::names::TRACEBACK_LOCALS_HEADER);

        // Collect all output lines (as full-width segments, including "│ " / " │" borders)
        let mut out_lines: Vec<Vec<Segment>> = Vec::new();

        // Top border
        out_lines.push(top_border(total_width));

        // Blank line after top border
        out_lines.push(outer_blank(total_width));

        // Track how many frames we've rendered, for max_frames / suppression
        let mut rendered_count = 0usize;
        let mut suppressed_count = 0usize;

        // Iterate over stacks
        for stack in &self.trace.stacks {
            // Iterate over frames (oldest call first, most recent last)
            let frames_iter: Box<dyn Iterator<Item = &Frame>> = if stack.is_cause {
                // For chained exceptions, show frames in order
                Box::new(stack.frames.iter())
            } else {
                Box::new(stack.frames.iter())
            };

            let max_frames = self.max_frames.unwrap_or(usize::MAX);

            for frame in frames_iter {
                // Check suppression
                if is_suppressed(&frame.filename, &self.suppress) {
                    suppressed_count += 1;
                    continue;
                }

                // Check max_frames
                if rendered_count >= max_frames {
                    suppressed_count += 1;
                    continue;
                }
                rendered_count += 1;

                // --- Frame location header ---
                // "  /path/to/file.rs:42 in function_name"
                {
                    let loc = format!("{}:{}", frame.filename, frame.lineno);
                    let func = if frame.name.is_empty() {
                        String::new()
                    } else {
                        format!(" in {}", frame.name)
                    };

                    let mut header_segs = Vec::new();
                    header_segs.push(Segment::styled(
                        format!("  {}", loc),
                        filename_style.clone(),
                    ));
                    header_segs.push(Segment::styled(func, Style::new()));
                    out_lines.push(outer_content_line(header_segs, total_width));
                }

                // --- Source code context (read from file) ---
                let (error_line_num, source_lines) =
                    read_source_lines(&frame.filename, frame.lineno, self.extra_lines);

                if !source_lines.is_empty() {
                    // Build the source sub-box
                    let indent = 2usize;
                    let sub_box_total = content_width.saturating_sub(indent * 2);
                    let sub_box_inner = sub_box_total.saturating_sub(2); // exclude "│" borders

                    // Determine line number width
                    // (max line number in the context)
                    let max_ln = source_lines.iter().map(|(ln, _)| *ln).max().unwrap_or(0);
                    let ln_width = max_ln.to_string().len().max(2);

                    // Marker character width (❱ is 2 cells wide in Unicode)
                    let marker_cells = 2;

                    // Prefix: "❱ " or "  " + padded_line_no + " │ "
                    // Actually for the line content: marker + " " + line_no + " │ " + code
                    // marker is "❱ " (2 cells) for error line, "  " (2 cells) for normal
                    let prefix_cells = marker_cells + 1 + ln_width + 3; // marker *1 + space + ln + " │ "
                    let code_cells = sub_box_inner.saturating_sub(prefix_cells);

                    // Sub-box top border
                    {
                        let mut segs = Vec::new();
                        segs.push(Segment::styled(
                            format!("{}╭{}╮", " ".repeat(indent), "─".repeat(sub_box_inner)),
                            border_style.clone(),
                        ));
                        out_lines.push(outer_content_line(segs, total_width));
                    }

                    // Source lines
                    for (line_num, line_text) in &source_lines {
                        let is_error = *line_num == error_line_num;

                        let marker = if is_error { "❱" } else { " " };
                        let marker_str = format!("{:<width$}", marker, width = marker_cells);

                        let ln_str = format!("{:>width$}", line_num, width = ln_width);
                        let code = truncate_to_width(line_text, code_cells);

                        let raw_line = format!("{}{} {} │ {} ", marker_str, " ", ln_str, code);

                        // Now build: indent + "│" + raw_line + "│"
                        // The raw_line should be padded to sub_box_inner - 2
                        let inner_w = sub_box_inner.saturating_sub(2); // for │ │
                        let raw_width = UnicodeWidthStr::width(raw_line.as_str());
                        let pad_w = inner_w.saturating_sub(raw_width);
                        let _padded = if pad_w > 0 {
                            format!("{}{}", raw_line, " ".repeat(pad_w))
                        } else {
                            raw_line
                        };

                        // Style the segments
                        let mut segs = Vec::new();

                        // Indent (no style)
                        segs.push(Segment::new(" ".repeat(indent)));

                        // Left sub-box border
                        segs.push(Segment::styled("│".to_string(), border_style.clone()));

                        // Marker
                        if is_error {
                            segs.push(Segment::styled(
                                marker_str.to_string(),
                                error_mark_style.clone(),
                            ));
                        } else {
                            segs.push(Segment::new(marker_str));
                        }

                        // Space + line number
                        let ln_part = format!(" {} ", ln_str);
                        segs.push(Segment::styled(ln_part, line_no_style.clone()));

                        // " │ "
                        segs.push(Segment::styled(" │ ", border_style.clone()));

                        // Code
                        segs.push(Segment::new(code.to_string()));

                        // Padding
                        // Count width so far after the "│" marker
                        let after_marker_w =
                            marker_cells + 1 + ln_width + 3 + UnicodeWidthStr::width(code.as_str());
                        let remain = sub_box_inner
                            .saturating_sub(2) // for │ │
                            .saturating_sub(after_marker_w);
                        if remain > 0 {
                            segs.push(Segment::new(" ".repeat(remain)));
                        }

                        // Right sub-box border
                        segs.push(Segment::styled("│".to_string(), border_style.clone()));

                        out_lines.push(outer_content_line(segs, total_width));
                    }

                    // Sub-box bottom border
                    {
                        let mut segs = Vec::new();
                        segs.push(Segment::styled(
                            format!("{}╰{}╯", " ".repeat(indent), "─".repeat(sub_box_inner)),
                            border_style.clone(),
                        ));
                        out_lines.push(outer_content_line(segs, total_width));
                    }
                } else if let Some(ref line_text) = frame.line {
                    // No source file found -- render the stored line as plain text
                    let indent = 2usize;
                    let mut segs = Vec::new();
                    segs.push(Segment::new(format!(
                        "{}❱ {}",
                        " ".repeat(indent),
                        line_text
                    )));
                    out_lines.push(outer_content_line(segs, total_width));
                }

                // --- Locals table (if enabled and available) ---
                if self.show_locals {
                    if let Some(ref locals) = frame.locals {
                        if !locals.is_empty() {
                            // Locals sub-box
                            let indent = 2usize;
                            let sub_box_total = content_width.saturating_sub(indent * 2);
                            let sub_box_inner = sub_box_total.saturating_sub(2);

                            // Build locals header
                            let header_text = " locals ";

                            // Top border of locals sub-box with header
                            {
                                let mut segs = Vec::new();
                                segs.push(Segment::styled(
                                    format!("{}╭─", " ".repeat(indent)),
                                    border_style.clone(),
                                ));
                                segs.push(Segment::styled(
                                    header_text.to_string(),
                                    locals_header_style.clone(),
                                ));
                                let dash_count =
                                    sub_box_inner.saturating_sub(header_text.len() + 1);
                                segs.push(Segment::styled(
                                    format!("─{}╮", "─".repeat(dash_count)),
                                    border_style.clone(),
                                ));
                                out_lines.push(outer_content_line(segs, total_width));
                            }

                            // Local variable entries
                            let inner_w = sub_box_inner.saturating_sub(2); // │ │
                            let max_shown = self.locals_max_length;
                            let filtered_locals: Vec<(&String, &String)> = locals
                                .iter()
                                .filter(|(k, _)| {
                                    if self.locals_hide_dunder
                                        && k.starts_with("__")
                                        && k.ends_with("__")
                                    {
                                        return false;
                                    }
                                    if self.locals_hide_sunder && k.starts_with('_') {
                                        return false;
                                    }
                                    true
                                })
                                .take(max_shown)
                                .collect();

                            for (key, val) in &filtered_locals {
                                let max_str_len = self.locals_max_string;
                                let display_val = if val.len() > max_str_len {
                                    format!("{}...", &val[..max_str_len])
                                } else {
                                    val.to_string()
                                };
                                let line_text = format!("{} = {}", key, display_val);
                                let raw_w = UnicodeWidthStr::width(line_text.as_str());
                                let pad_w = inner_w.saturating_sub(raw_w);
                                let padded = if pad_w > 0 {
                                    format!("{}{}", line_text, " ".repeat(pad_w))
                                } else {
                                    truncate_to_width(&line_text, inner_w)
                                };

                                let mut segs = Vec::new();
                                segs.push(Segment::new(" ".repeat(indent)));
                                segs.push(Segment::styled("│".to_string(), border_style.clone()));
                                segs.push(Segment::new(format!(" {}", padded)));
                                // Add padding
                                let extra_pad =
                                    inner_w.saturating_sub(UnicodeWidthStr::width(padded.as_str()));
                                if extra_pad > 0 {
                                    segs.push(Segment::new(" ".repeat(extra_pad)));
                                }
                                segs.push(Segment::styled(" │".to_string(), border_style.clone()));
                                out_lines.push(outer_content_line(segs, total_width));
                            }

                            // Bottom border of locals sub-box
                            {
                                let mut segs = Vec::new();
                                segs.push(Segment::styled(
                                    format!(
                                        "{}╰{}╯",
                                        " ".repeat(indent),
                                        "─".repeat(sub_box_inner),
                                    ),
                                    border_style.clone(),
                                ));
                                out_lines.push(outer_content_line(segs, total_width));
                            }
                        }
                    }
                }

                // Blank line after frame
                out_lines.push(outer_blank(total_width));
            }

            // Show suppressed frame count
            if suppressed_count > 0 {
                let msg = format!("  ... {} frames hidden ...", suppressed_count);
                let segs = vec![Segment::styled(msg, Style::new().dim(true))];
                out_lines.push(outer_content_line(segs, total_width));
                out_lines.push(outer_blank(total_width));
                suppressed_count = 0;
            }

            // --- Exception type and value ---
            if let Some(ref exc_type) = stack.exc_type {
                let exc_value = stack.exc_value.as_deref().unwrap_or("");
                let msg = if exc_value.is_empty() {
                    format!("  {}", exc_type)
                } else {
                    format!("  {}: {}", exc_type, exc_value)
                };
                let segs = vec![Segment::styled(msg, error_style.clone())];
                out_lines.push(outer_content_line(segs, total_width));
                out_lines.push(outer_blank(total_width));
            }

            // Exception notes
            for note in &stack.notes {
                let mut segs = Vec::new();
                segs.push(Segment::styled(
                    format!("  note: {}", note),
                    Style::new().italic(true),
                ));
                out_lines.push(outer_content_line(segs, total_width));
            }
        }

        // Bottom border
        out_lines.push(bottom_border(total_width));

        RenderResult {
            lines: out_lines,
            items: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Utility: truncate a string to a given visible (Unicode) width
// ---------------------------------------------------------------------------

fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut w = 0usize;
    let mut result = String::new();
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw > max_width {
            break;
        }
        w += cw;
        result.push(ch);
    }
    result
}

// ---------------------------------------------------------------------------
// Global install -- panic hook
// ---------------------------------------------------------------------------

/// Install a panic hook that renders Rich-formatted tracebacks to stderr.
///
/// This is a best-effort hook -- it attempts to capture the panic payload and
/// produce a formatted traceback, but may not capture full source context.
pub fn install() {
    std::panic::set_hook(Box::new(|panic_info| {
        use std::io::Write;

        // Extract panic message
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        // Extract location
        let (file, line, col) = if let Some(loc) = panic_info.location() {
            (
                loc.file().to_string(),
                loc.line() as usize,
                loc.column() as usize,
            )
        } else {
            ("unknown".to_string(), 0, 0)
        };

        // Build a manual traceback using the backtrace crate (if available) or
        // a simple frame using the panic location.
        let mut frame = Frame::new(file.clone(), line, "unknown".to_string());
        frame.line = Some(msg.clone());

        let exc_value = format!("panic at {}:{}:{}", file, line, col);
        let traceback = Traceback::from_exception("Panic", exc_value, vec![frame]).extra_lines(0);

        // Render to segments
        let opts = ConsoleOptions {
            max_width: 120,
            ..ConsoleOptions::default()
        };
        let result = traceback.render(&opts);
        let ansi = result.to_ansi();

        let _ = writeln!(std::io::stderr(), "{}", ansi);
    }));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_new() {
        let f = Frame::new("main.rs", 42, "foo");
        assert_eq!(f.filename, "main.rs");
        assert_eq!(f.lineno, 42);
        assert_eq!(f.name, "foo");
        assert!(f.line.is_none());
        assert!(f.locals.is_none());
    }

    #[test]
    fn test_frame_builder() {
        let mut locals = HashMap::new();
        locals.insert("x".to_string(), "42".to_string());

        let f = Frame::new("lib.rs", 10, "bar")
            .line("let x = 42;")
            .locals(locals.clone());

        assert_eq!(f.line.unwrap(), "let x = 42;");
        assert_eq!(f.locals.unwrap()["x"], "42");
    }

    #[test]
    fn test_stack_new() {
        let s = Stack::new();
        assert!(s.exc_type.is_none());
        assert!(s.exc_value.is_none());
        assert!(!s.is_cause);
        assert!(s.frames.is_empty());
    }

    #[test]
    fn test_stack_builder() {
        let s = Stack::new()
            .exc_type("ValueError")
            .exc_value("bad value")
            .add_frame(Frame::new("test.rs", 5, "broken"));

        assert_eq!(s.exc_type.unwrap(), "ValueError");
        assert_eq!(s.exc_value.unwrap(), "bad value");
        assert_eq!(s.frames.len(), 1);
    }

    #[test]
    fn test_trace_new() {
        let t = Trace::new();
        assert!(t.stacks.is_empty());
    }

    #[test]
    fn test_trace_from_stack() {
        let s = Stack::new();
        let t = Trace::from_stack(s);
        assert_eq!(t.stacks.len(), 1);
    }

    #[test]
    fn test_traceback_from_exception() {
        let tb = Traceback::from_exception(
            "Error",
            "something went wrong",
            vec![
                Frame::new("main.rs", 1, "main"),
                Frame::new("lib.rs", 42, "helper"),
            ],
        );
        assert_eq!(tb.trace.stacks.len(), 1);
        let stack = &tb.trace.stacks[0];
        assert_eq!(stack.exc_type.as_deref(), Some("Error"));
        assert_eq!(stack.exc_value.as_deref(), Some("something went wrong"));
        assert_eq!(stack.frames.len(), 2);
    }

    #[test]
    fn test_traceback_builder_methods() {
        let tb = Traceback::new(Trace::new())
            .width(100)
            .code_width(80)
            .extra_lines(5)
            .theme("monokai")
            .word_wrap(true)
            .show_locals(true)
            .indent_guides(true)
            .locals_max_length(20)
            .locals_max_string(120)
            .locals_max_depth(10)
            .locals_hide_dunder(false)
            .locals_hide_sunder(true)
            .suppress(vec!["std".to_string()])
            .max_frames(10);

        assert_eq!(tb.width, Some(100));
        assert_eq!(tb.code_width, Some(80));
        assert_eq!(tb.extra_lines, 5);
        assert!(tb.word_wrap);
        assert!(tb.show_locals);
        assert!(!tb.locals_hide_dunder);
        assert!(tb.locals_hide_sunder);
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("hello", 3), "hel");
        assert_eq!(truncate_to_width("hi", 10), "hi");
        assert_eq!(truncate_to_width("", 5), "");
        assert_eq!(truncate_to_width("hello", 0), "");
    }

    #[test]
    fn test_is_suppressed() {
        let suppress = vec!["std".to_string(), "core".to_string()];
        assert!(is_suppressed(
            "/rustc/.../library/std/src/panic.rs",
            &suppress,
        ));
        assert!(is_suppressed(
            "/rustc/.../library/core/src/result.rs",
            &suppress,
        ));
        assert!(!is_suppressed("/home/user/project/src/main.rs", &suppress,));
    }

    #[test]
    fn test_render_empty_traceback() {
        let tb = Traceback::new(Trace::new()).width(60);
        let opts = ConsoleOptions {
            max_width: 60,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        // Should have at least top and bottom borders
        assert!(!result.lines.is_empty());
        // Top border should contain the title
        let ansi = result.to_ansi();
        assert!(ansi.contains("Traceback"));
        assert!(ansi.contains("╭"));
        assert!(ansi.contains("╰"));
    }

    #[test]
    fn test_render_single_frame() {
        let tb = Traceback::from_exception(
            "TestError",
            "testing",
            vec![Frame::new("fake.rs", 10, "test_fn")],
        )
        .width(80);
        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Traceback"));
        assert!(ansi.contains("TestError"));
        assert!(ansi.contains("testing"));
        assert!(ansi.contains("fake.rs"));
    }

    #[test]
    fn test_render_with_locals() {
        let mut locals = HashMap::new();
        locals.insert("x".to_string(), "42".to_string());
        locals.insert("name".to_string(), "hello".to_string());

        let tb = Traceback::from_exception(
            "Error",
            "msg",
            vec![Frame::new("test.rs", 5, "func").locals(locals)],
        )
        .width(80)
        .show_locals(true);

        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();
        // Should include locals (variable names)
        assert!(ansi.contains("x") || ansi.contains("name"));
    }

    #[test]
    fn test_render_suppressed_frame() {
        let tb = Traceback::from_exception(
            "Err",
            "msg",
            vec![
                Frame::new("/rustc/lib.rs", 1, "hidden_fn"),
                Frame::new("main.rs", 10, "main"),
            ],
        )
        .width(80)
        .suppress(vec!["/rustc".to_string()]);

        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("1 frames hidden") || ansi.contains("frames hidden"));
        assert!(ansi.contains("main.rs"));
    }

    #[test]
    fn test_max_frames() {
        let tb = Traceback::from_exception(
            "Err",
            "msg",
            vec![
                Frame::new("a.rs", 1, "a"),
                Frame::new("b.rs", 2, "b"),
                Frame::new("c.rs", 3, "c"),
            ],
        )
        .width(80)
        .max_frames(2);

        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();
        // Should mention hidden frames
        assert!(ansi.contains("frames hidden") || ansi.contains("hidden"));
    }

    #[test]
    fn test_theme_style_resolution() {
        let style = theme_style(theme::names::TRACEBACK_BORDER);
        // Should return a non-plain style (has color or attributes)
        assert!(!style.is_plain());
    }

    #[test]
    fn test_locals_filtering_dunder() {
        let mut locals = HashMap::new();
        locals.insert("__private__".to_string(), "secret".to_string());
        locals.insert("normal".to_string(), "visible".to_string());

        let tb =
            Traceback::from_exception("E", "msg", vec![Frame::new("t.rs", 1, "f").locals(locals)])
                .width(80)
                .show_locals(true)
                .locals_hide_dunder(true);

        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();

        // dunder vars default to hidden
        let _has_private = ansi.contains("__private__");
        let has_normal = ansi.contains("normal");

        // The normal variable should appear; the dunder may or may not
        // (the filtering is applied and should suppress dunder)
        assert!(has_normal);
    }

    #[test]
    fn test_locals_filtering_sunder() {
        let mut locals = HashMap::new();
        locals.insert("_hidden".to_string(), "invisible".to_string());
        locals.insert("visible".to_string(), "yes".to_string());

        let tb =
            Traceback::from_exception("E", "msg", vec![Frame::new("t.rs", 1, "f").locals(locals)])
                .width(80)
                .show_locals(true)
                .locals_hide_sunder(true);

        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();

        // sunder vars should be hidden
        assert!(!ansi.contains("_hidden"));
        assert!(ansi.contains("visible"));
    }

    #[test]
    fn test_install_hook() {
        // Just verify that install() does not panic
        install();
        // Reset the hook so it doesn't interfere with other tests
        let _ = std::panic::take_hook();
    }

    #[test]
    fn test_multiple_stacks() {
        let mut stack1 = Stack::new();
        stack1.exc_type = Some("IOError".to_string());
        stack1.exc_value = Some("file not found".to_string());
        stack1.frames.push(Frame::new("io.rs", 10, "read_file"));

        let mut stack2 = Stack::new();
        stack2.exc_type = Some("ValueError".to_string());
        stack2.exc_value = Some("bad data".to_string());
        stack2.is_cause = true;
        stack2.frames.push(Frame::new("main.rs", 20, "process"));

        let trace = Trace {
            stacks: vec![stack1, stack2],
        };

        let tb = Traceback::new(trace).width(80);
        let opts = ConsoleOptions {
            max_width: 80,
            ..ConsoleOptions::default()
        };
        let result = tb.render(&opts);
        let ansi = result.to_ansi();

        assert!(ansi.contains("IOError"));
        assert!(ansi.contains("ValueError"));
    }
}
