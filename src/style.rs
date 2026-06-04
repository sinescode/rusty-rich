//! Text style — equivalent to Rich's `style.py`.
//!
//! A [`Style`] combines foreground/background color with 13 text attributes
//! (bold, dim, italic, underline, blink, reverse, strike, underline2, frame,
//! encircle, overline, blink2, conceal), plus optional link and metadata.
//!
//! # Quick Example
//!
//! ```rust
//! use rusty_rich::{Style, Color};
//!
//! let style = Style::new()
//!     .color(Color::parse("cyan").unwrap())
//!     .bgcolor(Color::parse("#1E1E2E").unwrap())
//!     .bold(true)
//!     .italic(true);
//!
//! // Parse from a string
//! let parsed = Style::from_str("bold red on blue");
//! ```
//!
//! # Style Combination
//!
//! Styles combine left-to-right via [`Style::combine`] with a 3-state attribute
//! cascade: explicit `true` wins over inherit, explicit `false` resets, and
//! unset falls through to the parent.
//!
//! # StyleStack
//!
//! [`StyleStack`] tracks nested style inheritance for markup parsing. Push
//! a style when entering a tag, pop when leaving.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::color::{Color, ColorType, EIGHT_BIT_PALETTE, STANDARD_COLOR_NAMES, STANDARD_PALETTE};

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// Style attributes — bit flags
// ---------------------------------------------------------------------------

/// Bit flags for text attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attributes(u32);

impl Attributes {
    /// Bit flag for bold text.
    pub const BOLD: u32 = 1 << 0;
    /// Bit flag for dim/dark text.
    pub const DIM: u32 = 1 << 1;
    /// Bit flag for italic text.
    pub const ITALIC: u32 = 1 << 2;
    /// Bit flag for underlined text.
    pub const UNDERLINE: u32 = 1 << 3;
    /// Bit flag for blinking text.
    pub const BLINK: u32 = 1 << 4;
    /// Bit flag for reverse-video text.
    pub const REVERSE: u32 = 1 << 5;
    /// Bit flag for strikethrough text.
    pub const STRIKE: u32 = 1 << 6;
    /// Bit flag for double underline.
    pub const UNDERLINE2: u32 = 1 << 7;
    /// Bit flag for framed text.
    pub const FRAME: u32 = 1 << 8;
    /// Bit flag for encircled text.
    pub const ENCIRCLE: u32 = 1 << 9;
    /// Bit flag for overlined text.
    pub const OVERLINE: u32 = 1 << 10;
    /// Bit flag for rapid blink.
    pub const BLINK2: u32 = 1 << 11;
    /// Bit flag for concealed/hidden text.
    pub const CONCEAL: u32 = 1 << 12;

    /// Create an empty set of attributes (no flags set).
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Set or clear a specific attribute bit.
    pub fn set(&mut self, bit: u32, value: bool) {
        if value {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }

    /// Check whether a specific attribute bit is set.
    pub fn get(&self, bit: u32) -> bool {
        self.0 & bit != 0
    }

    /// Return the raw bitmask value.
    pub const fn bits(&self) -> u32 {
        self.0
    }
}

/// All 13 style attribute bits in order (for iteration).
pub const STYLE_BITS: &[u32] = &[
    Attributes::BOLD, Attributes::DIM, Attributes::ITALIC,
    Attributes::UNDERLINE, Attributes::BLINK, Attributes::REVERSE,
    Attributes::STRIKE, Attributes::UNDERLINE2, Attributes::FRAME,
    Attributes::ENCIRCLE, Attributes::OVERLINE, Attributes::BLINK2,
    Attributes::CONCEAL,
];

/// All 13 style attribute (name, bit) pairs for iteration.
pub const STYLE_ATTRIBUTES: &[(&str, u32)] = &[
    ("bold", Attributes::BOLD),
    ("dim", Attributes::DIM),
    ("italic", Attributes::ITALIC),
    ("underline", Attributes::UNDERLINE),
    ("blink", Attributes::BLINK),
    ("reverse", Attributes::REVERSE),
    ("strike", Attributes::STRIKE),
    ("underline2", Attributes::UNDERLINE2),
    ("frame", Attributes::FRAME),
    ("encircle", Attributes::ENCIRCLE),
    ("overline", Attributes::OVERLINE),
    ("blink2", Attributes::BLINK2),
    ("conceal", Attributes::CONCEAL),
];

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<&str> = Vec::new();
        if self.get(Self::BOLD) { parts.push("bold"); }
        if self.get(Self::DIM) { parts.push("dim"); }
        if self.get(Self::ITALIC) { parts.push("italic"); }
        if self.get(Self::UNDERLINE) { parts.push("underline"); }
        if self.get(Self::BLINK) { parts.push("blink"); }
        if self.get(Self::REVERSE) { parts.push("reverse"); }
        if self.get(Self::CONCEAL) { parts.push("conceal"); }
        if self.get(Self::STRIKE) { parts.push("strike"); }
        if self.get(Self::OVERLINE) { parts.push("overline"); }
        if parts.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

// ---------------------------------------------------------------------------
// Style
// ---------------------------------------------------------------------------

/// A constant null (empty) style. Use instead of `Style::null()` to avoid
/// allocation when you need a null style repeatedly.
pub const NULL_STYLE: Style = Style {
    color: None,
    bgcolor: None,
    attributes: Attributes(0),
    set_attributes: 0,
    link: None,
    link_id: 0,
    is_null: true,
    meta: None,
};

/// A terminal style.
///
/// Supports foreground color, background color, attributes, and an optional
/// hyperlink. Attributes use a three-state system: set to `true`, set to
/// `false`, or not set (`None`).
#[derive(Debug, Clone)]
pub struct Style {
    pub(crate) color: Option<Color>,
    pub(crate) bgcolor: Option<Color>,
    pub(crate) attributes: Attributes,
    /// Which attribute bits have been explicitly set (vs inherited).
    pub(crate) set_attributes: u32,
    pub(crate) link: Option<String>,
    pub(crate) link_id: u32,
    pub(crate) is_null: bool,
    /// Arbitrary metadata attached to this style.
    pub(crate) meta: Option<Vec<u8>>,
}

impl Style {
    // -- constructors -------------------------------------------------------

    /// Create a null (empty) style.
    ///
    /// For a zero-allocation alternative, use the [`NULL_STYLE`] constant
    /// directly or clone it.
    pub fn null() -> Self {
        NULL_STYLE.clone()
    }

    /// Create a new style with optional settings.
    pub fn new() -> Self {
        Self {
            color: None,
            bgcolor: None,
            attributes: Attributes::empty(),
            set_attributes: 0,
            link: None,
            link_id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            is_null: false,
            meta: None,
        }
    }

    /// Builder: set foreground color.
    pub fn color(mut self, color: impl Into<Option<Color>>) -> Self {
        self.color = color.into();
        self
    }

    /// Builder: set background color.
    pub fn bgcolor(mut self, bgcolor: impl Into<Option<Color>>) -> Self {
        self.bgcolor = bgcolor.into();
        self
    }

    /// Builder: set bold.
    pub fn bold(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::BOLD;
        self.attributes.set(Attributes::BOLD, value);
        self
    }

    /// Builder: set dim.
    pub fn dim(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::DIM;
        self.attributes.set(Attributes::DIM, value);
        self
    }

    /// Builder: set italic.
    pub fn italic(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::ITALIC;
        self.attributes.set(Attributes::ITALIC, value);
        self
    }

    /// Builder: set underline.
    pub fn underline(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::UNDERLINE;
        self.attributes.set(Attributes::UNDERLINE, value);
        self
    }

    /// Builder: set blink.
    pub fn blink(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::BLINK;
        self.attributes.set(Attributes::BLINK, value);
        self
    }

    /// Builder: set reverse.
    pub fn reverse(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::REVERSE;
        self.attributes.set(Attributes::REVERSE, value);
        self
    }

    /// Builder: set strikethrough.
    pub fn strike(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::STRIKE;
        self.attributes.set(Attributes::STRIKE, value);
        self
    }

    /// Builder: set blink2 (rapid blink).
    pub fn blink2(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::BLINK2;
        self.attributes.set(Attributes::BLINK2, value);
        self
    }

    /// Builder: set conceal.
    pub fn conceal(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::CONCEAL;
        self.attributes.set(Attributes::CONCEAL, value);
        self
    }

    /// Builder: set double underline.
    pub fn underline2(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::UNDERLINE2;
        self.attributes.set(Attributes::UNDERLINE2, value);
        self
    }

    /// Builder: set frame.
    pub fn frame(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::FRAME;
        self.attributes.set(Attributes::FRAME, value);
        self
    }

    /// Builder: set encircle.
    pub fn encircle(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::ENCIRCLE;
        self.attributes.set(Attributes::ENCIRCLE, value);
        self
    }

    /// Builder: set overline.
    pub fn overline(mut self, value: bool) -> Self {
        self.set_attributes |= Attributes::OVERLINE;
        self.attributes.set(Attributes::OVERLINE, value);
        self
    }

    /// Return a copy with foreground and background colors stripped.
    pub fn without_color(&self) -> Self {
        let mut s = self.clone();
        s.color = None;
        s.bgcolor = None;
        s
    }

    /// Return a style with the background color set to the foreground color,
    /// useful for background-only rendering.
    pub fn background_style(&self) -> Self {
        let mut s = Self::new();
        s.bgcolor = self.color.clone();
        s
    }

    /// Returns true if the background is not set (transparent).
    pub fn transparent_background(&self) -> bool {
        self.bgcolor.is_none()
    }

    /// Builder: set link.
    pub fn link(mut self, url: impl Into<String>) -> Self {
        self.link = Some(url.into());
        self.link_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        self
    }

    /// Builder: set style from a string (e.g. "bold red on blue").
    ///
    /// Supports negation with `not` prefix, `!` prefix, and `no` prefix
    /// (e.g. `"not bold"`, `"!bold"`, `"nobold"`). The `not` keyword works
    /// as a prefix for the NEXT token, so `"not bold not italic"` correctly
    /// disables both attributes.
    pub fn from_str(definition: &str) -> Self {
        let mut style = Self::new();
        let parts: Vec<&str> = definition.split_whitespace().collect();
        let mut i = 0;
        let mut negate = false;
        let mut saw_on = false;

        while i < parts.len() {
            let part = parts[i];

            // Handle standalone "not" prefix (negates next token)
            if part == "not" {
                negate = true;
                i += 1;
                continue;
            }

            match part {
                "bold" | "b" => {
                    style.set_attributes |= Attributes::BOLD;
                    style.attributes.set(Attributes::BOLD, !negate);
                }
                "dim" | "d" => {
                    style.set_attributes |= Attributes::DIM;
                    style.attributes.set(Attributes::DIM, !negate);
                }
                "italic" | "i" => {
                    style.set_attributes |= Attributes::ITALIC;
                    style.attributes.set(Attributes::ITALIC, !negate);
                }
                "underline" | "u" => {
                    style.set_attributes |= Attributes::UNDERLINE;
                    style.attributes.set(Attributes::UNDERLINE, !negate);
                }
                "blink" => {
                    style.set_attributes |= Attributes::BLINK;
                    style.attributes.set(Attributes::BLINK, !negate);
                }
                "reverse" | "r" => {
                    style.set_attributes |= Attributes::REVERSE;
                    style.attributes.set(Attributes::REVERSE, !negate);
                }
                "strike" | "s" => {
                    style.set_attributes |= Attributes::STRIKE;
                    style.attributes.set(Attributes::STRIKE, !negate);
                }
                "none" | "default" => {}
                "on" => {
                    saw_on = true;
                    // If next token is a color, consume it as background
                    if i + 1 < parts.len() {
                        if let Ok(c) = Color::parse(parts[i + 1]) {
                            style.bgcolor = Some(c);
                            i += 1; // consumed the color token
                        }
                    }
                }
                part if part.starts_with('!') => {
                    // Inline negation: !bold, !italic, etc.
                    let inner = &part[1..];
                    let (bit, _name) = match inner {
                        "bold" | "b" => (Attributes::BOLD, "bold"),
                        "dim" | "d" => (Attributes::DIM, "dim"),
                        "italic" | "i" => (Attributes::ITALIC, "italic"),
                        "underline" | "u" => (Attributes::UNDERLINE, "underline"),
                        "blink" => (Attributes::BLINK, "blink"),
                        "reverse" | "r" => (Attributes::REVERSE, "reverse"),
                        "strike" | "s" => (Attributes::STRIKE, "strike"),
                        _ => {
                            // Not a known attribute — skip
                            i += 1;
                            negate = false;
                            continue;
                        }
                    };
                    style.set_attributes |= bit;
                    style.attributes.set(bit, false);
                }
                part if part.starts_with("no") && part.len() > 2 => {
                    // Prefix negation: nobold, noitalic, nounderline
                    let inner = &part[2..];
                    let (bit, _name) = match inner {
                        "bold" => (Attributes::BOLD, "bold"),
                        "italic" => (Attributes::ITALIC, "italic"),
                        "underline" => (Attributes::UNDERLINE, "underline"),
                        _ => {
                            // Not a known attribute — skip
                            i += 1;
                            negate = false;
                            continue;
                        }
                    };
                    style.set_attributes |= bit;
                    style.attributes.set(bit, false);
                }
                part if part.starts_with("link=") => {
                    style.link = Some(part[5..].to_string());
                }
                part if part.starts_with("on ") => {
                    if let Ok(c) = Color::parse(&part[3..]) {
                        style.bgcolor = Some(c);
                    }
                }
                part => {
                    // Try as color name
                    if let Ok(c) = Color::parse(part) {
                        if saw_on {
                            style.bgcolor = Some(c);
                            saw_on = false;
                        } else {
                            style.color = Some(c);
                        }
                    }
                }
            }
            negate = false;
            i += 1;
        }
        style
    }

    // -- queries ------------------------------------------------------------

    /// Returns `true` if this is a null (empty) style.
    pub fn is_null(&self) -> bool {
        self.is_null
    }

    /// Returns `true` if this style has no colors, attributes, or link set.
    pub fn is_plain(&self) -> bool {
        self.color.is_none()
            && self.bgcolor.is_none()
            && self.set_attributes == 0
            && self.link.is_none()
    }

    /// Check if the bold attribute is explicitly set to `true`.
    pub fn get_bold(&self) -> Option<bool> {
        if self.set_attributes & Attributes::BOLD != 0 {
            Some(self.attributes.get(Attributes::BOLD))
        } else {
            None
        }
    }

    /// Check if the italic attribute is explicitly set to `true`.
    pub fn get_italic(&self) -> Option<bool> {
        if self.set_attributes & Attributes::ITALIC != 0 {
            Some(self.attributes.get(Attributes::ITALIC))
        } else {
            None
        }
    }

    /// Merge two styles: `self` is the base, `other` overrides.
    pub fn combine(&self, other: &Style) -> Style {
        if other.is_null {
            return self.clone();
        }
        if self.is_null {
            return other.clone();
        }

        let mut combined = self.clone();
        if other.color.is_some() {
            combined.color = other.color.clone();
        }
        if other.bgcolor.is_some() {
            combined.bgcolor = other.bgcolor.clone();
        }
        // Attributes: other's set bits override self (3-state cascade)
        for &bit in STYLE_BITS {
            if other.set_attributes & bit != 0 {
                combined.set_attributes |= bit;
                combined.attributes.set(bit, other.attributes.get(bit));
            }
        }
        if other.link.is_some() {
            combined.link = other.link.clone();
            combined.link_id = other.link_id;
        }
        if other.meta.is_some() {
            combined.meta = other.meta.clone();
        }
        combined.is_null = false;
        combined
    }

    /// Render this style as ANSI SGR escape sequences.
    ///
    /// Uses a pre-allocated `String` with direct `push_str` instead of
    /// `Vec<String>` + `join()`, avoiding ~3× allocations in the render
    /// hot path.
    pub fn to_ansi(&self) -> String {
        if self.is_null {
            return String::new();
        }
        let mut out = String::with_capacity(48);
        let mut first = true;

        // Macro to push a simple code with proper separator
        macro_rules! push_code {
            ($code:expr) => {{
                if first {
                    out.push_str("\x1b[");
                    first = false;
                } else {
                    out.push(';');
                }
                out.push_str($code);
            }};
        }

        // Foreground color
        if let Some(ref c) = self.color {
            match c.color_type {
                crate::color::ColorType::Default => push_code!("39"),
                crate::color::ColorType::Standard => {
                    if let Some(n) = c.number {
                        let code = if n < 8 { 30 + n } else { 82 + n };
                        push_code!(&code.to_string());
                    }
                }
                crate::color::ColorType::EightBit => {
                    if let Some(n) = c.number {
                        out.push_str(if first { "\x1b[38;5;" } else { ";38;5;" });
                        first = false;
                        out.push_str(&n.to_string());
                    }
                }
                crate::color::ColorType::TrueColor => {
                    if let Some((r, g, b)) = c.triplet {
                        out.push_str(if first { "\x1b[38;2;" } else { ";38;2;" });
                        first = false;
                        out.push_str(&format!("{r};{g};{b}"));
                    }
                }
            }
        }

        // Background color
        if let Some(ref c) = self.bgcolor {
            match c.color_type {
                crate::color::ColorType::Default => push_code!("49"),
                crate::color::ColorType::Standard => {
                    if let Some(n) = c.number {
                        let code = if n < 8 { 40 + n } else { 92 + n };
                        push_code!(&code.to_string());
                    }
                }
                crate::color::ColorType::EightBit => {
                    if let Some(n) = c.number {
                        out.push_str(if first { "\x1b[48;5;" } else { ";48;5;" });
                        first = false;
                        out.push_str(&n.to_string());
                    }
                }
                crate::color::ColorType::TrueColor => {
                    if let Some((r, g, b)) = c.triplet {
                        out.push_str(if first { "\x1b[48;2;" } else { ";48;2;" });
                        first = false;
                        out.push_str(&format!("{r};{g};{b}"));
                    }
                }
            }
        }

        // Attributes — use push_code! macro for single-code attributes
        if self.set_attributes & Attributes::BOLD != 0 {
            push_code!(if self.attributes.get(Attributes::BOLD) { "1" } else { "22" });
        }
        if self.set_attributes & Attributes::DIM != 0 {
            push_code!(if self.attributes.get(Attributes::DIM) { "2" } else { "22" });
        }
        if self.set_attributes & Attributes::ITALIC != 0 {
            push_code!(if self.attributes.get(Attributes::ITALIC) { "3" } else { "23" });
        }
        if self.set_attributes & Attributes::UNDERLINE != 0 {
            push_code!(if self.attributes.get(Attributes::UNDERLINE) { "4" } else { "24" });
        }
        if self.set_attributes & Attributes::BLINK != 0 {
            push_code!(if self.attributes.get(Attributes::BLINK) { "5" } else { "25" });
        }
        if self.set_attributes & Attributes::REVERSE != 0 {
            push_code!(if self.attributes.get(Attributes::REVERSE) { "7" } else { "27" });
        }
        if self.set_attributes & Attributes::CONCEAL != 0 {
            push_code!(if self.attributes.get(Attributes::CONCEAL) { "8" } else { "28" });
        }
        if self.set_attributes & Attributes::STRIKE != 0 {
            push_code!(if self.attributes.get(Attributes::STRIKE) { "9" } else { "29" });
        }
        if self.set_attributes & Attributes::UNDERLINE2 != 0 {
            push_code!(if self.attributes.get(Attributes::UNDERLINE2) { "21" } else { "24" });
        }
        if self.set_attributes & Attributes::BLINK2 != 0 {
            push_code!(if self.attributes.get(Attributes::BLINK2) { "6" } else { "25" });
        }
        if self.set_attributes & Attributes::FRAME != 0 {
            push_code!(if self.attributes.get(Attributes::FRAME) { "51" } else { "54" });
        }
        if self.set_attributes & Attributes::ENCIRCLE != 0 {
            push_code!(if self.attributes.get(Attributes::ENCIRCLE) { "52" } else { "54" });
        }
        if self.set_attributes & Attributes::OVERLINE != 0 {
            push_code!(if self.attributes.get(Attributes::OVERLINE) { "53" } else { "55" });
        }

        if !first {
            out.push('m');
        }
        out
    }

    /// Return the ANSI reset sequence needed to turn off this style.
    pub fn reset_ansi(&self) -> &'static str {
        "\x1b[0m"
    }

    // -- chaining --------------------------------------------------------------

    /// Create a chain-of-styles fallback. When `self` has a value set, use it;
    /// otherwise fall through to `other`.
    pub fn chain(&self, other: &Style) -> Style {
        let mut result = Style::new();
        result.color = self.color.clone().or_else(|| other.color.clone());
        result.bgcolor = self.bgcolor.clone().or_else(|| other.bgcolor.clone());
        result.link = self.link.clone().or_else(|| other.link.clone());
        result.meta = self.meta.clone().or_else(|| other.meta.clone());
        for &bit in STYLE_BITS {
            if self.set_attributes & bit != 0 {
                result.set_attributes |= bit;
                result.attributes.set(bit, self.attributes.get(bit));
            } else if other.set_attributes & bit != 0 {
                result.set_attributes |= bit;
                result.attributes.set(bit, other.attributes.get(bit));
            }
        }
        result
    }

    // -- copy / clear ----------------------------------------------------------

    /// Explicit clone (delegates to Clone, named for Python parity).
    pub fn copy(&self) -> Style {
        self.clone()
    }

    /// Clear the meta field and link field, returning self for chaining.
    pub fn clear_meta_and_links(&mut self) -> &mut Self {
        self.meta = None;
        self.link = None;
        self
    }

    // -- constructors ----------------------------------------------------------

    /// Create a style with just a foreground color set.
    pub fn from_color(color: Color) -> Self {
        Self::new().color(color)
    }

    /// Create a style with metadata.
    pub fn from_meta(meta: Vec<u8>) -> Self {
        let mut s = Self::new();
        s.meta = Some(meta);
        s
    }

    // -- html export -----------------------------------------------------------

    /// Generate CSS style string for HTML export.
    pub fn get_html_style(&self, _theme: Option<&crate::export::ExportTheme>) -> String {
        if self.is_null {
            return String::new();
        }
        let mut parts: Vec<String> = Vec::new();

        if let Some(ref c) = self.color {
            let hex = color_to_css_hex(c);
            if !hex.is_empty() {
                parts.push(format!("color: {}", hex));
            }
        }
        if let Some(ref c) = self.bgcolor {
            let hex = color_to_css_hex(c);
            if !hex.is_empty() {
                parts.push(format!("background-color: {}", hex));
            }
        }
        if self.set_attributes & Attributes::BOLD != 0 && self.attributes.get(Attributes::BOLD) {
            parts.push("font-weight: bold".into());
        }
        if self.set_attributes & Attributes::ITALIC != 0 && self.attributes.get(Attributes::ITALIC) {
            parts.push("font-style: italic".into());
        }

        // text-decoration: combine underline, strike, blink
        let mut decor: Vec<&str> = Vec::new();
        if self.set_attributes & Attributes::UNDERLINE != 0
            && self.attributes.get(Attributes::UNDERLINE)
        {
            decor.push("underline");
        }
        if self.set_attributes & Attributes::UNDERLINE2 != 0
            && self.attributes.get(Attributes::UNDERLINE2)
        {
            decor.push("underline");
        }
        if self.set_attributes & Attributes::STRIKE != 0
            && self.attributes.get(Attributes::STRIKE)
        {
            decor.push("line-through");
        }
        if (self.set_attributes & Attributes::BLINK != 0
            && self.attributes.get(Attributes::BLINK))
            || (self.set_attributes & Attributes::BLINK2 != 0
                && self.attributes.get(Attributes::BLINK2))
        {
            decor.push("blink");
        }
        if !decor.is_empty() {
            parts.push(format!("text-decoration: {}", decor.join(" ")));
        }

        // reverse video → CSS invert filter
        if self.set_attributes & Attributes::REVERSE != 0
            && self.attributes.get(Attributes::REVERSE)
        {
            parts.push("filter: invert(100%)".into());
        }

        // conceal → hidden visibility
        if self.set_attributes & Attributes::CONCEAL != 0
            && self.attributes.get(Attributes::CONCEAL)
        {
            parts.push("visibility: hidden".into());
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("; ")
        }
    }

    // -- normalize -------------------------------------------------------------

    /// Return a "normalized" style: remove negative (explicitly false) attributes
    /// that just reset inherited ones. Only keep explicitly true attributes and
    /// colors.
    pub fn normalize(&self) -> Style {
        let mut s = Style::new();
        s.color = self.color.clone();
        s.bgcolor = self.bgcolor.clone();
        s.link = self.link.clone();
        s.link_id = self.link_id;
        s.meta = self.meta.clone();
        for &bit in STYLE_BITS {
            if self.set_attributes & bit != 0 && self.attributes.get(bit) {
                s.set_attributes |= bit;
                s.attributes.set(bit, true);
            }
        }
        s
    }

    // -- utility ---------------------------------------------------------------

    /// Return the "first" significant color name for display purposes
    /// (fg color name, or bg color name, or None).
    pub fn pick_first(&self) -> Option<&'static str> {
        if let Some(ref c) = self.color {
            if let Some(name) = color_to_name(c) {
                return Some(name);
            }
        }
        if let Some(ref c) = self.bgcolor {
            if let Some(name) = color_to_name(c) {
                return Some(name);
            }
        }
        None
    }

    /// Render `text` wrapped in this style's ANSI codes.
    pub fn render(&self, text: &str) -> String {
        format!("{}{}{}", self.to_ansi(), text, self.reset_ansi())
    }

    /// Render a test/demo string. If text is None, use "Lorem ipsum".
    pub fn test(&self, text: Option<&str>) -> String {
        let t = text.unwrap_or("Lorem ipsum");
        self.render(t)
    }

    /// Update or clear the link, returning self for chaining.
    pub fn update_link(&mut self, url: Option<String>) -> &mut Self {
        self.link = url;
        if self.link.is_some() {
            self.link_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        }
        self
    }

    // -- accessors -------------------------------------------------------------

    /// Get a reference to metadata.
    pub fn meta(&self) -> Option<&Vec<u8>> {
        self.meta.as_ref()
    }

    /// Get a mutable reference to metadata.
    pub fn meta_mut(&mut self) -> Option<&mut Vec<u8>> {
        self.meta.as_mut()
    }

    /// Set metadata, returning self for chaining.
    pub fn set_meta(&mut self, meta: Option<Vec<u8>>) -> &mut Self {
        self.meta = meta;
        self
    }

    /// Get the link ID.
    pub fn link_id(&self) -> u32 {
        self.link_id
    }

    /// Alias for `bgcolor()` (Python rich has both `.on()` and `.bgcolor()`).
    pub fn on(self, color: impl Into<Option<Color>>) -> Self {
        self.bgcolor(color)
    }

    /// Get a reference to the foreground color.
    pub fn color_ref(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// Get a reference to the background color.
    pub fn bgcolor_ref(&self) -> Option<&Color> {
        self.bgcolor.as_ref()
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Style {
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
            && self.bgcolor == other.bgcolor
            && self.attributes == other.attributes
            && self.set_attributes == other.set_attributes
            && self.link == other.link
    }
}

impl Eq for Style {}

impl Hash for Style {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.bgcolor.hash(state);
        self.attributes.hash(state);
        self.set_attributes.hash(state);
        self.link.hash(state);
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null {
            return write!(f, "null");
        }
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref c) = self.color {
            parts.push(c.to_string());
        }
        if let Some(ref c) = self.bgcolor {
            parts.push(format!("on {}", c));
        }
        let attrs = self.attributes.to_string();
        if attrs != "none" {
            parts.push(attrs);
        }
        if parts.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

/// Convenience type alias.
pub type StyleType = Style;

// -- helper functions for html export and color name lookup ------------------

/// Convert a `Color` to a CSS hex string `#rrggbb`.
fn color_to_css_hex(c: &Color) -> String {
    match c.color_type {
        ColorType::Default => String::new(),
        ColorType::Standard => {
            if let Some(n) = c.number {
                let (r, g, b) = STANDARD_PALETTE[n as usize];
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            } else {
                String::new()
            }
        }
        ColorType::EightBit => {
            if let Some(n) = c.number {
                let [r, g, b] = EIGHT_BIT_PALETTE[n as usize];
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            } else {
                String::new()
            }
        }
        ColorType::TrueColor => {
            if let Some((r, g, b)) = c.triplet {
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            } else {
                String::new()
            }
        }
    }
}

/// Return the static color name for a Standard color, or `None` otherwise.
fn color_to_name(c: &Color) -> Option<&'static str> {
    match c.color_type {
        ColorType::Standard => {
            if let Some(n) = c.number {
                Some(STANDARD_COLOR_NAMES[n as usize])
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// StyleStack — a stack of styles (for nested markup)
// ---------------------------------------------------------------------------

/// A stack of styles, used when rendering nested markup.
///
/// Tracks tag names alongside styles to support proper close-tag matching
/// (e.g., `[/bold]` inside `[italic][bold]...` correctly pops to bold,
/// not just the top of the stack).
#[derive(Debug, Clone)]
pub struct StyleStack {
    stack: Vec<Style>,
    /// Tag names corresponding to each pushed style. Used by `pop_to()`
    /// to find and remove the matching opening tag.
    tag_names: Vec<String>,
    default_style: Style,
}

impl StyleStack {
    /// Create a new style stack with a given default style.
    pub fn new(default_style: Style) -> Self {
        Self {
            stack: Vec::new(),
            tag_names: Vec::new(),
            default_style,
        }
    }

    /// Get the current (combined) style.
    pub fn current(&self) -> Style {
        let mut combined = self.default_style.clone();
        for s in &self.stack {
            combined = combined.combine(s);
        }
        combined
    }

    /// Push a style onto the stack (backward-compatible, no tag name).
    pub fn push(&mut self, style: Style) {
        self.tag_names.push(String::new());
        self.stack.push(style);
    }

    /// Push a style with an associated tag name for close-tag matching.
    pub fn push_named(&mut self, name: String, style: Style) {
        self.tag_names.push(name);
        self.stack.push(style);
    }

    /// Pop the top style (and its tag name).
    pub fn pop(&mut self) -> Option<Style> {
        self.tag_names.pop();
        self.stack.pop()
    }

    /// Pop styles until the matching opening tag is found and removed.
    ///
    /// Searches from the top of the stack for a tag with the given name.
    /// If found, removes it and everything above it. If not found, pops
    /// just one style as a fallback.
    pub fn pop_to(&mut self, name: &str) {
        if let Some(pos) = self.tag_names.iter().rposition(|n| n == name) {
            self.stack.truncate(pos);
            self.tag_names.truncate(pos);
        } else {
            // Tag not found — pop one as fallback
            self.stack.pop();
            self.tag_names.pop();
        }
    }

    /// Get the depth.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if the stack is empty (no pushed styles).
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_parse() {
        let s = Style::from_str("bold red");
        assert_eq!(s.get_bold(), Some(true));
        assert!(s.color.is_some());
    }

    #[test]
    fn test_style_combine() {
        let base = Style::from_str("red");
        let over = Style::from_str("bold");
        let combined = base.combine(&over);
        assert_eq!(combined.get_bold(), Some(true));
        assert!(combined.color.is_some());
    }

    #[test]
    fn test_ansi_output() {
        let s = Style::new().color(Color::parse("red").unwrap()).bold(true);
        let ansi = s.to_ansi();
        assert!(ansi.contains("31")); // red foreground
        assert!(ansi.contains("1"));  // bold
    }

    #[test]
    fn test_chain() {
        let a = Style::new().bold(true);
        let b = Style::new().color(Color::parse("red").unwrap()).italic(true);
        let chained = a.chain(&b);
        assert_eq!(chained.get_bold(), Some(true));
        assert!(chained.attributes.get(Attributes::ITALIC));
        assert!(chained.set_attributes & Attributes::ITALIC != 0);
        assert!(chained.color.is_some());
    }

    #[test]
    fn test_chain_precedence() {
        let a = Style::new().bold(true).color(Color::parse("red").unwrap());
        let b = Style::new().bold(false).color(Color::parse("blue").unwrap());
        let chained = a.chain(&b);
        // a sets bold(true) and color(red); b sets bold(false) and color(blue)
        // chain: self's values take priority
        assert_eq!(chained.get_bold(), Some(true));
        let c = chained.color.as_ref().unwrap();
        let name = color_to_name(c);
        assert_eq!(name, Some("red"));
    }

    #[test]
    fn test_copy() {
        let s = Style::new().bold(true).color(Color::parse("red").unwrap());
        let c = s.copy();
        assert_eq!(s, c);
    }

    #[test]
    fn test_clear_meta_and_links() {
        let mut s = Style::new().link("https://example.com");
        s.meta = Some(vec![1, 2, 3]);
        s.clear_meta_and_links();
        assert!(s.link.is_none());
        assert!(s.meta.is_none());
    }

    #[test]
    fn test_from_color() {
        let s = Style::from_color(Color::parse("red").unwrap());
        assert!(s.color.is_some());
        assert!(s.bgcolor.is_none());
    }

    #[test]
    fn test_from_meta() {
        let s = Style::from_meta(vec![10, 20, 30]);
        assert_eq!(s.meta(), Some(&vec![10, 20, 30]));
    }

    #[test]
    fn test_get_html_style() {
        let s = Style::new()
            .color(Color::parse("red").unwrap())
            .bold(true)
            .italic(true);
        let css = s.get_html_style(None);
        assert!(css.contains("color:"));
        assert!(css.contains("font-weight: bold"));
        assert!(css.contains("font-style: italic"));
    }

    #[test]
    fn test_get_html_style_underline_strike() {
        let s = Style::new()
            .color(Color::parse("red").unwrap())
            .underline(true)
            .strike(true);
        let css = s.get_html_style(None);
        assert!(css.contains("text-decoration:"));
        assert!(css.contains("underline"));
        assert!(css.contains("line-through"));
    }

    #[test]
    fn test_get_html_style_null() {
        let s = Style::null();
        let css = s.get_html_style(None);
        assert!(css.is_empty());
    }

    #[test]
    fn test_normalize() {
        let s = Style::new().bold(true).italic(false);
        let n = s.normalize();
        assert_eq!(n.get_bold(), Some(true));
        // italic was set to false, so normalize should remove it
        assert!(n.set_attributes & Attributes::ITALIC == 0);
    }

    #[test]
    fn test_pick_first() {
        let s = Style::new().color(Color::parse("red").unwrap());
        assert_eq!(s.pick_first(), Some("red"));
    }

    #[test]
    fn test_pick_first_fallback() {
        let s = Style::new().bgcolor(Color::parse("blue").unwrap());
        assert_eq!(s.pick_first(), Some("blue"));
    }

    #[test]
    fn test_pick_first_none() {
        let s = Style::new();
        assert_eq!(s.pick_first(), None);
    }

    #[test]
    fn test_render() {
        let s = Style::new().bold(true).color(Color::parse("red").unwrap());
        let rendered = s.render("hello");
        assert!(rendered.starts_with("\x1b["));
        assert!(rendered.contains("hello"));
        assert!(rendered.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_test_with_text() {
        let s = Style::new().bold(true);
        let out = s.test(Some("custom"));
        assert!(out.contains("custom"));
    }

    #[test]
    fn test_test_default() {
        let s = Style::new().bold(true);
        let out = s.test(None);
        assert!(out.contains("Lorem ipsum"));
    }

    #[test]
    fn test_update_link() {
        let mut s = Style::new();
        s.update_link(Some("https://example.com".into()));
        assert!(s.link.is_some());
        let first_id = s.link_id;
        s.update_link(None);
        assert!(s.link.is_none());
        assert_eq!(s.link_id, first_id);
    }

    #[test]
    fn test_link_id() {
        let s = Style::new().link("https://example.com");
        assert!(s.link_id() > 0);
    }

    #[test]
    fn test_meta_methods() {
        let mut s = Style::new();
        s.set_meta(Some(vec![1, 2, 3]));
        assert_eq!(s.meta(), Some(&vec![1, 2, 3]));
        if let Some(m) = s.meta_mut() {
            m.push(4);
        }
        assert_eq!(s.meta(), Some(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_on() {
        let s = Style::new().on(Color::parse("red").unwrap());
        assert!(s.bgcolor.is_some());
        let b = Color::parse("red").unwrap();
        assert_eq!(s.bgcolor.unwrap(), b);
    }

    #[test]
    fn test_references() {
        let s = Style::new()
            .color(Color::parse("red").unwrap())
            .bgcolor(Color::parse("blue").unwrap());
        assert!(s.color_ref().is_some());
        assert!(s.bgcolor_ref().is_some());
    }

    #[test]
    fn test_color_to_css_hex() {
        let c = Color::parse("red").unwrap();
        let hex = color_to_css_hex(&c);
        assert_eq!(hex, "#800000"); // standard red
    }

    #[test]
    fn test_color_to_css_hex_truecolor() {
        let c = Color::from_rgb(255, 0, 128);
        let hex = color_to_css_hex(&c);
        assert_eq!(hex, "#ff0080");
    }

    #[test]
    fn test_null_style_constant() {
        assert!(NULL_STYLE.is_null());
        assert!(NULL_STYLE.color.is_none());
        assert!(NULL_STYLE.bgcolor.is_none());
        assert_eq!(NULL_STYLE.set_attributes, 0);
        // Style::null() should equal NULL_STYLE
        assert_eq!(Style::null(), NULL_STYLE);
    }

    #[test]
    fn test_get_html_style_blink() {
        let s = Style::new().blink(true);
        let css = s.get_html_style(None);
        assert!(css.contains("blink"));
    }

    #[test]
    fn test_get_html_style_reverse() {
        let s = Style::new().reverse(true);
        let css = s.get_html_style(None);
        assert!(css.contains("invert(100%)"));
    }

    #[test]
    fn test_get_html_style_conceal() {
        let s = Style::new().conceal(true);
        let css = s.get_html_style(None);
        assert!(css.contains("visibility: hidden"));
    }

    #[test]
    fn test_static_attributes() {
        assert!(!STYLE_ATTRIBUTES.is_empty());
        let names: Vec<&str> = STYLE_ATTRIBUTES.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"bold"));
        assert!(names.contains(&"italic"));
        assert!(names.contains(&"underline"));
        assert!(!names.contains(&"notexist"));
    }

    // -----------------------------------------------------------------------
    // "not" prefix / negation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_not_bold_with_space() {
        // 'not' prefix negates the next token
        let s = Style::from_str("not bold");
        assert_eq!(s.get_bold(), Some(false));
    }

    #[test]
    fn test_not_italic_with_space() {
        let s = Style::from_str("not italic");
        assert_eq!(s.get_italic(), Some(false));
    }

    #[test]
    fn test_not_underline_with_space() {
        let s = Style::from_str("not underline");
        // Verify the bit is set and value is false (no get_underline accessor)
        assert!(s.set_attributes & Attributes::UNDERLINE != 0);
        assert!(!s.attributes.get(Attributes::UNDERLINE));
    }

    #[test]
    fn test_bang_negation_bold() {
        let s = Style::from_str("!bold");
        assert_eq!(s.get_bold(), Some(false));
    }

    #[test]
    fn test_nobold_prefix() {
        let s = Style::from_str("nobold");
        assert_eq!(s.get_bold(), Some(false));
    }

    #[test]
    fn test_not_bold_red() {
        // "not bold red" should disable bold and set red color
        let s = Style::from_str("not bold red");
        assert_eq!(s.get_bold(), Some(false));
        assert!(s.color.is_some());
    }

    #[test]
    fn test_not_multiple() {
        // "not bold not italic" should disable both
        let s = Style::from_str("not bold not italic");
        assert_eq!(s.get_bold(), Some(false));
        assert_eq!(s.get_italic(), Some(false));
    }

    #[test]
    fn test_on_next_color() {
        // "on red" sets background color to red
        let s = Style::from_str("on red");
        assert!(s.bgcolor.is_some());
    }
}
