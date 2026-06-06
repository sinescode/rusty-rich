//! Text wrapping utilities — equivalent to Rich's `_wrap.py`.
//!
//! Provides word-boundary-aware text splitting for line wrapping. Used by
//! [`Text::wrap`](crate::Text::wrap) and the console rendering pipeline to
//! break long lines at natural word boundaries rather than mid-word.
//!
//! # Example
//!
//! ```rust
//! use rusty_rich::wrap::{words, divide_line};
//!
//! let text = "Hello world, how are you?";
//! let words: Vec<_> = words(text).collect();
//! assert_eq!(words.len(), 4);
//!
//! let breaks = divide_line(text, 10, true);
//! // Text broken to fit in 10 columns
//! ```

use regex::Regex;
use std::sync::LazyLock;

use crate::cells::cell_len;

// ---------------------------------------------------------------------------
// Word splitting
// ---------------------------------------------------------------------------

/// A word extracted from text along with its position and display width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// The word text (including trailing whitespace).
    pub text: String,
    /// Byte offset of the word start in the original text.
    pub start: usize,
    /// Byte offset of the word end in the original text.
    pub end: usize,
    /// Unicode display width of the word.
    pub width: usize,
}

/// Compiled regex matching a word: non-whitespace followed by optional whitespace.
static WORD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s*\S+\s*").expect("invalid word regex"));

/// Yield each word from the text.
///
/// A "word" in this context includes the actual word and any whitespace
/// to the right. This matches Python Rich's `_wrap.words()` behavior.
///
/// Returns an iterator of [`Word`] values, each containing the word text,
/// byte offsets, and display width.
pub fn words(text: &str) -> impl Iterator<Item = Word> + '_ {
    let mut position = 0;
    std::iter::from_fn(move || {
        let m = WORD_RE.find_at(text, position)?;
        let word_text = m.as_str();
        let word_width = cell_len(word_text);
        let start = m.start();
        let end = m.end();
        position = end;
        Some(Word {
            text: word_text.to_string(),
            start,
            end,
            width: word_width,
        })
    })
}

// ---------------------------------------------------------------------------
// Line division
// ---------------------------------------------------------------------------

/// Compute break positions for splitting a line to fit within a given width.
///
/// Given a string of text and a width (measured in cells), returns a list of
/// byte offsets at which the string should be split in order to fit within
/// the given width.
///
/// When `fold` is true, words longer than `width` will be broken across
/// multiple lines (character-level folding). When false, such words are
/// cropped at the boundary.
///
/// This matches Python Rich's `_wrap.divide_line()` behavior.
pub fn divide_line(text: &str, width: usize, fold: bool) -> Vec<usize> {
    let mut break_positions: Vec<usize> = Vec::new();
    let mut cell_offset: usize = 0;

    for word in words(text) {
        let word_len = cell_len(word.text.trim_end());
        let remaining = width.saturating_sub(cell_offset);

        if word_len <= remaining {
            // Word fits on current line — advance the cell offset
            cell_offset += word.width;
        } else if word_len > width {
            // Word is wider than any line
            if fold {
                // Fold: break the word character-by-character
                let mut char_offset = word.start;
                let mut char_cell_offset: usize = 0;
                for ch in word.text.chars() {
                    let ch_width = cell_len(ch.encode_utf8(&mut [0u8; 4]));
                    if char_cell_offset + ch_width > width && char_cell_offset > 0 {
                        if char_offset > 0 {
                            break_positions.push(char_offset);
                        }
                        char_cell_offset = 0;
                    }
                    char_cell_offset += ch_width;
                    char_offset += ch.len_utf8();
                }
                cell_offset = char_cell_offset;
            } else {
                // Crop: break before the oversized word
                if word.start > 0 {
                    break_positions.push(word.start);
                }
                cell_offset = word.width;
            }
        } else {
            // Word doesn't fit on current line but fits on next line
            if cell_offset > 0 && word.start > 0 {
                break_positions.push(word.start);
            }
            cell_offset = word.width;
        }
    }

    break_positions
}

/// A convenience version that always enables folding (the most common case).
pub fn divide_line_fold(text: &str, width: usize) -> Vec<usize> {
    divide_line(text, width, true)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_words_basic() {
        let text = "Hello world, how are you?";
        let ws: Vec<Word> = words(text).collect();
        assert_eq!(ws.len(), 4);
        assert_eq!(ws[0].text, "Hello ");
        assert_eq!(ws[1].text, "world, ");
        assert_eq!(ws[2].text, "how ");
        assert_eq!(ws[3].text, "are you?");
    }

    #[test]
    fn test_words_single_word() {
        let ws: Vec<Word> = words("hello").collect();
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].text, "hello");
    }

    #[test]
    fn test_words_empty() {
        let ws: Vec<Word> = words("").collect();
        assert_eq!(ws.len(), 0);
    }

    #[test]
    fn test_words_leading_whitespace() {
        let ws: Vec<Word> = words("   hello world").collect();
        assert_eq!(ws.len(), 2);
        assert_eq!(ws[0].text, "   hello ");
    }

    #[test]
    fn test_divide_line_no_wrap_needed() {
        let text = "short";
        let breaks = divide_line(text, 80, true);
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_divide_line_basic() {
        let text = "one two three four";
        // Force width so "three four" wraps
        let breaks = divide_line(text, 14, true);
        // "one two " is 8 chars, "three " is 7 → fits in 14, "four" wraps
        // Actually let's calculate: "one two three " = 15 cells > 14
        // So break before "three": "one two " = 8 cells, break, "three four" on next
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_divide_line_long_word_fold() {
        let text = "abc defghijklmnop";
        let breaks = divide_line(text, 5, true);
        // "abc " fits (4), "defghijklmnop" is 14 cells > 5, so fold it
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_divide_line_long_word_no_fold() {
        let text = "abc supercalifragilistic";
        let breaks = divide_line(text, 5, false);
        // "abc " fits, then we break before the long word (crop)
        assert!(!breaks.is_empty());
    }

    #[test]
    fn test_divide_line_exact_fit() {
        let text = "12345 67890";
        // "12345 " = 6, "67890" = 5, total = 11
        let breaks = divide_line(text, 11, true);
        assert!(breaks.is_empty());
    }

    #[test]
    fn test_divide_line_with_cjk() {
        let text = "Hello 世界 World";
        let breaks = divide_line(text, 10, true);
        // Should handle CJK characters with wider cell widths
        let _ = breaks; // at minimum, should not panic
    }
}
