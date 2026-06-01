//! Text style — equivalent to Rich's `style.py`.
//!
//! A Style combines a foreground color, background color, and a set of
//! boolean text attributes (bold, italic, underline, etc.).

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::color::Color;

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// Style attributes — bit flags
// ---------------------------------------------------------------------------

/// Bit flags for text attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attributes(u32);

impl Attributes {
    pub const BOLD: u32 = 1 << 0;
    pub const DIM: u32 = 1 << 1;
    pub const ITALIC: u32 = 1 << 2;
    pub const UNDERLINE: u32 = 1 << 3;
    pub const BLINK: u32 = 1 << 4;
    pub const REVERSE: u32 = 1 << 5;
    pub const STRIKE: u32 = 1 << 6;
    pub const UNDERLINE2: u32 = 1 << 7;
    pub const FRAME: u32 = 1 << 8;
    pub const ENCIRCLE: u32 = 1 << 9;
    pub const OVERLINE: u32 = 1 << 10;
    pub const BLINK2: u32 = 1 << 11;
    pub const CONCEAL: u32 = 1 << 12;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn set(&mut self, bit: u32, value: bool) {
        if value {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }

    pub fn get(&self, bit: u32) -> bool {
        self.0 & bit != 0
    }

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
    pub fn null() -> Self {
        Self {
            color: None,
            bgcolor: None,
            attributes: Attributes::empty(),
            set_attributes: 0,
            link: None,
            link_id: 0,
            is_null: true,
            meta: None,
        }
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
    pub fn from_str(definition: &str) -> Self {
        let mut style = Self::new();
        for part in definition.split_whitespace() {
            match part {
                "bold" | "b" => { style.set_attributes |= Attributes::BOLD; style.attributes.set(Attributes::BOLD, true); }
                "dim" | "d" => { style.set_attributes |= Attributes::DIM; style.attributes.set(Attributes::DIM, true); }
                "italic" | "i" => { style.set_attributes |= Attributes::ITALIC; style.attributes.set(Attributes::ITALIC, true); }
                "underline" | "u" => { style.set_attributes |= Attributes::UNDERLINE; style.attributes.set(Attributes::UNDERLINE, true); }
                "blink" => { style.set_attributes |= Attributes::BLINK; style.attributes.set(Attributes::BLINK, true); }
                "reverse" | "r" => { style.set_attributes |= Attributes::REVERSE; style.attributes.set(Attributes::REVERSE, true); }
                "strike" | "s" => { style.set_attributes |= Attributes::STRIKE; style.attributes.set(Attributes::STRIKE, true); }
                "not bold" | "!bold" | "nobold" => { style.set_attributes |= Attributes::BOLD; style.attributes.set(Attributes::BOLD, false); }
                "not italic" | "!italic" | "noitalic" => { style.set_attributes |= Attributes::ITALIC; style.attributes.set(Attributes::ITALIC, false); }
                "not underline" | "!underline" | "nounderline" => { style.set_attributes |= Attributes::UNDERLINE; style.attributes.set(Attributes::UNDERLINE, false); }
                "none" | "default" => {}
                "on" => { /* "on <color>" handled below */ }
                part if part.starts_with("on ") => {
                    if let Ok(c) = Color::parse(&part[3..]) {
                        style.bgcolor = Some(c);
                    }
                }
                part if part.starts_with("link=") => {
                    style.link = Some(part[5..].to_string());
                }
                part => {
                    // Try as color name
                    if let Ok(c) = Color::parse(part) {
                        if style.bgcolor.is_some() && style.color.is_none() {
                            // We already saw "on" — don't overwrite fg
                        } else {
                            style.color = Some(c);
                        }
                    }
                }
            }
        }
        style
    }

    // -- queries ------------------------------------------------------------

    pub fn is_null(&self) -> bool {
        self.is_null
    }

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
    pub fn to_ansi(&self) -> String {
        if self.is_null {
            return String::new();
        }
        let mut codes: Vec<String> = Vec::new();

        // Foreground color
        if let Some(ref c) = self.color {
            match c.color_type {
                crate::color::ColorType::Default => codes.push("39".into()),
                crate::color::ColorType::Standard => {
                    if let Some(n) = c.number {
                        if n < 8 {
                            codes.push((30 + n).to_string());
                        } else {
                            codes.push((82 + n).to_string()); // 90-97 for bright
                        }
                    }
                }
                crate::color::ColorType::EightBit => {
                    if let Some(n) = c.number {
                        codes.push(format!("38;5;{n}"));
                    }
                }
                crate::color::ColorType::TrueColor => {
                    if let Some((r, g, b)) = c.triplet {
                        codes.push(format!("38;2;{r};{g};{b}"));
                    }
                }
            }
        }

        // Background color
        if let Some(ref c) = self.bgcolor {
            match c.color_type {
                crate::color::ColorType::Default => codes.push("49".into()),
                crate::color::ColorType::Standard => {
                    if let Some(n) = c.number {
                        if n < 8 {
                            codes.push((40 + n).to_string());
                        } else {
                            codes.push((92 + n).to_string()); // 100-107
                        }
                    }
                }
                crate::color::ColorType::EightBit => {
                    if let Some(n) = c.number {
                        codes.push(format!("48;5;{n}"));
                    }
                }
                crate::color::ColorType::TrueColor => {
                    if let Some((r, g, b)) = c.triplet {
                        codes.push(format!("48;2;{r};{g};{b}"));
                    }
                }
            }
        }

        // Attributes
        if self.set_attributes & Attributes::BOLD != 0 {
            codes.push(if self.attributes.get(Attributes::BOLD) { "1" } else { "22" }.into());
        }
        if self.set_attributes & Attributes::DIM != 0 {
            codes.push(if self.attributes.get(Attributes::DIM) { "2" } else { "22" }.into());
        }
        if self.set_attributes & Attributes::ITALIC != 0 {
            codes.push(if self.attributes.get(Attributes::ITALIC) { "3" } else { "23" }.into());
        }
        if self.set_attributes & Attributes::UNDERLINE != 0 {
            codes.push(if self.attributes.get(Attributes::UNDERLINE) { "4" } else { "24" }.into());
        }
        if self.set_attributes & Attributes::BLINK != 0 {
            codes.push(if self.attributes.get(Attributes::BLINK) { "5" } else { "25" }.into());
        }
        if self.set_attributes & Attributes::REVERSE != 0 {
            codes.push(if self.attributes.get(Attributes::REVERSE) { "7" } else { "27" }.into());
        }
        if self.set_attributes & Attributes::CONCEAL != 0 {
            codes.push(if self.attributes.get(Attributes::CONCEAL) { "8" } else { "28" }.into());
        }
        if self.set_attributes & Attributes::STRIKE != 0 {
            codes.push(if self.attributes.get(Attributes::STRIKE) { "9" } else { "29" }.into());
        }
        if self.set_attributes & Attributes::CONCEAL != 0 {
            codes.push(if self.attributes.get(Attributes::CONCEAL) { "8" } else { "28" }.into());
        }
        if self.set_attributes & Attributes::UNDERLINE2 != 0 {
            codes.push(if self.attributes.get(Attributes::UNDERLINE2) { "21" } else { "24" }.into());
        }
        if self.set_attributes & Attributes::BLINK2 != 0 {
            codes.push(if self.attributes.get(Attributes::BLINK2) { "6" } else { "25" }.into());
        }
        if self.set_attributes & Attributes::FRAME != 0 {
            codes.push(if self.attributes.get(Attributes::FRAME) { "51" } else { "54" }.into());
        }
        if self.set_attributes & Attributes::ENCIRCLE != 0 {
            codes.push(if self.attributes.get(Attributes::ENCIRCLE) { "52" } else { "54" }.into());
        }
        if self.set_attributes & Attributes::OVERLINE != 0 {
            codes.push(if self.attributes.get(Attributes::OVERLINE) { "53" } else { "55" }.into());
        }

        if codes.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", codes.join(";"))
        }
    }

    /// Return the ANSI reset sequence needed to turn off this style.
    pub fn reset_ansi(&self) -> &'static str {
        "\x1b[0m"
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

// ---------------------------------------------------------------------------
// StyleStack — a stack of styles (for nested markup)
// ---------------------------------------------------------------------------

/// A stack of styles, used when rendering nested markup.
#[derive(Debug, Clone)]
pub struct StyleStack {
    stack: Vec<Style>,
    default_style: Style,
}

impl StyleStack {
    pub fn new(default_style: Style) -> Self {
        Self {
            stack: Vec::new(),
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

    /// Push a style onto the stack.
    pub fn push(&mut self, style: Style) {
        self.stack.push(style);
    }

    /// Pop the top style.
    pub fn pop(&mut self) -> Option<Style> {
        self.stack.pop()
    }

    /// Get the depth.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

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
}
