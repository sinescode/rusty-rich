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
///
/// Handles standard colors (30-37, 40-47, 90-97, 100-107), attributes (1-9,
/// 21-29, 51-55), and extended colors (38 for fg, 48 for bg) in both 8-bit
/// (38;5;N) and TrueColor (38;2;R;G;B) forms.
fn apply_sgr(style: &Style, params: &str) -> Style {
    if params.is_empty() || params == "0" {
        return Style::new(); // Reset
    }

    let mut s = style.clone();
    let parts: Vec<&str> = params.split(';').collect();
    let mut i = 0usize;

    while i < parts.len() {
        let n = parts[i].parse::<u32>().unwrap_or(999);
        match n {
            0 => s = Style::new(), // Reset
            1 => s = s.bold(true),
            2 => s = s.dim(true),
            3 => s = s.italic(true),
            4 => s = s.underline(true),
            5 => s = s.blink(true),
            6 => s = s.blink2(true),
            7 => s = s.reverse(true),
            8 => s = s.conceal(true),
            9 => s = s.strike(true),
            21 => s = s.underline2(true),
            22 => { s = s.bold(false); s = s.dim(false); }
            23 => s = s.italic(false),
            24 => s = s.underline(false),
            25 => { s = s.blink(false); s = s.blink2(false); }
            27 => s = s.reverse(false),
            28 => s = s.conceal(false),
            29 => s = s.strike(false),
            30..=37 => {
                let idx = n - 30;
                s = s.color(crate::color::Color::from_8bit(idx as u8));
            }
            38 => {
                // Extended foreground color
                i += 1;
                if i < parts.len() {
                    match parts[i] {
                        "5" => {
                            // 8-bit color: 38;5;N
                            i += 1;
                            if i < parts.len() {
                                if let Ok(n) = parts[i].parse::<u8>() {
                                    s = s.color(crate::color::Color::from_8bit(n));
                                }
                            }
                        }
                        "2" => {
                            // TrueColor: 38;2;R;G;B
                            if i + 3 < parts.len() {
                                let r = parts[i + 1].parse::<u8>().unwrap_or(0);
                                let g = parts[i + 2].parse::<u8>().unwrap_or(0);
                                let b = parts[i + 3].parse::<u8>().unwrap_or(0);
                                s = s.color(crate::color::Color::from_rgb(r, g, b));
                                i += 3;
                            }
                        }
                        _ => {}
                    }
                }
            }
            39 => { s = s.color(crate::color::Color::default()); }
            40..=47 => {
                let idx = n - 40;
                s = s.bgcolor(crate::color::Color::from_8bit(idx as u8));
            }
            48 => {
                // Extended background color
                i += 1;
                if i < parts.len() {
                    match parts[i] {
                        "5" => {
                            // 8-bit color: 48;5;N
                            i += 1;
                            if i < parts.len() {
                                if let Ok(n) = parts[i].parse::<u8>() {
                                    s = s.bgcolor(crate::color::Color::from_8bit(n));
                                }
                            }
                        }
                        "2" => {
                            // TrueColor: 48;2;R;G;B
                            if i + 3 < parts.len() {
                                let r = parts[i + 1].parse::<u8>().unwrap_or(0);
                                let g = parts[i + 2].parse::<u8>().unwrap_or(0);
                                let b = parts[i + 3].parse::<u8>().unwrap_or(0);
                                s = s.bgcolor(crate::color::Color::from_rgb(r, g, b));
                                i += 3;
                            }
                        }
                        _ => {}
                    }
                }
            }
            49 => { s = s.bgcolor(crate::color::Color::default()); }
            51 => s = s.frame(true),
            52 => s = s.encircle(true),
            53 => s = s.overline(true),
            54 => { s = s.frame(false); s = s.encircle(false); }
            55 => s = s.overline(false),
            90..=97 => {
                let idx = n - 90 + 8; // bright colors start at 8
                s = s.color(crate::color::Color::from_8bit(idx as u8));
            }
            100..=107 => {
                let idx = n - 100 + 8;
                s = s.bgcolor(crate::color::Color::from_8bit(idx as u8));
            }
            _ => {}
        }
        i += 1;
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
