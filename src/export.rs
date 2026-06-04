//! HTML and SVG export — equivalent to Rich's `_export_format.py` and
//! Console export methods.
//!
//! Converts rendered console output into HTML and SVG documents, preserving
//! colors, styles, and layout. Uses `TerminalTheme` to map ANSI colors to
//! CSS-compatible RGB values.

use crate::color::Color;
use crate::segment::Segment;

// ---------------------------------------------------------------------------
// Terminal theme presets (matching Python Rich defaults)
// ---------------------------------------------------------------------------

/// A terminal color theme used for HTML/SVG export.
#[derive(Debug, Clone)]
pub struct ExportTheme {
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    /// ANSI palette: 16 standard colors
    pub ansi_colors: [(u8, u8, u8); 16],
}

impl Default for ExportTheme {
    fn default() -> Self {
        ExportTheme {
            background: (0, 0, 0),
            foreground: (255, 255, 255),
            ansi_colors: [
                (0, 0, 0),       // 0: black
                (128, 0, 0),     // 1: red
                (0, 128, 0),     // 2: green
                (128, 128, 0),   // 3: yellow
                (0, 0, 128),     // 4: blue
                (128, 0, 128),   // 5: magenta
                (0, 128, 128),   // 6: cyan
                (192, 192, 192), // 7: white
                (128, 128, 128), // 8: bright black
                (255, 0, 0),     // 9: bright red
                (0, 255, 0),     // 10: bright green
                (255, 255, 0),   // 11: bright yellow
                (0, 0, 255),     // 12: bright blue
                (255, 0, 255),   // 13: bright magenta
                (0, 255, 255),   // 14: bright cyan
                (255, 255, 255), // 15: bright white
            ],
        }
    }
}

/// Monokai-inspired dark theme for HTML/SVG export.
pub const EXPORT_THEME_MONOKAI: ExportTheme = ExportTheme {
    background: (39, 40, 34),
    foreground: (248, 248, 242),
    ansi_colors: [
        (39, 40, 34),    // 0: black (bg)
        (249, 38, 114),  // 1: red
        (166, 226, 46),  // 2: green
        (230, 219, 116), // 3: yellow
        (102, 217, 239), // 4: blue
        (174, 129, 255), // 5: magenta
        (161, 239, 228), // 6: cyan
        (248, 248, 242), // 7: white
        (117, 113, 94),  // 8: bright black
        (249, 38, 114),  // 9: bright red
        (166, 226, 46),  // 10: bright green
        (230, 219, 116), // 11: bright yellow
        (102, 217, 239), // 12: bright blue
        (174, 129, 255), // 13: bright magenta
        (161, 239, 228), // 14: bright cyan
        (248, 248, 242), // 15: bright white
    ],
};

/// Dimmed Monokai variant -- lower contrast, suitable for comfortable reading.
pub const EXPORT_THEME_DIMMED_MONOKAI: ExportTheme = ExportTheme {
    background: (35, 35, 35),
    foreground: (185, 188, 186),
    ansi_colors: [
        (35, 35, 35),    // 0
        (190, 63, 72),   // 1
        (135, 154, 59),  // 2
        (197, 166, 56),  // 3
        (79, 118, 161),  // 4
        (133, 92, 141),  // 5
        (87, 143, 164),  // 6
        (185, 188, 186), // 7
        (83, 83, 83),    // 8
        (240, 80, 80),   // 9
        (148, 166, 73),  // 10
        (215, 180, 66),  // 11
        (108, 147, 177), // 12
        (152, 117, 171), // 13
        (101, 164, 179), // 14
        (230, 235, 235), // 15
    ],
};

/// Night Owl-inspired dark theme with deep blue background.
pub const EXPORT_THEME_NIGHT_OWLISH: ExportTheme = ExportTheme {
    background: (1, 22, 39),
    foreground: (214, 222, 235),
    ansi_colors: [
        (1, 22, 39),     // 0
        (255, 88, 116),  // 1
        (173, 219, 103), // 2
        (255, 203, 107), // 3
        (130, 170, 255), // 4
        (199, 146, 234), // 5
        (137, 221, 255), // 6
        (214, 222, 235), // 7
        (84, 94, 109),   // 8
        (255, 88, 116),  // 9
        (173, 219, 103), // 10
        (255, 203, 107), // 11
        (130, 170, 255), // 12
        (199, 146, 234), // 13
        (137, 221, 255), // 14
        (255, 255, 255), // 15
    ],
};

/// Light theme with white background, suitable for SVG export snippets.
pub const EXPORT_THEME_SVG: ExportTheme = ExportTheme {
    background: (255, 255, 255),
    foreground: (0, 0, 0),
    ansi_colors: [
        (0, 0, 0),       // 0: black
        (204, 0, 0),     // 1: red
        (0, 170, 0),     // 2: green
        (204, 102, 0),   // 3: yellow
        (0, 0, 204),     // 4: blue
        (170, 0, 170),   // 5: magenta
        (0, 170, 170),   // 6: cyan
        (170, 170, 170), // 7: white
        (102, 102, 102), // 8: bright black
        (255, 0, 0),     // 9: bright red
        (0, 255, 0),     // 10: bright green
        (255, 255, 0),   // 11: bright yellow
        (0, 0, 255),     // 12: bright blue
        (255, 0, 255),   // 13: bright magenta
        (0, 255, 255),   // 14: bright cyan
        (255, 255, 255), // 15: bright white
    ],
};

// ---------------------------------------------------------------------------
// HTML export
// ---------------------------------------------------------------------------

/// The HTML document template used by `export_html`.
pub const CONSOLE_HTML_FORMAT: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>rusty-rich</title>
<style>
    body {{
        margin: 0;
        padding: 0;
    }}
    pre.rich-html {{
        font-family: {font_family};
        font-size: {font_size}px;
        line-height: {line_height};
        color: {foreground};
        background-color: {background};
        margin: 0;
        padding: 16px 24px;
        white-space: pre-wrap;
        word-wrap: break-word;
        overflow-x: auto;
    }}
</style>
</head>
<body>
<pre class="rich-html">
{code}
</pre>
</body>
</html>"#;

/// Options for HTML export.
#[derive(Debug, Clone)]
pub struct ExportHtmlOptions {
    /// Font family for the output.
    pub font_family: String,
    /// Font size in pixels.
    pub font_size: u32,
    /// Line height multiplier.
    pub line_height: f64,
    /// Terminal color theme.
    pub theme: ExportTheme,
    /// Code block to insert.
    pub code: String,
}

impl Default for ExportHtmlOptions {
    fn default() -> Self {
        Self {
            font_family: "'Fira Code', 'Cascadia Code', 'JetBrains Mono', 'Source Code Pro', Menlo, Consolas, monospace".into(),
            font_size: 14,
            line_height: 1.45,
            theme: ExportTheme::default(),
            code: String::new(),
        }
    }
}

/// Generate a full HTML document from rendered terminal output.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::export::{export_html, ExportHtmlOptions};
///
/// let html = export_html(&ExportHtmlOptions {
///     code: "[bold red]Hello[/bold red]".into(),
///     ..Default::default()
/// });
/// std::fs::write("output.html", html).unwrap();
/// ```
pub fn export_html(options: &ExportHtmlOptions) -> String {
    let fg = options.theme.foreground;
    let bg = options.theme.background;

    // Safe ordered replacement: {code} is replaced FIRST to prevent injection
    // via font_family/font_size containing literal placeholder strings (VULN-007)
    CONSOLE_HTML_FORMAT
        .replace("{code}", &escape_html(&options.code))
        .replace("{font_family}", &escape_html(&options.font_family))
        .replace("{font_size}", &escape_html(&options.font_size.to_string()))
        .replace(
            "{line_height}",
            &escape_html(&options.line_height.to_string()),
        )
        .replace(
            "{foreground}",
            &escape_html(&format!("rgb({},{},{})", fg.0, fg.1, fg.2)),
        )
        .replace(
            "{background}",
            &escape_html(&format!("rgb({},{},{})", bg.0, bg.1, bg.2)),
        )
}

/// Save rendered output as an HTML file.
///
/// Convenience wrapper around `export_html` that writes to disk.
pub fn save_html(
    path: impl AsRef<std::path::Path>,
    options: &ExportHtmlOptions,
) -> std::io::Result<()> {
    std::fs::write(path.as_ref(), export_html(options))
}

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------

/// The SVG document template used by `export_svg`.
pub const CONSOLE_SVG_FORMAT: &str = r#"<svg class="rich-svg" xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<style>
    text {{ font-family: {font_family}; font-size: {font_size}px; }}
</style>
<rect width="100%" height="100%" fill="{background}"/>
<text x="0" y="{baseline}" xml:space="preserve">
{code}
</text>
</svg>"#;

/// Options for SVG export.
#[derive(Debug, Clone)]
pub struct ExportSvgOptions {
    /// Font family for the output.
    pub font_family: String,
    /// Font size in pixels.
    pub font_size: u32,
    /// Terminal color theme.
    pub theme: ExportTheme,
    /// Code block to insert.
    pub code: String,
    /// SVG canvas width.
    pub width: u32,
    /// SVG canvas height.
    pub height: u32,
}

impl Default for ExportSvgOptions {
    fn default() -> Self {
        Self {
            font_family: "'Fira Code', 'Cascadia Code', 'JetBrains Mono', monospace".into(),
            font_size: 14,
            theme: EXPORT_THEME_SVG,
            code: String::new(),
            width: 800,
            height: 600,
        }
    }
}

/// Generate a full SVG document from rendered terminal output.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::export::{export_svg, ExportSvgOptions};
///
/// let svg = export_svg(&ExportSvgOptions {
///     code: "[bold blue]Hello SVG[/bold blue]".into(),
///     ..Default::default()
/// });
/// std::fs::write("output.svg", svg).unwrap();
/// ```
pub fn export_svg(options: &ExportSvgOptions) -> String {
    let fg = options.theme.foreground;
    let bg = options.theme.background;
    let baseline = options.font_size as f64 * 1.2; // approximate first-line baseline

    // Safe ordered replacement: {code} goes FIRST, all values escaped (VULN-007)
    CONSOLE_SVG_FORMAT
        .replace("{code}", &escape_xml(&options.code))
        .replace("{font_family}", &escape_xml(&options.font_family))
        .replace("{font_size}", &escape_xml(&options.font_size.to_string()))
        .replace("{width}", &escape_xml(&options.width.to_string()))
        .replace("{height}", &escape_xml(&options.height.to_string()))
        .replace(
            "{background}",
            &escape_xml(&format!("rgb({},{},{})", bg.0, bg.1, bg.2)),
        )
        .replace("{baseline}", &escape_xml(&format!("{:.0}", baseline)))
        .replace("{foreground}", &format!("rgb({},{},{})", fg.0, fg.1, fg.2))
}

/// Save rendered output as an SVG file.
pub fn save_svg(
    path: impl AsRef<std::path::Path>,
    options: &ExportSvgOptions,
) -> std::io::Result<()> {
    std::fs::write(path.as_ref(), export_svg(options))
}

// ---------------------------------------------------------------------------
// Text export
// ---------------------------------------------------------------------------

/// Options for plain-text export (strips ANSI escape codes).
#[derive(Debug, Clone)]
pub struct ExportTextOptions {
    /// The text to export (may contain ANSI escapes).
    pub text: String,
    /// If true, strip ANSI escape sequences. If false, keep them.
    pub strip_ansi: bool,
}

impl Default for ExportTextOptions {
    fn default() -> Self {
        Self {
            text: String::new(),
            strip_ansi: true,
        }
    }
}

/// Export text, optionally stripping ANSI escape sequences. Returns plain text.
pub fn export_text(options: &ExportTextOptions) -> String {
    if options.strip_ansi {
        strip_ansi_escapes(&options.text)
    } else {
        options.text.clone()
    }
}

/// Save text output to a file, optionally stripping ANSI escape sequences.
pub fn save_text(
    path: impl AsRef<std::path::Path>,
    options: &ExportTextOptions,
) -> std::io::Result<()> {
    std::fs::write(path.as_ref(), export_text(options))
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Escape special HTML characters in text.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Escape special XML characters (`&`, `<`, `>`, `"`, `'`) in text.
pub fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Strip ANSI escape sequences from text, returning plain text.
///
/// Handles all ECMA-48 / ISO 6429 escape sequence types:
/// - CSI: `ESC [` parameter-bytes intermediate-bytes final-byte
/// - OSC: `ESC ]` ... `BEL` or `ESC \\`
/// - DCS: `ESC P` ... `ESC \\`
/// - APC: `ESC _` ... `ESC \\`
/// - PM:  `ESC ^` ... `ESC \\`
/// - SOS: `ESC X` ... `ESC \\`
pub fn strip_ansi_escapes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some(&'[') => {
                    chars.next(); // consume '['
                                  // Consume parameter bytes (0x30-0x3F: digits, ;, ?, !, >)
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == ';' || c == '?' || c == '!' || c == '>' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Consume intermediate bytes (0x20-0x2F)
                    while let Some(&c) = chars.peek() {
                        if (0x20..=0x2F).contains(&(c as u32)) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Consume final byte (0x40-0x7E)
                    chars.next();
                }
                // OSC, DCS, APC, PM, SOS — terminated by BEL or ST
                Some(&']') | Some(&'P') | Some(&'_') | Some(&'^') | Some(&'X') => {
                    chars.next(); // consume the type byte
                    while let Some(&c) = chars.peek() {
                        if c == '\x07' {
                            chars.next();
                            break;
                        } else if c == '\x1b' {
                            chars.next();
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                        } else {
                            chars.next();
                        }
                    }
                }
                // Unknown escape type — just consume ESC
                _ => {}
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert Rich styled segments to HTML with inline CSS spans.
///
/// Each segment's foreground color, background color, bold, italic, etc.
/// are mapped to `<span style="...">` elements.
pub fn segments_to_html(segments: &[Segment], theme: &ExportTheme) -> String {
    let mut html = String::new();

    for seg in segments {
        let mut styles: Vec<String> = Vec::new();

        if let Some(ref style) = seg.style {
            // Foreground color
            if let Some(color) = &style.color {
                let rgb = resolve_color(color, theme);
                styles.push(format!("color:rgb({},{},{})", rgb.0, rgb.1, rgb.2));
            } else {
                // Use foreground default
                let fg = theme.foreground;
                styles.push(format!("color:rgb({},{},{})", fg.0, fg.1, fg.2));
            }

            // Background color
            if let Some(bgcolor) = &style.bgcolor {
                let rgb = resolve_color(bgcolor, theme);
                styles.push(format!(
                    "background-color:rgb({},{},{})",
                    rgb.0, rgb.1, rgb.2
                ));
            }

            // Text attributes
            let attrs = &style.attributes;
            if attrs.get(crate::style::Attributes::BOLD) {
                styles.push("font-weight:bold".into());
            }
            if attrs.get(crate::style::Attributes::ITALIC) {
                styles.push("font-style:italic".into());
            }
            if attrs.get(crate::style::Attributes::UNDERLINE)
                || attrs.get(crate::style::Attributes::UNDERLINE2)
            {
                styles.push("text-decoration:underline".into());
            }
            if attrs.get(crate::style::Attributes::STRIKE) {
                styles.push("text-decoration:line-through".into());
            }
            if attrs.get(crate::style::Attributes::DIM) {
                styles.push("opacity:0.7".into());
            }
            if attrs.get(crate::style::Attributes::CONCEAL) {
                styles.push("visibility:hidden".into());
            }

            // Hyperlink
            if let Some(ref link) = style.link {
                let escaped_link = escape_html(link);
                let style_attr = if styles.is_empty() {
                    String::new()
                } else {
                    format!(" style=\"{}\"", styles.join("; "))
                };
                html.push_str(&format!(
                    "<a href=\"{}\"{}>{}</a>",
                    escaped_link,
                    style_attr,
                    escape_html(&seg.text)
                ));
                continue; // skip normal span handling for links
            }
        } else {
            // No style — use theme defaults
            let fg = theme.foreground;
            styles.push(format!("color:rgb({},{},{})", fg.0, fg.1, fg.2));
        }

        // Emit styled span
        if styles.is_empty() {
            html.push_str(&escape_html(&seg.text));
        } else {
            let style_attr = styles.join("; ");
            html.push_str(&format!(
                "<span style=\"{}\">{}</span>",
                style_attr,
                escape_html(&seg.text)
            ));
        }
    }

    html
}

/// Convert Rich styled segments to SVG `<tspan>` elements with inline fill
/// colors. Each segment's foreground color maps to a `fill` attribute, and
/// text attributes (bold, italic, underline, strike, dim, conceal) are
/// mapped to CSS properties.
pub fn segments_to_svg(segments: &[Segment], theme: &ExportTheme) -> String {
    let mut svg = String::new();

    for seg in segments {
        let mut styles: Vec<String> = Vec::new();

        if let Some(ref style) = seg.style {
            // Foreground color
            if let Some(color) = &style.color {
                let rgb = resolve_color(color, theme);
                styles.push(format!("fill:rgb({},{},{})", rgb.0, rgb.1, rgb.2));
            } else {
                let fg = theme.foreground;
                styles.push(format!("fill:rgb({},{},{})", fg.0, fg.1, fg.2));
            }

            // Text attributes
            let attrs = &style.attributes;
            if attrs.get(crate::style::Attributes::BOLD) {
                styles.push("font-weight:bold".into());
            }
            if attrs.get(crate::style::Attributes::ITALIC) {
                styles.push("font-style:italic".into());
            }
            if attrs.get(crate::style::Attributes::UNDERLINE)
                || attrs.get(crate::style::Attributes::UNDERLINE2)
            {
                styles.push("text-decoration:underline".into());
            }
            if attrs.get(crate::style::Attributes::STRIKE) {
                styles.push("text-decoration:line-through".into());
            }
            if attrs.get(crate::style::Attributes::DIM) {
                styles.push("opacity:0.7".into());
            }
            if attrs.get(crate::style::Attributes::CONCEAL) {
                styles.push("visibility:hidden".into());
            }
        } else {
            let fg = theme.foreground;
            styles.push(format!("fill:rgb({},{},{})", fg.0, fg.1, fg.2));
        }

        if styles.is_empty() {
            svg.push_str(&escape_xml(&seg.text));
        } else {
            let style_attr = styles.join("; ");
            svg.push_str(&format!(
                "<tspan style=\"{}\">{}</tspan>",
                style_attr,
                escape_xml(&seg.text)
            ));
        }
    }

    svg
}

/// Resolve a color to an RGB triplet given a terminal theme.
fn resolve_color(color: &Color, theme: &ExportTheme) -> (u8, u8, u8) {
    match color.color_type {
        crate::color::ColorType::Default => theme.foreground,
        crate::color::ColorType::Standard => {
            let idx = color.number.unwrap_or(7) as usize % 16;
            theme.ansi_colors[idx]
        }
        crate::color::ColorType::EightBit => {
            let idx = color.number.unwrap_or(0) as usize % 256;
            rgb_for_8bit(idx)
        }
        crate::color::ColorType::TrueColor => {
            if let Some(ref triplet) = color.triplet {
                (triplet.0, triplet.1, triplet.2)
            } else {
                theme.foreground
            }
        }
    }
}

/// Map an 8-bit (256) color index to an RGB triplet.
fn rgb_for_8bit(index: usize) -> (u8, u8, u8) {
    if index < 16 {
        // Standard ANSI colors
        crate::color::STANDARD_PALETTE
            .get(index)
            .copied()
            .unwrap_or((0, 0, 0))
    } else if index < 232 {
        // 6×6×6 color cube
        let idx = index - 16;
        let r = (idx / 36) as u8 * 51;
        let g = ((idx / 6) % 6) as u8 * 51;
        let b = (idx % 6) as u8 * 51;
        (r, g, b)
    } else {
        // Greyscale ramp (232–255)
        let g = ((index - 232) * 10 + 8) as u8;
        (g, g, g)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::style::Style;

    #[test]
    fn test_escape_html_basic() {
        assert_eq!(escape_html("<hello>"), "&lt;hello&gt;");
        assert_eq!(escape_html("\"a\" & 'b'"), "&quot;a&quot; &amp; 'b'");
    }

    #[test]
    fn test_strip_ansi_escapes() {
        let input = "\x1b[31mred\x1b[0m normal";
        assert_eq!(strip_ansi_escapes(input), "red normal");
    }

    #[test]
    fn test_strip_ansi_complex() {
        let input = "\x1b[1;31mBold Red\x1b[0m \x1b[4munderlined\x1b[0m";
        assert_eq!(strip_ansi_escapes(input), "Bold Red underlined");
    }

    #[test]
    fn test_strip_ansi_no_escapes() {
        assert_eq!(strip_ansi_escapes("plain text"), "plain text");
    }

    #[test]
    fn test_export_html_basic() {
        let opts = ExportHtmlOptions {
            code: "Hello World".into(),
            ..Default::default()
        };
        let html = export_html(&opts);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("rich-html"));
        assert!(html.contains("font-family"));
    }

    #[test]
    fn test_export_html_escapes_markup() {
        let opts = ExportHtmlOptions {
            code: "<script>alert('xss')</script>".into(),
            ..Default::default()
        };
        let html = export_html(&opts);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_export_svg_basic() {
        let opts = ExportSvgOptions {
            code: "SVG text".into(),
            ..Default::default()
        };
        let svg = export_svg(&opts);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("SVG text"));
        assert!(svg.contains("rich-svg"));
    }

    #[test]
    fn test_export_svg_theme() {
        let opts = ExportSvgOptions {
            code: "test".into(),
            theme: EXPORT_THEME_SVG,
            ..Default::default()
        };
        let svg = export_svg(&opts);
        assert!(svg.contains("rgb(255,255,255)")); // background
    }

    #[test]
    fn test_export_text_strip() {
        let opts = ExportTextOptions {
            text: "\x1b[1;32mGreen Bold\x1b[0m".into(),
            strip_ansi: true,
        };
        assert_eq!(export_text(&opts), "Green Bold");
    }

    #[test]
    fn test_export_text_keep() {
        let ansi = "\x1b[31mred\x1b[0m";
        let opts = ExportTextOptions {
            text: ansi.into(),
            strip_ansi: false,
        };
        assert_eq!(export_text(&opts), ansi);
    }

    #[test]
    fn test_rgb_for_8bit_standard() {
        assert_eq!(rgb_for_8bit(0), (0, 0, 0)); // black
        assert_eq!(rgb_for_8bit(1), (128, 0, 0)); // red
        assert_eq!(rgb_for_8bit(15), (255, 255, 255)); // bright white
    }

    #[test]
    fn test_rgb_for_8bit_cube() {
        assert_eq!(rgb_for_8bit(16), (0, 0, 0));
        let idx = 16 + 1 * 36 + 2 * 6 + 3; // R=1, G=2, B=3
        assert_eq!(rgb_for_8bit(idx), (51, 102, 153));
    }

    #[test]
    fn test_rgb_for_8bit_greyscale() {
        assert_eq!(rgb_for_8bit(232), (8, 8, 8));
        assert_eq!(rgb_for_8bit(255), (238, 238, 238));
    }

    #[test]
    fn test_segments_to_html_styled() {
        let seg = Segment::styled(
            "hello",
            Style::new().color(Color::parse("red").unwrap()).bold(true),
        );
        let html = segments_to_html(&[seg], &ExportTheme::default());
        assert!(html.contains("color:rgb(128,0,0)"));
        assert!(html.contains("font-weight:bold"));
        assert!(html.contains("hello"));
    }

    #[test]
    fn test_segments_to_html_plain() {
        let seg = Segment::new("plain");
        let html = segments_to_html(&[seg], &ExportTheme::default());
        assert!(html.contains("plain"));
        assert!(html.contains("color:rgb(255,255,255)"));
    }

    #[test]
    fn test_export_theme_defaults() {
        let theme = ExportTheme::default();
        assert_eq!(theme.background, (0, 0, 0));
        assert_eq!(theme.foreground, (255, 255, 255));
    }

    #[test]
    fn test_segments_to_svg_styled() {
        let seg = Segment::styled(
            "hello",
            Style::new().color(Color::parse("red").unwrap()).bold(true),
        );
        let svg = segments_to_svg(&[seg], &ExportTheme::default());
        assert!(svg.contains("fill:rgb(128,0,0)"));
        assert!(svg.contains("font-weight:bold"));
        assert!(svg.contains("hello"));
        assert!(svg.contains("<tspan"));
    }

    #[test]
    fn test_segments_to_svg_plain() {
        let seg = Segment::new("plain");
        let svg = segments_to_svg(&[seg], &ExportTheme::default());
        assert!(svg.contains("plain"));
        assert!(svg.contains("fill:rgb(255,255,255)"));
    }

    #[test]
    fn test_save_to_disk() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_export.html");
        let opts = ExportHtmlOptions {
            code: "test".into(),
            ..Default::default()
        };
        save_html(&path, &opts).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("test"));
        std::fs::remove_file(&path).unwrap();
    }
}
