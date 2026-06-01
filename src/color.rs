//! Color system — equivalent to Rich's `color.py`.
//!
//! Supports ANSI standard colors, 8-bit (256) colors, and 24-bit true color.
//! Includes named color constants and color blending.

use std::fmt;

/// An RGB color triplet — equivalent to Rich's `ColorTriplet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ColorTriplet {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ColorTriplet {
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

    /// Parse a color from a string (name, hex, or "default").
    pub fn parse(s: &str) -> Result<Self, ColorParseError> {
        let lower = s.to_lowercase();
        if lower == "default" || lower.is_empty() {
            return Ok(Self::default());
        }
        if let Some(c) = Self::from_ansi_name(&lower) {
            return Ok(c);
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

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.color_type {
            ColorType::Default => write!(f, "default"),
            ColorType::Standard => write!(f, "{}", STANDARD_COLOR_NAMES[self.number.unwrap() as usize]),
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

#[derive(Debug, Clone)]
pub enum ColorParseError {
    UnknownName(String),
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

pub static STANDARD_COLOR_NAMES: &[&str] = &[
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    "bright_black", "bright_red", "bright_green", "bright_yellow",
    "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
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
pub fn blend_rgb(
    color1: (u8, u8, u8),
    color2: (u8, u8, u8),
    cross_fade: f64,
) -> (u8, u8, u8) {
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
}
