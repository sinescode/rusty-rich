//! Box drawing — equivalent to Rich's `box.py`.
//!
//! Defines various box styles (ROUNDED, SQUARE, HEAVY, etc.) using Unicode
//! box-drawing characters, with ASCII-safe fallbacks.

// ---------------------------------------------------------------------------
// Box — defines characters for drawing a bordered box
// ---------------------------------------------------------------------------

/// A set of box-drawing characters defining the look of borders.
///
/// Layout of the 8-line string that defines a box:
///
/// ```text
/// ┌─┬┐ top
/// │ ││ head
/// ├─┼┤ head_row
/// │ ││ mid
/// ├─┼┤ row
/// ├─┼┤ foot_row
/// │ ││ foot
/// └─┴┘ bottom
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxStyle {
    // top row
    pub top_left: char,
    pub top: char,
    pub top_divider: char,
    pub top_right: char,
    // head row (where content is on same line as top border)
    pub head_left: char,
    pub head_horizontal: char,
    pub head_vertical: char,
    pub head_right: char,
    // head_row (separator after header)
    pub head_row_left: char,
    pub head_row_horizontal: char,
    pub head_row_cross: char,
    pub head_row_right: char,
    // mid (between rows when show_lines is off)
    pub mid_left: char,
    pub mid_horizontal: char,
    pub mid_vertical: char,
    pub mid_right: char,
    // row (between rows when show_lines is on)
    pub row_left: char,
    pub row_horizontal: char,
    pub row_cross: char,
    pub row_right: char,
    // foot_row (separator before footer)
    pub foot_row_left: char,
    pub foot_row_horizontal: char,
    pub foot_row_cross: char,
    pub foot_row_right: char,
    // foot
    pub foot_left: char,
    pub foot_horizontal: char,
    pub foot_vertical: char,
    pub foot_right: char,
    // bottom row
    pub bottom_left: char,
    pub bottom: char,
    pub bottom_divider: char,
    pub bottom_right: char,
    /// True if this box uses only ASCII characters.
    pub ascii: bool,
}

impl BoxStyle {
    /// Parse a box style from an 8-line string.
    pub fn from_str(box_str: &str, ascii: bool) -> Self {
        let lines: Vec<&str> = box_str.lines().collect();
        assert_eq!(lines.len(), 8, "Box definition must have exactly 8 lines");

        let line_chars: Vec<Vec<char>> = lines.iter()
            .map(|l| l.chars().collect())
            .collect();

        // Each line should have 4 characters
        for (i, chars) in line_chars.iter().enumerate() {
            assert_eq!(chars.len(), 4, "Line {i} must have exactly 4 characters");
        }

        let l = &line_chars;
        Self {
            top_left: l[0][0], top: l[0][1], top_divider: l[0][2], top_right: l[0][3],
            head_left: l[1][0], head_horizontal: l[1][1], head_vertical: l[1][2], head_right: l[1][3],
            head_row_left: l[2][0], head_row_horizontal: l[2][1], head_row_cross: l[2][2], head_row_right: l[2][3],
            mid_left: l[3][0], mid_horizontal: l[3][1], mid_vertical: l[3][2], mid_right: l[3][3],
            row_left: l[4][0], row_horizontal: l[4][1], row_cross: l[4][2], row_right: l[4][3],
            foot_row_left: l[5][0], foot_row_horizontal: l[5][1], foot_row_cross: l[5][2], foot_row_right: l[5][3],
            foot_left: l[6][0], foot_horizontal: l[6][1], foot_vertical: l[6][2], foot_right: l[6][3],
            bottom_left: l[7][0], bottom: l[7][1], bottom_divider: l[7][2], bottom_right: l[7][3],
            ascii,
        }
    }

    /// Get the plain text representation of the box definition.
    pub fn to_string(&self) -> String {
        format!(
            "{}{}{}{}\n{}{}{}{}\n{}{}{}{}\n{}{}{}{}\n{}{}{}{}\n{}{}{}{}\n{}{}{}{}\n{}{}{}{}",
            self.top_left, self.top, self.top_divider, self.top_right,
            self.head_left, self.head_horizontal, self.head_vertical, self.head_right,
            self.head_row_left, self.head_row_horizontal, self.head_row_cross, self.head_row_right,
            self.mid_left, self.mid_horizontal, self.mid_vertical, self.mid_right,
            self.row_left, self.row_horizontal, self.row_cross, self.row_right,
            self.foot_row_left, self.foot_row_horizontal, self.foot_row_cross, self.foot_row_right,
            self.foot_left, self.foot_horizontal, self.foot_vertical, self.foot_right,
            self.bottom_left, self.bottom, self.bottom_divider, self.bottom_right,
        )
    }
}

impl std::fmt::Display for BoxStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// ---------------------------------------------------------------------------
// Predefined box styles (matching Rich's defaults)
// ---------------------------------------------------------------------------

/// ASCII-only box.
pub const ASCII: &str = "\
+--+
| ||
|-+|
| ||
|-+|
|-+|
| ||
+-++";

/// ASCII with double edges.
pub const ASCII2: &str = "\
+-++
| ||
| ||
| ||
| ||
| ||
| ||
+-++";

/// Square box with double horizontal header separator.
pub const SQUARE_DOUBLE_HEAD: &str = "\
┌─┬┐
│ ║│
├─╪┤
│ ││
├─┼┤
├─┼┤
│ ││
└─┴┘";

/// Minimal box with double horizontal separator (head row only).
pub const MINIMAL_DOUBLE_HEAD: &str = "  ═ \n  ═ \n  ═ \n    \n    \n    \n  ═ \n    ";

/// Simple box with a single horizontal rule under the header.
pub const SIMPLE_HEAD: &str = "    \n    \n    \n    \n    \n    \n ━┿ \n    ";

/// ASCII box style with a double header line.
pub const ASCII_DOUBLE_HEAD: &str = "\
+-++
| ||
+=+|
| ||
|-+|
|-+|
| ||
+-++";

/// Rounded corners.
pub const ROUNDED: &str = "\
╭─┬╮
│ ││
├─┼┤
│ ││
├─┼┤
├─┼┤
│ ││
╰─┴╯";

/// Square corners.
pub const SQUARE: &str = "\
┌─┬┐
│ ││
├─┼┤
│ ││
├─┼┤
├─┼┤
│ ││
└─┴┘";

/// Heavy borders.
pub const HEAVY: &str = "\
┏━┳┓
┃ ┃┃
┣━╋┫
┃ ┃┃
┣━╋┫
┣━╋┫
┃ ┃┃
┗━┻┛";

/// Heavy edge, light inner.
pub const HEAVY_EDGE: &str = "\
┏━┯┓
┃ │┃
┠─┼┨
┃ │┃
┠─┼┨
┠─┼┨
┃ │┃
┗━┷┛";

/// Heavy header.
pub const HEAVY_HEAD: &str = "\
┏━┳┓
┃ ┃┃
┡━╇┩
│ ││
├─┼┤
├─┼┤
│ ││
└─┴┘";

/// Double borders.
pub const DOUBLE: &str = "\
╔═╦╗
║ ║║
╠═╬╣
║ ║║
╠═╬╣
╠═╬╣
║ ║║
╚═╩╝";

/// Double edge (like DOUBLE but inner is single).
pub const DOUBLE_EDGE: &str = "\
╔═╤╗
║ │║
╟─┼╢
║ │║
╟─┼╢
╟─┼╢
║ │║
╚═╧╝";

/// Simple (no borders, just vertical separators).
pub const SIMPLE: &str = "    \n    \n ── \n    \n    \n ── \n    \n    ";

/// Simple with heavy header.
pub const SIMPLE_HEAVY: &str = "    \n    \n ━━ \n    \n    \n ━━ \n    \n    ";

/// Minimal (just horizontal rule under header).
pub const MINIMAL: &str = "\
  ╌
  ╌
  ╌




  ╌ ";

/// Minimal with heavy header.
pub const MINIMAL_HEAVY: &str = "\
  ╍
  ╍
  ╍




  ╍ ";

// ---------------------------------------------------------------------------
// Box style constants (lazily parsed)
// ---------------------------------------------------------------------------

use once_cell::sync::Lazy;

/// Rounded box (default for Panel).
pub static BOX_ROUNDED: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(ROUNDED, false));
pub static BOX_SQUARE: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(SQUARE, false));
pub static BOX_HEAVY: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(HEAVY, false));
pub static BOX_HEAVY_EDGE: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(HEAVY_EDGE, false));
pub static BOX_HEAVY_HEAD: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(HEAVY_HEAD, false));
pub static BOX_DOUBLE: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(DOUBLE, false));
pub static BOX_DOUBLE_EDGE: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(DOUBLE_EDGE, false));
pub static BOX_SIMPLE: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(SIMPLE, false));
pub static BOX_SIMPLE_HEAVY: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(SIMPLE_HEAVY, false));
pub static BOX_MINIMAL: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(MINIMAL, false));
pub static BOX_MINIMAL_HEAVY: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(MINIMAL_HEAVY, false));
pub static BOX_ASCII: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(ASCII, true));
pub static BOX_ASCII2: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(ASCII2, true));
pub static BOX_SQUARE_DOUBLE_HEAD: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(SQUARE_DOUBLE_HEAD, false));
pub static BOX_MINIMAL_DOUBLE_HEAD: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(MINIMAL_DOUBLE_HEAD, false));
pub static BOX_SIMPLE_HEAD: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(SIMPLE_HEAD, false));
pub static BOX_ASCII_DOUBLE_HEAD: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(ASCII_DOUBLE_HEAD, true));

// ---------------------------------------------------------------------------
// MARKDOWN box (no outer border)
// ---------------------------------------------------------------------------

pub const MARKDOWN: &str = "    \n| ||\n|-||\n| ||\n|-||\n|-||\n| ||\n    ";

pub static BOX_MARKDOWN: Lazy<BoxStyle> = Lazy::new(|| BoxStyle::from_str(MARKDOWN, false));

// ---------------------------------------------------------------------------
// Safe box (for Windows legacy terminals)
// ---------------------------------------------------------------------------

/// Return an ASCII-safe version of a box if needed.
pub fn get_safe_box(box_style: &BoxStyle, ascii_only: bool) -> BoxStyle {
    if ascii_only && !box_style.ascii {
        BOX_ASCII.clone()
    } else {
        box_style.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounded_box() {
        let b = &*BOX_ROUNDED;
        assert_eq!(b.top_left, '╭');
        assert_eq!(b.bottom_right, '╯');
    }

    #[test]
    fn test_box_from_str() {
        let b = BoxStyle::from_str(ROUNDED, false);
        assert_eq!(b, *BOX_ROUNDED);
    }

    #[test]
    fn test_new_box_styles_parse() {
        // Verify that the new box styles parse without panicking
        let _ = &*BOX_SQUARE_DOUBLE_HEAD;
        let _ = &*BOX_MINIMAL_DOUBLE_HEAD;
        let _ = &*BOX_SIMPLE_HEAD;
        let _ = &*BOX_ASCII_DOUBLE_HEAD;

        // Spot-check characters
        let sq = &*BOX_SQUARE_DOUBLE_HEAD;
        assert_eq!(sq.top_left, '┌');
        assert_eq!(sq.head_vertical, '║');
        assert_eq!(sq.head_row_horizontal, '─');

        let ac = &*BOX_ASCII_DOUBLE_HEAD;
        assert_eq!(ac.head_row_left, '+');
        assert_eq!(ac.head_row_horizontal, '=');
    }
}
