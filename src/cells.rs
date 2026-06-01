//! Unicode cell width handling — equivalent to Rich's `cells.py`.
//!
//! Provides correct cell-width measurement for CJK, emoji, and ZWJ sequences.

// ---------------------------------------------------------------------------
// cell_len — total Unicode display width of a string
// ---------------------------------------------------------------------------

/// Return the total cell width of `text`, handling double-width characters
/// and zero-width characters correctly.
pub fn cell_len(text: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(text)
}

// ---------------------------------------------------------------------------
// get_character_cell_size — width of a single character
// ---------------------------------------------------------------------------

/// Return the cell width of a single character (0, 1, or 2).
pub fn get_character_cell_size(ch: char) -> usize {
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// set_cell_size — pad or crop text to a target cell width
// ---------------------------------------------------------------------------

/// Pad or crop `text` so that its total cell width equals `target_width`.
/// If the text is too short, right-pad with spaces.
/// If it's too long, it is cropped (if a double-width character is split,
/// it is replaced with spaces).
pub fn set_cell_size(text: &str, target_width: usize) -> String {
    let current = cell_len(text);
    if current == target_width {
        return text.to_string();
    }
    if current > target_width {
        // Crop to target
        let mut out = String::new();
        let mut w = 0usize;
        for ch in text.chars() {
            let cw = get_character_cell_size(ch);
            if w + cw > target_width {
                // Fill remaining with spaces
                out.push_str(&" ".repeat(target_width - w));
                return out;
            }
            out.push(ch);
            w += cw;
        }
        out
    } else {
        // Pad with spaces
        format!("{}{}", text, " ".repeat(target_width - current))
    }
}

// ---------------------------------------------------------------------------
// chop_cells — break text into lines that fit within width cells
// ---------------------------------------------------------------------------

/// Break `text` into a list of strings, each of which fits within `width`
/// cells. Double-width characters are not split; if a character would span
/// the boundary, it starts the next line.
pub fn chop_cells(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_w = 0usize;

    for ch in text.chars() {
        let cw = get_character_cell_size(ch);

        if ch == '\n' {
            lines.push(current);
            current = String::new();
            current_w = 0;
            continue;
        }

        if current_w + cw > width {
            lines.push(current);
            current = String::new();
            current_w = 0;
        }

        current.push(ch);
        current_w += cw;
    }

    if !current.is_empty() {
        lines.push(current);
    }

    // If all lines are empty (text was empty), return one empty line
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

// ---------------------------------------------------------------------------
// split_text — split text at a given cell offset
// ---------------------------------------------------------------------------

/// Split text at the given cell offset, returning `(left, right)`.
/// If the offset falls in the middle of a double-width character, that
/// character is replaced with spaces in the left part and starts the right.
pub fn split_text(text: &str, cell_offset: usize) -> (String, String) {
    let mut left = String::new();
    let mut right = String::new();
    let mut w = 0usize;
    let mut passed = false;

    for ch in text.chars() {
        let cw = get_character_cell_size(ch);

        if passed {
            right.push(ch);
        } else if w + cw > cell_offset {
            // This character straddles the boundary
            // Fill remaining cell space on left with spaces
            left.push_str(&" ".repeat(cell_offset - w));
            right.push(ch);
            passed = true;
        } else {
            left.push(ch);
            w += cw;
            if w == cell_offset {
                passed = true;
            }
        }
    }

    (left, right)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_len_ascii() {
        assert_eq!(cell_len("hello"), 5);
    }

    #[test]
    fn test_cell_len_cjk() {
        assert_eq!(cell_len("你好"), 4); // 2 wide chars × 2 = 4
    }

    #[test]
    fn test_cell_len_emoji() {
        assert_eq!(cell_len("🎉"), 2);
    }

    #[test]
    fn test_set_cell_size_pad() {
        assert_eq!(set_cell_size("hi", 10), "hi        ");
    }

    #[test]
    fn test_set_cell_size_crop() {
        assert_eq!(set_cell_size("hello world", 5), "hello");
    }

    #[test]
    fn test_set_cell_size_exact() {
        assert_eq!(set_cell_size("hello", 5), "hello");
    }

    #[test]
    fn test_chop_cells_basic() {
        let lines = chop_cells("hello world", 5);
        assert_eq!(lines, vec!["hello", " worl", "d"]);
    }

    #[test]
    fn test_chop_cells_newline() {
        let lines = chop_cells("a\nb\nc", 10);
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_text_middle() {
        let (left, right) = split_text("hello world", 5);
        assert_eq!(left, "hello");
        assert_eq!(right, " world");
    }

    #[test]
    fn test_split_text_past_end() {
        let (left, right) = split_text("hi", 10);
        assert_eq!(left, "hi");
        assert_eq!(right, "");
    }
}
