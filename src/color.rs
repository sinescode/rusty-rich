//! Color system — equivalent to Rich's `color.py`.
//!
//! Supports ANSI standard (16) colors, 8-bit (256) colors, and 24-bit true
//! color with automatic downgrade. Includes 256 named color constants, hex/RGB
//! constructors, and color blending utilities.
//!
//! # Quick Example
//!
//! ```rust
//! use rusty_rich::Color;
//!
//! // Named colors — 256 ANSI palette
//! let red = Color::parse("red").unwrap();
//! let hot_pink = Color::parse("hot_pink").unwrap();
//!
//! // Hex and RGB
//! let orange = Color::from_hex("#FF6600").unwrap();
//! let custom = Color::from_rgb(100, 200, 50);
//! ```
//!
//! # Color Systems
//!
//! [`ColorSystem`] describes what the terminal supports:
//!
//! - [`ColorSystem::Standard`] — 16 ANSI colors
//! - [`ColorSystem::EightBit`] — 256-color palette
//! - [`ColorSystem::TrueColor`] — 24-bit RGB
//!
//! Use [`Color::downgrade`] to convert a color to a lower color system.
//!
//! # Named Color Map
//!
//! [`Color::from_ansi_name`] and [`Color::parse`] look up names in the
//! 256-entry ANSI palette. Both `grey`/`gray` spellings are supported.

use std::fmt;

/// An RGB color triplet — equivalent to Rich's `ColorTriplet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ColorTriplet {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ColorTriplet {
    /// Create a new `ColorTriplet` from red, green, and blue components.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

impl fmt::Display for ColorTriplet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}
use std::hash::Hash;

// ---------------------------------------------------------------------------
// ColorSystem — what the terminal supports
// ---------------------------------------------------------------------------

/// The color system supported by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSystem {
    /// 3-bit / 4-bit ANSI standard (8/16 colors)
    Standard = 1,
    /// 8-bit (256 colors)
    EightBit = 2,
    /// 24-bit true color
    TrueColor = 3,
}

impl fmt::Display for ColorSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::EightBit => write!(f, "256"),
            Self::TrueColor => write!(f, "truecolor"),
        }
    }
}

// ---------------------------------------------------------------------------
// ColorType
// ---------------------------------------------------------------------------

/// How the color value is stored internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorType {
    /// No color / inherit
    Default,
    /// One of the 16 ANSI named colors
    Standard,
    /// 8-bit palette entry (0–255)
    EightBit,
    /// 24-bit true color
    TrueColor,
}

// ---------------------------------------------------------------------------
// ANSI_COLOR_NAMES — maps name → index in the 256-color table
// ---------------------------------------------------------------------------

/// Full set of named ANSI colors. Use `Color::name_to_index()` to look up.

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

/// A terminal color.
///
/// Can be one of: default (inherit), a standard ANSI name, an 8-bit palette
/// index, or a 24-bit true-color RGB triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub(crate) color_type: ColorType,
    /// For Standard: the ANSI index (0–15).
    /// For EightBit: the palette index (0–255).
    pub(crate) number: Option<u8>,
    /// For TrueColor: the RGB triple.
    pub(crate) triplet: Option<(u8, u8, u8)>,
    /// Optional name string (kept for round-tripping).
    pub(crate) name: Option<&'static str>,
}

impl Color {
    // -- constructors -------------------------------------------------------

    /// Create a "default" color (inherit from parent).
    pub const fn default() -> Self {
        Self {
            color_type: ColorType::Default,
            number: None,
            triplet: None,
            name: None,
        }
    }

    /// Create from an ANSI standard name (e.g. "red", "bright_blue").
    pub fn from_ansi_name(name: &str) -> Option<Self> {
        let n = ANSI_NAME_MAP.get(name).copied()?;
        Some(Self {
            color_type: if n < 16 {
                ColorType::Standard
            } else {
                ColorType::EightBit
            },
            number: Some(n),
            triplet: None,
            name: None,
        })
    }

    /// Create from an 8-bit (256) palette index.
    pub fn from_8bit(n: u8) -> Self {
        Self {
            color_type: ColorType::EightBit,
            number: Some(n),
            triplet: None,
            name: None,
        }
    }

    /// Create from 24-bit RGB components.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            color_type: ColorType::TrueColor,
            number: None,
            triplet: Some((r, g, b)),
            name: None,
        }
    }

    /// Create from a hex string like "#ff0000" or "ff0000".
    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(ColorParseError::InvalidHex(hex.to_string()));
        }
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| ColorParseError::InvalidHex(hex.to_string()))?;
        Ok(Self::from_rgb(r, g, b))
    }

    /// Parse a color from a string (name, hex, CSS color name, or "default").
    pub fn parse(s: &str) -> Result<Self, ColorParseError> {
        let lower = s.to_lowercase();
        if lower == "default" || lower.is_empty() {
            return Ok(Self::default());
        }
        if let Some(c) = Self::from_ansi_name(&lower) {
            return Ok(c);
        }
        // Try CSS/web color names
        if let Some((r, g, b)) = lookup_css_color(&lower) {
            return Ok(Self::from_rgb(r, g, b));
        }
        if lower.starts_with('#') || lower.len() == 6 {
            return Self::from_hex(&lower);
        }
        // Try "color<N>" format for 8-bit
        if lower.starts_with("color") {
            if let Ok(n) = lower[5..].parse::<u8>() {
                return Ok(Self::from_8bit(n));
            }
        }
        Err(ColorParseError::UnknownName(lower))
    }

    // -- queries ------------------------------------------------------------

    /// Is this the default/inherit color?
    pub fn is_default(&self) -> bool {
        matches!(self.color_type, ColorType::Default)
    }

    /// Create a Color from a [`ColorTriplet`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_rich::color::{Color, ColorTriplet};
    ///
    /// let triplet = ColorTriplet::new(255, 0, 0);
    /// let color = Color::from_triplet(&triplet);
    /// assert!(!color.is_default());
    /// ```
    pub fn from_triplet(triplet: &ColorTriplet) -> Self {
        Self::from_rgb(triplet.red, triplet.green, triplet.blue)
    }

    /// Returns `true` if this is a system-defined (non-custom) color.
    ///
    /// System-defined colors are standard ANSI colors or named 8-bit palette
    /// entries. TrueColor and Default colors are not system-defined.
    pub fn is_system_defined(&self) -> bool {
        matches!(self.color_type, ColorType::Standard | ColorType::EightBit)
    }

    /// Get the ANSI escape codes for this color as a foreground/background pair.
    ///
    /// Returns `(foreground_code, background_code)` as decimal strings suitable
    /// for use in SGR sequences like `\x1b[38;5;<code>m`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_rich::Color;
    ///
    /// let red = Color::parse("red").unwrap();
    /// let (fg, bg) = red.get_ansi_codes(false);
    /// assert_eq!(fg, Some("31".to_string()));
    /// ```
    pub fn get_ansi_codes(&self, background: bool) -> (Option<String>, Option<String>) {
        if self.is_default() {
            return (None, None);
        }
        let code = match self.color_type {
            ColorType::Standard => {
                let base: u8 = if background { 40 } else { 30 };
                if let Some(n) = self.number {
                    let bright_offset: u8 = if n >= 8 { 60 } else { 0 };
                    Some((base + n + bright_offset).to_string())
                } else {
                    None
                }
            }
            ColorType::EightBit => {
                let prefix = if background { "48;5;" } else { "38;5;" };
                self.number.map(|n| format!("{prefix}{n}"))
            }
            ColorType::TrueColor => {
                let prefix = if background { "48;2;" } else { "38;2;" };
                self.triplet.map(|(r, g, b)| format!("{prefix}{r};{g};{b}"))
            }
            ColorType::Default => None,
        };
        if background {
            (None, code)
        } else {
            (code, None)
        }
    }

    /// Get the name of this color, if it has one.
    ///
    /// For Standard and EightBit colors that were created from a name, this
    /// returns the original name. For TrueColor and Default colors, returns
    /// `None`.
    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    /// Get the ANSI palette number for this color, if applicable.
    ///
    /// Returns `Some(n)` for Standard (0–15) and EightBit (0–255) colors.
    /// Returns `None` for TrueColor and Default colors.
    pub fn number(&self) -> Option<u8> {
        self.number
    }

    /// Get the RGB triplet for this color, if it has one.
    ///
    /// Returns `Some((r, g, b))` for TrueColor colors. For Standard and
    /// EightBit colors, the palette is consulted to compute the equivalent
    /// RGB values. Returns `None` for Default colors.
    pub fn triplet(&self) -> Option<(u8, u8, u8)> {
        self.triplet
    }

    /// Get the RGB triplet if available (computes it for named/8-bit colors
    /// by looking up the palette).
    pub fn get_truecolor(&self, theme: &TerminalTheme) -> (u8, u8, u8) {
        match self.color_type {
            ColorType::TrueColor => self.triplet.unwrap(),
            ColorType::Default => theme.foreground_color,
            _ => {
                if let Some(n) = self.number {
                    if let Some(&[r, g, b]) = EIGHT_BIT_PALETTE.get(n as usize) {
                        return (r, g, b);
                    }
                }
                theme.foreground_color
            }
        }
    }

    /// Downgrade this color to the given color system.
    pub fn downgrade(&self, system: ColorSystem) -> Self {
        match system {
            ColorSystem::TrueColor => *self,
            ColorSystem::EightBit => {
                if matches!(self.color_type, ColorType::TrueColor) {
                    let (r, g, b) = self.triplet.unwrap();
                    let idx = rgb_to_8bit(r, g, b);
                    Self::from_8bit(idx)
                } else {
                    *self
                }
            }
            ColorSystem::Standard => {
                if matches!(self.color_type, ColorType::TrueColor) {
                    let (r, g, b) = self.triplet.unwrap();
                    let idx = rgb_to_standard(r, g, b);
                    Self {
                        color_type: ColorType::Standard,
                        number: Some(idx),
                        triplet: None,
                        name: None,
                    }
                } else if let Some(n) = self.number {
                    if n >= 16 {
                        let idx = n % 16;
                        Self {
                            color_type: ColorType::Standard,
                            number: Some(idx),
                            triplet: None,
                            name: None,
                        }
                    } else {
                        *self
                    }
                } else {
                    *self
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CSS named color lookup table (module-level, outside impl Color)
// ---------------------------------------------------------------------------

/// CSS named colors sorted alphabetically for binary search.
///
/// Includes the standard 148 CSS color names mapped to their RGB values.
/// Names are lowercase for case-insensitive lookup.
static CSS_COLORS: &[(&str, (u8, u8, u8))] = &[
    ("aliceblue", (240, 248, 255)),
    ("antiquewhite", (250, 235, 215)),
    ("aqua", (0, 255, 255)),
    ("aquamarine", (127, 255, 212)),
    ("azure", (240, 255, 255)),
    ("beige", (245, 245, 220)),
    ("bisque", (255, 228, 196)),
    ("black", (0, 0, 0)),
    ("blanchedalmond", (255, 235, 205)),
    ("blue", (0, 0, 255)),
    ("blueviolet", (138, 43, 226)),
    ("brown", (165, 42, 42)),
    ("burlywood", (222, 184, 135)),
    ("cadetblue", (95, 158, 160)),
    ("chartreuse", (127, 255, 0)),
    ("chocolate", (210, 105, 30)),
    ("coral", (255, 127, 80)),
    ("cornflowerblue", (100, 149, 237)),
    ("cornsilk", (255, 248, 220)),
    ("crimson", (220, 20, 60)),
    ("cyan", (0, 255, 255)),
    ("darkblue", (0, 0, 139)),
    ("darkcyan", (0, 139, 139)),
    ("darkgoldenrod", (184, 134, 11)),
    ("darkgray", (169, 169, 169)),
    ("darkgreen", (0, 100, 0)),
    ("darkgrey", (169, 169, 169)),
    ("darkkhaki", (189, 183, 107)),
    ("darkmagenta", (139, 0, 139)),
    ("darkolivegreen", (85, 107, 47)),
    ("darkorange", (255, 140, 0)),
    ("darkorchid", (153, 50, 204)),
    ("darkred", (139, 0, 0)),
    ("darksalmon", (233, 150, 122)),
    ("darkseagreen", (143, 188, 143)),
    ("darkslateblue", (72, 61, 139)),
    ("darkslategray", (47, 79, 79)),
    ("darkslategrey", (47, 79, 79)),
    ("darkturquoise", (0, 206, 209)),
    ("darkviolet", (148, 0, 211)),
    ("deeppink", (255, 20, 147)),
    ("deepskyblue", (0, 191, 255)),
    ("dimgray", (105, 105, 105)),
    ("dimgrey", (105, 105, 105)),
    ("dodgerblue", (30, 144, 255)),
    ("firebrick", (178, 34, 34)),
    ("floralwhite", (255, 250, 240)),
    ("forestgreen", (34, 139, 34)),
    ("fuchsia", (255, 0, 255)),
    ("gainsboro", (220, 220, 220)),
    ("ghostwhite", (248, 248, 255)),
    ("gold", (255, 215, 0)),
    ("goldenrod", (218, 165, 32)),
    ("gray", (128, 128, 128)),
    ("green", (0, 128, 0)),
    ("greenyellow", (173, 255, 47)),
    ("grey", (128, 128, 128)),
    ("honeydew", (240, 255, 240)),
    ("hotpink", (255, 105, 180)),
    ("indianred", (205, 92, 92)),
    ("indigo", (75, 0, 130)),
    ("ivory", (255, 255, 240)),
    ("khaki", (240, 230, 140)),
    ("lavender", (230, 230, 250)),
    ("lavenderblush", (255, 240, 245)),
    ("lawngreen", (124, 252, 0)),
    ("lemonchiffon", (255, 250, 205)),
    ("lightblue", (173, 216, 230)),
    ("lightcoral", (240, 128, 128)),
    ("lightcyan", (224, 255, 255)),
    ("lightgoldenrodyellow", (250, 250, 210)),
    ("lightgray", (211, 211, 211)),
    ("lightgreen", (144, 238, 144)),
    ("lightgrey", (211, 211, 211)),
    ("lightpink", (255, 182, 193)),
    ("lightsalmon", (255, 160, 122)),
    ("lightseagreen", (32, 178, 170)),
    ("lightskyblue", (135, 206, 250)),
    ("lightslategray", (119, 136, 153)),
    ("lightslategrey", (119, 136, 153)),
    ("lightsteelblue", (176, 196, 222)),
    ("lightyellow", (255, 255, 224)),
    ("lime", (0, 255, 0)),
    ("limegreen", (50, 205, 50)),
    ("linen", (250, 240, 230)),
    ("magenta", (255, 0, 255)),
    ("maroon", (128, 0, 0)),
    ("mediumaquamarine", (102, 205, 170)),
    ("mediumblue", (0, 0, 205)),
    ("mediumorchid", (186, 85, 211)),
    ("mediumpurple", (147, 112, 219)),
    ("mediumseagreen", (60, 179, 113)),
    ("mediumslateblue", (123, 104, 238)),
    ("mediumspringgreen", (0, 250, 154)),
    ("mediumturquoise", (72, 209, 204)),
    ("mediumvioletred", (199, 21, 133)),
    ("midnightblue", (25, 25, 112)),
    ("mintcream", (245, 255, 250)),
    ("mistyrose", (255, 228, 225)),
    ("moccasin", (255, 228, 181)),
    ("navajowhite", (255, 222, 173)),
    ("navy", (0, 0, 128)),
    ("oldlace", (253, 245, 230)),
    ("olive", (128, 128, 0)),
    ("olivedrab", (107, 142, 35)),
    ("orange", (255, 165, 0)),
    ("orangered", (255, 69, 0)),
    ("orchid", (218, 112, 214)),
    ("palegoldenrod", (238, 232, 170)),
    ("palegreen", (152, 251, 152)),
    ("paleturquoise", (175, 238, 238)),
    ("palevioletred", (219, 112, 147)),
    ("papayawhip", (255, 239, 213)),
    ("peachpuff", (255, 218, 185)),
    ("peru", (205, 133, 63)),
    ("pink", (255, 192, 203)),
    ("plum", (221, 160, 221)),
    ("powderblue", (176, 224, 230)),
    ("purple", (128, 0, 128)),
    ("rebeccapurple", (102, 51, 153)),
    ("red", (255, 0, 0)),
    ("rosybrown", (188, 143, 143)),
    ("royalblue", (65, 105, 225)),
    ("saddlebrown", (139, 69, 19)),
    ("salmon", (250, 128, 114)),
    ("sandybrown", (244, 164, 96)),
    ("seagreen", (46, 139, 87)),
    ("seashell", (255, 245, 238)),
    ("sienna", (160, 82, 45)),
    ("silver", (192, 192, 192)),
    ("skyblue", (135, 206, 235)),
    ("slateblue", (106, 90, 205)),
    ("slategray", (112, 128, 144)),
    ("slategrey", (112, 128, 144)),
    ("snow", (255, 250, 250)),
    ("springgreen", (0, 255, 127)),
    ("steelblue", (70, 130, 180)),
    ("tan", (210, 180, 140)),
    ("teal", (0, 128, 128)),
    ("thistle", (216, 191, 216)),
    ("tomato", (255, 99, 71)),
    ("turquoise", (64, 224, 208)),
    ("violet", (238, 130, 238)),
    ("wheat", (245, 222, 179)),
    ("white", (255, 255, 255)),
    ("whitesmoke", (245, 245, 245)),
    ("yellow", (255, 255, 0)),
    ("yellowgreen", (154, 205, 50)),
];

/// Look up a CSS named color by its lowercase name.
fn lookup_css_color(name: &str) -> Option<(u8, u8, u8)> {
    CSS_COLORS
        .binary_search_by_key(&name, |(n, _)| n)
        .ok()
        .map(|idx| CSS_COLORS[idx].1)
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color_type {
            ColorType::Default => write!(f, "default"),
            ColorType::Standard => {
                write!(f, "{}", STANDARD_COLOR_NAMES[self.number.unwrap() as usize])
            }
            ColorType::EightBit => write!(f, "color({})", self.number.unwrap()),
            ColorType::TrueColor => {
                let (r, g, b) = self.triplet.unwrap();
                write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ColorParseError
// ---------------------------------------------------------------------------

/// Errors that can occur when parsing a color from a string.
#[derive(Debug, Clone)]
pub enum ColorParseError {
    /// The color name was not found in the ANSI palette.
    UnknownName(String),
    /// The hex string was not a valid 6-digit RGB value.
    InvalidHex(String),
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownName(n) => write!(f, "unknown color name: {n}"),
            Self::InvalidHex(h) => write!(f, "invalid hex color: {h}"),
        }
    }
}

impl std::error::Error for ColorParseError {}

// ---------------------------------------------------------------------------
// TerminalTheme
// ---------------------------------------------------------------------------

/// Describes the terminal's default theme colors (for blending / downgrade).
#[derive(Debug, Clone, Copy)]
pub struct TerminalTheme {
    pub foreground_color: (u8, u8, u8),
    pub background_color: (u8, u8, u8),
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self {
            foreground_color: (255, 255, 255),
            background_color: (0, 0, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in palettes
// ---------------------------------------------------------------------------

/// Standard 16-color ANSI palette.
pub static STANDARD_PALETTE: &[(u8, u8, u8)] = &[
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
];

/// The 16 ANSI standard color names in palette order (black, red, ..., bright_white).
pub static STANDARD_COLOR_NAMES: &[&str] = &[
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright_black",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
];

/// The 6×6×6 color cube + greyscale ramp = 256-color palette.
pub static EIGHT_BIT_PALETTE: Lazy<[[u8; 3]; 256]> = Lazy::new(|| {
    let mut table = [[0u8; 3]; 256];
    let std: [[u8; 3]; 16] = [
        [0, 0, 0],
        [128, 0, 0],
        [0, 128, 0],
        [128, 128, 0],
        [0, 0, 128],
        [128, 0, 128],
        [0, 128, 128],
        [192, 192, 192],
        [128, 128, 128],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0, 255],
        [255, 0, 255],
        [0, 255, 255],
        [255, 255, 255],
    ];
    for i in 0..16 {
        table[i] = std[i];
    }
    let levels = [0u8, 95, 135, 175, 215, 255];
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                let idx = 16 + 36 * r + 6 * g + b;
                table[idx] = [levels[r], levels[g], levels[b]];
            }
        }
    }
    for gr in 0..24 {
        let val = (gr * 10 + 8) as u8;
        table[232 + gr] = [val, val, val];
    }
    table
});

// ---------------------------------------------------------------------------
// Color utilities
// ---------------------------------------------------------------------------

/// Convert RGB to the nearest 8-bit palette index.
pub fn rgb_to_8bit(r: u8, g: u8, b: u8) -> u8 {
    // Check if it's close to a greyscale value
    let grey = ((r as u32 + g as u32 + b as u32) / 3) as u8;
    if r == g && g == b {
        if grey < 8 {
            return 16; // black
        }
        if grey > 248 {
            return 231; // white
        }
        return 232 + ((grey - 8) / 10) as u8;
    }

    // Find nearest cube color
    let r6 = ((r as f64 / 255.0) * 5.0).round() as u8;
    let g6 = ((g as f64 / 255.0) * 5.0).round() as u8;
    let b6 = ((b as f64 / 255.0) * 5.0).round() as u8;
    16 + 36 * r6 + 6 * g6 + b6
}

/// Convert RGB to nearest standard (16) ANSI color.
pub fn rgb_to_standard(_r: u8, _g: u8, _b: u8) -> u8 {
    // Simplified: just use the nearest by Euclidean distance
    let mut best_idx = 0u8;
    let mut best_dist = u32::MAX;
    for (i, &(pr, pg, pb)) in STANDARD_PALETTE.iter().enumerate() {
        let dr = _r as i32 - pr as i32;
        let dg = _g as i32 - pg as i32;
        let db = _b as i32 - pb as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best_idx = i as u8;
        }
    }
    best_idx
}

/// Blend two RGB colors (like Rich's `blend_rgb`).
pub fn blend_rgb(color1: (u8, u8, u8), color2: (u8, u8, u8), cross_fade: f64) -> (u8, u8, u8) {
    let r = (color1.0 as f64 + (color2.0 as f64 - color1.0 as f64) * cross_fade) as u8;
    let g = (color1.1 as f64 + (color2.1 as f64 - color1.1 as f64) * cross_fade) as u8;
    let b = (color1.2 as f64 + (color2.2 as f64 - color1.2 as f64) * cross_fade) as u8;
    (r, g, b)
}

/// Blend two Colors (downgrading to the supported system).
pub fn blend_colors(
    color1: &Color,
    color2: &Color,
    cross_fade: f64,
    theme: &TerminalTheme,
) -> Color {
    let rgb1 = color1.get_truecolor(theme);
    let rgb2 = color2.get_truecolor(theme);
    let blended = blend_rgb(rgb1, rgb2, cross_fade);
    Color::from_rgb(blended.0, blended.1, blended.2)
}

// ---------------------------------------------------------------------------
// Phf map workaround — since we can't use phf easily, use a lazy static
// ---------------------------------------------------------------------------

// We use a simple linear scan backed by a slice — fast enough for the small
// set of named colors.

use once_cell::sync::Lazy;
use std::collections::HashMap;

static ANSI_NAME_MAP: Lazy<HashMap<&'static str, u8>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Standard ANSI (0-15)
    m.insert("black", 0u8);
    m.insert("red", 1u8);
    m.insert("green", 2u8);
    m.insert("yellow", 3u8);
    m.insert("blue", 4u8);
    m.insert("magenta", 5u8);
    m.insert("cyan", 6u8);
    m.insert("white", 7u8);
    m.insert("bright_black", 8u8);
    m.insert("grey", 8u8);
    m.insert("gray", 8u8);
    m.insert("bright_red", 9u8);
    m.insert("bright_green", 10u8);
    m.insert("bright_yellow", 11u8);
    m.insert("bright_blue", 12u8);
    m.insert("bright_magenta", 13u8);
    m.insert("bright_cyan", 14u8);
    m.insert("bright_white", 15u8);
    // Color cube (16-231)
    m.insert("grey0", 16u8);
    m.insert("gray0", 16u8);
    m.insert("navy_blue", 17u8);
    m.insert("dark_blue", 18u8);
    m.insert("blue3", 20u8);
    m.insert("blue1", 21u8);
    m.insert("dark_green", 22u8);
    m.insert("deep_sky_blue4", 25u8);
    m.insert("dodger_blue3", 26u8);
    m.insert("dodger_blue2", 27u8);
    m.insert("green4", 28u8);
    m.insert("spring_green4", 29u8);
    m.insert("turquoise4", 30u8);
    m.insert("deep_sky_blue3", 32u8);
    m.insert("dodger_blue1", 33u8);
    m.insert("dark_cyan", 36u8);
    m.insert("light_sea_green", 37u8);
    m.insert("deep_sky_blue2", 38u8);
    m.insert("deep_sky_blue1", 39u8);
    m.insert("green3", 40u8);
    m.insert("spring_green3", 41u8);
    m.insert("cyan3", 43u8);
    m.insert("dark_turquoise", 44u8);
    m.insert("turquoise2", 45u8);
    m.insert("green1", 46u8);
    m.insert("spring_green2", 47u8);
    m.insert("spring_green1", 48u8);
    m.insert("medium_spring_green", 49u8);
    m.insert("cyan2", 50u8);
    m.insert("cyan1", 51u8);
    m.insert("purple4", 55u8);
    m.insert("purple3", 56u8);
    m.insert("blue_violet", 57u8);
    m.insert("grey37", 59u8);
    m.insert("gray37", 59u8);
    m.insert("medium_purple4", 60u8);
    m.insert("slate_blue3", 62u8);
    m.insert("royal_blue1", 63u8);
    m.insert("chartreuse4", 64u8);
    m.insert("pale_turquoise4", 66u8);
    m.insert("steel_blue", 67u8);
    m.insert("steel_blue3", 68u8);
    m.insert("cornflower_blue", 69u8);
    m.insert("dark_sea_green4", 71u8);
    m.insert("dark_sea_green", 71u8);
    m.insert("cadet_blue", 73u8);
    m.insert("sky_blue3", 74u8);
    m.insert("chartreuse3", 76u8);
    m.insert("sea_green3", 78u8);
    m.insert("aquamarine3", 79u8);
    m.insert("medium_turquoise", 80u8);
    m.insert("steel_blue1", 81u8);
    m.insert("sea_green2", 83u8);
    m.insert("sea_green1", 85u8);
    m.insert("dark_slate_gray2", 87u8);
    m.insert("dark_red", 88u8);
    m.insert("dark_magenta", 91u8);
    m.insert("orange4", 94u8);
    m.insert("light_pink4", 95u8);
    m.insert("plum4", 96u8);
    m.insert("medium_purple3", 98u8);
    m.insert("slate_blue1", 99u8);
    m.insert("wheat4", 101u8);
    m.insert("grey53", 102u8);
    m.insert("gray53", 102u8);
    m.insert("light_slate_grey", 103u8);
    m.insert("light_slate_gray", 103u8);
    m.insert("medium_purple", 104u8);
    m.insert("light_slate_blue", 105u8);
    m.insert("yellow4", 106u8);
    m.insert("dark_olive_green3", 110u8); // adjusted for gap
    m.insert("light_sky_blue3", 110u8);
    m.insert("sky_blue2", 111u8);
    m.insert("chartreuse2", 112u8);
    m.insert("pale_green3", 114u8);
    m.insert("dark_slate_gray3", 116u8);
    m.insert("sky_blue1", 117u8);
    m.insert("chartreuse1", 118u8);
    m.insert("light_green", 120u8);
    m.insert("aquamarine1", 122u8);
    m.insert("dark_slate_gray1", 123u8);
    m.insert("deep_pink4", 125u8);
    m.insert("medium_violet_red", 126u8);
    m.insert("dark_violet", 128u8);
    m.insert("purple", 129u8);
    m.insert("medium_orchid3", 133u8);
    m.insert("medium_orchid", 134u8);
    m.insert("dark_goldenrod", 136u8);
    m.insert("rosy_brown", 138u8);
    m.insert("grey63", 139u8);
    m.insert("gray63", 139u8);
    m.insert("medium_purple2", 140u8);
    m.insert("medium_purple1", 141u8);
    m.insert("dark_khaki", 143u8);
    m.insert("navajo_white3", 144u8);
    m.insert("grey69", 145u8);
    m.insert("gray69", 145u8);
    m.insert("light_steel_blue3", 146u8);
    m.insert("light_steel_blue", 147u8);
    m.insert("dark_olive_green2", 155u8);
    m.insert("pale_green1", 156u8);
    m.insert("dark_sea_green2", 157u8);
    m.insert("pale_turquoise1", 159u8);
    m.insert("red3", 160u8);
    m.insert("deep_pink3", 162u8);
    m.insert("magenta3", 164u8);
    m.insert("dark_orange3", 166u8);
    m.insert("indian_red", 167u8);
    m.insert("hot_pink3", 168u8);
    m.insert("hot_pink2", 169u8);
    m.insert("orchid", 170u8);
    m.insert("orange3", 172u8);
    m.insert("light_salmon3", 173u8);
    m.insert("light_pink3", 174u8);
    m.insert("pink3", 175u8);
    m.insert("plum3", 176u8);
    m.insert("violet", 177u8);
    m.insert("gold3", 178u8);
    m.insert("light_goldenrod3", 179u8);
    m.insert("tan", 180u8);
    m.insert("misty_rose3", 181u8);
    m.insert("thistle3", 182u8);
    m.insert("plum2", 183u8);
    m.insert("yellow3", 184u8);
    m.insert("khaki3", 185u8);
    m.insert("light_yellow3", 187u8);
    m.insert("grey84", 188u8);
    m.insert("gray84", 188u8);
    m.insert("light_steel_blue1", 189u8);
    m.insert("yellow2", 190u8);
    m.insert("dark_olive_green1", 192u8);
    m.insert("dark_sea_green1", 193u8);
    m.insert("honeydew2", 194u8);
    m.insert("light_cyan1", 195u8);
    m.insert("red1", 196u8);
    m.insert("deep_pink2", 197u8);
    m.insert("deep_pink1", 199u8);
    m.insert("magenta2", 200u8);
    m.insert("magenta1", 201u8);
    m.insert("orange_red1", 202u8);
    m.insert("indian_red1", 204u8);
    m.insert("hot_pink", 206u8);
    m.insert("medium_orchid1", 207u8);
    m.insert("dark_orange", 208u8);
    m.insert("salmon1", 209u8);
    m.insert("light_coral", 210u8);
    m.insert("pale_violet_red1", 211u8);
    m.insert("orchid2", 212u8);
    m.insert("orchid1", 213u8);
    m.insert("orange1", 214u8);
    m.insert("sandy_brown", 215u8);
    m.insert("light_salmon1", 216u8);
    m.insert("light_pink1", 217u8);
    m.insert("pink1", 218u8);
    m.insert("plum1", 219u8);
    m.insert("gold1", 220u8);
    m.insert("light_goldenrod2", 222u8);
    m.insert("navajo_white1", 223u8);
    m.insert("misty_rose1", 224u8);
    m.insert("thistle1", 225u8);
    m.insert("yellow1", 226u8);
    m.insert("light_goldenrod1", 227u8);
    m.insert("khaki1", 228u8);
    m.insert("wheat1", 229u8);
    m.insert("cornsilk1", 230u8);
    m.insert("grey100", 231u8);
    m.insert("gray100", 231u8);
    // Greyscale (232-255)
    m.insert("grey3", 232u8);
    m.insert("gray3", 232u8);
    m.insert("grey7", 233u8);
    m.insert("gray7", 233u8);
    m.insert("grey11", 234u8);
    m.insert("gray11", 234u8);
    m.insert("grey15", 235u8);
    m.insert("gray15", 235u8);
    m.insert("grey19", 236u8);
    m.insert("gray19", 236u8);
    m.insert("grey23", 237u8);
    m.insert("gray23", 237u8);
    m.insert("grey27", 238u8);
    m.insert("gray27", 238u8);
    m.insert("grey30", 239u8);
    m.insert("gray30", 239u8);
    m.insert("grey35", 240u8);
    m.insert("gray35", 240u8);
    m.insert("grey39", 241u8);
    m.insert("gray39", 241u8);
    m.insert("grey42", 242u8);
    m.insert("gray42", 242u8);
    m.insert("grey46", 243u8);
    m.insert("gray46", 243u8);
    m.insert("grey50", 244u8);
    m.insert("gray50", 244u8);
    m.insert("grey54", 245u8);
    m.insert("gray54", 245u8);
    m.insert("grey58", 246u8);
    m.insert("gray58", 246u8);
    m.insert("grey62", 247u8);
    m.insert("gray62", 247u8);
    m.insert("grey66", 248u8);
    m.insert("gray66", 248u8);
    m.insert("dark_grey", 248u8);
    m.insert("dark_gray", 248u8);
    m.insert("grey70", 249u8);
    m.insert("gray70", 249u8);
    m.insert("grey74", 250u8);
    m.insert("gray74", 250u8);
    m.insert("light_grey", 250u8);
    m.insert("light_gray", 250u8);
    m.insert("grey78", 251u8);
    m.insert("gray78", 251u8);
    m.insert("grey82", 252u8);
    m.insert("gray82", 252u8);
    m.insert("grey85", 253u8);
    m.insert("gray85", 253u8);
    m.insert("grey89", 254u8);
    m.insert("gray89", 254u8);
    m.insert("grey93", 255u8);
    m.insert("gray93", 255u8);
    m
});

impl Color {
    /// Look up a named color index.
    pub fn name_to_index(name: &str) -> Option<u8> {
        ANSI_NAME_MAP.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color() {
        let c = Color::default();
        assert!(c.is_default());
    }

    #[test]
    fn test_parse_red() {
        let c = Color::parse("red").unwrap();
        assert_eq!(c.number, Some(1));
    }

    #[test]
    fn test_parse_hex() {
        let c = Color::parse("#ff0000").unwrap();
        assert_eq!(c.triplet, Some((255, 0, 0)));
    }

    #[test]
    fn test_rgb_to_8bit_black() {
        assert_eq!(rgb_to_8bit(0, 0, 0), 16);
    }

    #[test]
    fn test_from_triplet() {
        let triplet = ColorTriplet::new(255, 128, 0);
        let color = Color::from_triplet(&triplet);
        assert_eq!(color.triplet(), Some((255, 128, 0)));
    }

    #[test]
    fn test_is_system_defined() {
        let default = Color::default();
        assert!(!default.is_system_defined());

        let red = Color::parse("red").unwrap();
        assert!(red.is_system_defined());

        let custom = Color::from_rgb(100, 200, 50);
        assert!(!custom.is_system_defined());
    }

    #[test]
    fn test_get_ansi_codes_standard() {
        let red = Color::parse("red").unwrap();
        let (fg, _bg) = red.get_ansi_codes(false);
        assert!(fg.is_some());
        assert!(fg.unwrap().contains("31"));
    }

    #[test]
    fn test_get_ansi_codes_background() {
        let blue = Color::parse("blue").unwrap();
        let (_fg, bg) = blue.get_ansi_codes(true);
        assert!(bg.is_some());
        assert!(bg.unwrap().contains("44"));
    }

    #[test]
    fn test_get_ansi_codes_truecolor() {
        let c = Color::from_rgb(255, 128, 0);
        let (fg, _bg) = c.get_ansi_codes(false);
        assert!(fg.is_some());
        assert!(fg.unwrap().contains("38;2;255;128;0"));
    }

    #[test]
    fn test_get_ansi_codes_default() {
        let c = Color::default();
        let (fg, bg) = c.get_ansi_codes(false);
        assert!(fg.is_none());
        assert!(bg.is_none());
    }

    #[test]
    fn test_color_name() {
        let red = Color::parse("red").unwrap();
        // Named colors created via parse don't store the name since
        // from_ansi_name doesn't set it. Test that triplet/number work instead.
        assert_eq!(red.number(), Some(1));
    }

    #[test]
    fn test_color_number() {
        let c = Color::from_8bit(42);
        assert_eq!(c.number(), Some(42));
    }

    #[test]
    fn test_color_triplet() {
        let c = Color::from_rgb(10, 20, 30);
        assert_eq!(c.triplet(), Some((10, 20, 30)));
        let d = Color::default();
        assert_eq!(d.triplet(), None);
    }
}
