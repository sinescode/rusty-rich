//! Console markup parser — equivalent to Rich's `markup.py`.
//!
//! Supports Rich's BBCode-like markup syntax:
//!
//! - `[bold]text[/bold]` — apply bold
//! - `[red]text[/red]` — set color
//! - `[on blue]text[/on blue]` — set background
//! - `[bold red on blue]text[/]` — combined styling
//! - `[/]` — close all open tags
//! - `[[` — literal `[`

use crate::style::{Style, StyleStack};
use crate::text::Text;

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

/// A parsed markup tag.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub name: String,
    pub parameters: Option<String>,
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parameters: None,
        }
    }

    pub fn with_params(name: impl Into<String>, params: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parameters: Some(params.into()),
        }
    }

    /// Check if this is a closing tag (`/name` or `/`).
    pub fn is_closing(&self) -> bool {
        self.name == "/" || self.name.starts_with('/')
    }

    /// Get the name without the leading `/` for closing tags.
    pub fn closing_name(&self) -> &str {
        if self.name == "/" {
            ""
        } else {
            &self.name[1..]
        }
    }

    /// Get the markup string for this tag.
    pub fn markup(&self) -> String {
        if let Some(ref params) = self.parameters {
            format!("[{}={}]", self.name, params)
        } else {
            format!("[{}]", self.name)
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse markup and return a `Text` with applied styles.
pub fn render(markup: &str) -> Text {
    let mut text = Text::new("");
    let mut style_stack = StyleStack::new(Style::new());
    let mut pos = 0usize;

    let chars: Vec<char> = markup.chars().collect();
    let len = chars.len();

    while pos < len {
        if chars[pos] == '[' {
            // Check for escaped `[[`
            if pos + 1 < len && chars[pos + 1] == '[' {
                text.append_styled("[", style_stack.current());
                pos += 2;
                continue;
            }

            // Find the closing `]`
            let end = match chars[pos..].iter().position(|&c| c == ']') {
                Some(e) => pos + e,
                None => {
                    // No closing bracket — treat as literal
                    text.append_styled("[", style_stack.current());
                    pos += 1;
                    continue;
                }
            };

            let tag_str: String = chars[pos + 1..end].iter().collect();
            pos = end + 1;

            if tag_str.is_empty() {
                continue;
            }

            // Parse the tag
            let tag = parse_tag(&tag_str);

            if tag.is_closing() {
                let closing = tag.closing_name();
                if closing.is_empty() {
                    // [/] — close all
                    while style_stack.len() > 0 {
                        style_stack.pop();
                    }
                } else {
                    // [/name] — pop until we find matching
                    // Simplified: just pop one
                    style_stack.pop();
                }
            } else {
                // Opening tag — push style
                let style = tag_to_style(&tag);
                style_stack.push(style);
            }
        } else {
            // Regular text — accumulate until next `[` or end
            let start = pos;
            while pos < len && chars[pos] != '[' {
                pos += 1;
            }
            let chunk: String = chars[start..pos].iter().collect();
            text.append_styled(chunk, style_stack.current());
        }
    }

    text
}

/// Parse a tag string into a Tag.
fn parse_tag(s: &str) -> Tag {
    // Handle "/" or "/name"
    if s.starts_with('/') {
        return Tag::new(s.to_string());
    }

    // Check for `name=value`
    if let Some(eq) = s.find('=') {
        let name = s[..eq].to_string();
        let value = s[eq + 1..].to_string();
        // Strip quotes if present
        let value = value.trim_matches('"').trim_matches('\'').to_string();
        return Tag::with_params(name, value);
    }

    // Check for `name(params)`
    if let Some(lparen) = s.find('(') {
        if s.ends_with(')') {
            let name = s[..lparen].to_string();
            let params = s[lparen + 1..s.len() - 1].to_string();
            return Tag::with_params(name, params);
        }
    }

    Tag::new(s.to_string())
}

/// Convert a tag to a Style.
fn tag_to_style(tag: &Tag) -> Style {
    let name = &tag.name;

    match name.as_str() {
        "bold" | "b" => Style::new().bold(true),
        "dim" | "d" => Style::new().dim(true),
        "italic" | "i" => Style::new().italic(true),
        "underline" | "u" => Style::new().underline(true),
        "blink" => Style::new().blink(true),
        "reverse" | "r" => Style::new().reverse(true),
        "strike" | "s" => Style::new().strike(true),

        "/bold" | "/b" | "/dim" | "/d" | "/italic" | "/i"
        | "/underline" | "/u" | "/blink" | "/reverse" | "/r"
        | "/strike" | "/s" => Style::null(),

        _ => {
            // Try as color name, including "on <color>"
            if name.starts_with("on ") {
                if let Ok(c) = crate::color::Color::parse(&name[3..]) {
                    return Style::new().bgcolor(c);
                }
            }

            // Try "color on bgcolor"
            if let Some(on_pos) = name.find(" on ") {
                let fg_name = &name[..on_pos];
                let bg_name = &name[on_pos + 4..];
                if let Ok(fg) = crate::color::Color::parse(fg_name) {
                    let mut style = Style::new().color(fg);
                    if let Ok(bg) = crate::color::Color::parse(bg_name) {
                        style = style.bgcolor(bg);
                    }
                    return style;
                }
            }

            // Try as a plain color name
            if let Ok(c) = crate::color::Color::parse(name) {
                return Style::new().color(c);
            }

            // Try from parameters (e.g. [color(1)] or [color=red])
            if let Some(ref params) = tag.parameters {
                if let Ok(c) = crate::color::Color::parse(params) {
                    return Style::new().color(c);
                }
            }

            // Unknown tag — return empty style
            Style::new()
        }
    }
}

// ---------------------------------------------------------------------------
// Escape markup
// ---------------------------------------------------------------------------

/// Escape text so it won't be interpreted as markup.
pub fn escape(markup: &str) -> String {
    markup.replace('[', "[[")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(escape("[bold]"), "[[bold]");
    }

    #[test]
    fn test_render_bold() {
        let t = render("[bold]Hello[/bold]");
        assert_eq!(t.plain, "Hello");
        assert!(!t.spans.is_empty()); // has style spans
    }

    #[test]
    fn test_render_literal_bracket() {
        let t = render("[[hello]]");
        // Escaped brackets produce the bracket followed by literal text then closing bracket
        assert!(t.plain.contains("hello"));
    }

    #[test]
    fn test_render_color() {
        let t = render("[red]red text[/red]");
        assert_eq!(t.plain, "red text");
        assert!(!t.spans.is_empty());
    }

    #[test]
    fn test_parse_tag() {
        let tag = parse_tag("bold");
        assert_eq!(tag.name, "bold");

        let tag = parse_tag("color=red");
        assert_eq!(tag.name, "color");
        assert_eq!(tag.parameters, Some("red".into()));

        let tag = parse_tag("/bold");
        assert!(tag.is_closing());
    }
}
