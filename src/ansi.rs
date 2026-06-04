//! ANSI escape sequence decoder — parse ANSI text into styled Text.

use crate::style::Style;
use crate::text::Text;
use regex::Regex;

/// Decode ANSI-escaped text into styled Text components.
pub struct AnsiDecoder;

impl AnsiDecoder {
    /// Parse ANSI text and return styled Text.
    pub fn decode(ansi_text: &str) -> Text {
        let mut text = Text::new("");
        let mut current_style = Style::new();
        let mut last_end = 0usize;

        // Match ANSI SGR escape sequences
        let re = Regex::new(r"\x1b\[([\d;]*)m").unwrap();

        for caps in re.captures_iter(ansi_text) {
            let m = caps.get(0).unwrap();
            let start = m.start();

            // Add text before this escape code
            if start > last_end {
                let plain = &ansi_text[last_end..start];
                text.append_styled(plain, current_style.clone());
            }

            // Parse SGR parameters
            let params = caps.get(1).map_or("", |p| p.as_str());
            current_style = apply_sgr(&current_style, params);
            last_end = m.end();
        }

        // Add remaining text
        if last_end < ansi_text.len() {
            text.append_styled(&ansi_text[last_end..], current_style);
        }

        text
    }
}

/// Apply SGR parameters to a style.
fn apply_sgr(style: &Style, params: &str) -> Style {
    if params.is_empty() || params == "0" {
        return Style::new(); // Reset
    }

    let mut s = style.clone();
    for param in params.split(';') {
        if let Ok(n) = param.parse::<u32>() {
            match n {
                0 => s = Style::new(), // Reset
                1 => {
                    s = s.bold(true);
                } // Bold
                2 => {
                    s = s.dim(true);
                } // Dim
                3 => {
                    s = s.italic(true);
                } // Italic
                4 => {
                    s = s.underline(true);
                } // Underline
                5 => {
                    s = s.blink(true);
                } // Slow blink
                6 => {
                    s = s.blink2(true);
                } // Fast blink
                7 => {
                    s = s.reverse(true);
                } // Reverse
                8 => {
                    s = s.conceal(true);
                } // Conceal
                9 => {
                    s = s.strike(true);
                } // Strikethrough
                21 => {
                    s = s.underline2(true);
                } // Double underline
                22 => {
                    s = s.bold(false);
                } // Normal intensity
                23 => {
                    s = s.italic(false);
                } // Not italic
                24 => {
                    s = s.underline(false);
                } // Not underline
                25 => {
                    s = s.blink(false);
                } // Not blink
                27 => {
                    s = s.reverse(false);
                } // Not reverse
                28 => {
                    s = s.conceal(false);
                } // Not conceal
                29 => {
                    s = s.strike(false);
                } // Not strikethrough
                30..=37 => {
                    // Standard fg
                    if let Ok(c) = crate::color::Color::parse(&format!("color({})", n - 30)) {
                        s = s.color(c);
                    }
                }
                38 => { /* Extended fg - skip for simplicity */ }
                39 => {
                    s = s.color(crate::color::Color::default());
                } // Default fg
                40..=47 => {
                    // Standard bg
                    if let Ok(c) = crate::color::Color::parse(&format!("color({})", n - 40)) {
                        s = s.bgcolor(c);
                    }
                }
                48 => { /* Extended bg - skip */ }
                49 => {
                    s = s.bgcolor(crate::color::Color::default());
                } // Default bg
                90..=97 => {
                    // Bright fg
                    if let Ok(c) = crate::color::Color::parse(&format!("color({})", n - 90 + 8)) {
                        s = s.color(c);
                    }
                }
                100..=107 => {
                    // Bright bg
                    if let Ok(c) = crate::color::Color::parse(&format!("color({})", n - 100 + 8)) {
                        s = s.bgcolor(c);
                    }
                }
                _ => {}
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bold() {
        let text = AnsiDecoder::decode("\x1b[1mBold Text\x1b[0m");
        assert!(text.plain.contains("Bold Text"));
        assert!(!text.spans.is_empty());
    }

    #[test]
    fn test_decode_reset() {
        let text = AnsiDecoder::decode("\x1b[31mRed\x1b[0m Normal");
        assert!(text.plain.contains("Red"));
        assert!(text.plain.contains("Normal"));
    }
}
