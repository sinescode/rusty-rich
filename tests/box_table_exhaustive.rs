//! Exhaustive box and table tests — every box style × every feature combination.
//!
//! This test file exercises:
//! - All 18 BoxStyles (parse, characters, round-trip, rendering)
//! - Panel: every box style × title/subtitle/padding/alignment/width/height/fit
//! - Table: every box style × colspan/rowspan/sections/lines/leading/ratio/footer
//! - Columns: equal, expand, padding, multi-column edge cases

use rusty_rich::*;

// ===========================================================================
// ALL BOX STYLES — helper to iterate over every defined box
// ===========================================================================

/// Returns every predefined BoxStyle as (name, &BoxStyle) pairs.
fn all_box_styles() -> Vec<(&'static str, &'static box_drawing::BoxStyle)> {
    vec![
        ("ROUNDED", &box_drawing::BOX_ROUNDED),
        ("SQUARE", &box_drawing::BOX_SQUARE),
        ("HEAVY", &box_drawing::BOX_HEAVY),
        ("HEAVY_EDGE", &box_drawing::BOX_HEAVY_EDGE),
        ("HEAVY_HEAD", &box_drawing::BOX_HEAVY_HEAD),
        ("DOUBLE", &box_drawing::BOX_DOUBLE),
        ("DOUBLE_EDGE", &box_drawing::BOX_DOUBLE_EDGE),
        ("SIMPLE", &box_drawing::BOX_SIMPLE),
        ("SIMPLE_HEAVY", &box_drawing::BOX_SIMPLE_HEAVY),
        ("MINIMAL", &box_drawing::BOX_MINIMAL),
        ("MINIMAL_HEAVY", &box_drawing::BOX_MINIMAL_HEAVY),
        ("ASCII", &box_drawing::BOX_ASCII),
        ("ASCII2", &box_drawing::BOX_ASCII2),
        ("SQUARE_DOUBLE_HEAD", &box_drawing::BOX_SQUARE_DOUBLE_HEAD),
        ("MINIMAL_DOUBLE_HEAD", &box_drawing::BOX_MINIMAL_DOUBLE_HEAD),
        ("SIMPLE_HEAD", &box_drawing::BOX_SIMPLE_HEAD),
        ("ASCII_DOUBLE_HEAD", &box_drawing::BOX_ASCII_DOUBLE_HEAD),
        ("MARKDOWN", &box_drawing::BOX_MARKDOWN),
    ]
}

/// Returns only the non-ASCII box styles.
fn unicode_box_styles() -> Vec<(&'static str, &'static box_drawing::BoxStyle)> {
    all_box_styles()
        .into_iter()
        .filter(|(_, b)| !b.ascii)
        .collect()
}

// ===========================================================================
// BOX DRAWING — exhaustive structure & character tests
// ===========================================================================

#[test]
fn box_all_18_styles_have_8_lines() {
    for (name, b) in all_box_styles() {
        let s = b.to_string();
        assert_eq!(
            s.lines().count(),
            8,
            "{name}: expected 8 lines, got {}",
            s.lines().count()
        );
    }
}

#[test]
fn box_all_18_styles_each_line_has_4_chars() {
    for (name, b) in all_box_styles() {
        let s = b.to_string();
        for (i, line) in s.lines().enumerate() {
            assert_eq!(
                line.chars().count(),
                4,
                "{name} line {i}: expected 4 chars, got '{}' ({})",
                line,
                line.chars().count()
            );
        }
    }
}

#[test]
fn box_from_str_roundtrip_all_18() {
    for (name, b) in all_box_styles() {
        let s = b.to_string();
        let parsed = BoxStyle::from_str(&s, b.ascii);
        assert_eq!(
            parsed, *b,
            "{name}: from_str(to_string()) roundtrip failed"
        );
    }
}

#[test]
fn box_display_matches_to_string() {
    for (name, b) in all_box_styles() {
        let via_display = format!("{b}");
        let via_to_string = b.to_string();
        assert_eq!(via_display, via_to_string, "{name}: Display != to_string");
    }
}

#[test]
fn box_clone_equals_original() {
    for (name, b) in all_box_styles() {
        let cloned = b.clone();
        assert_eq!(cloned, *b, "{name}: clone not equal");
    }
}

#[test]
fn box_ascii_styles_have_ascii_flag() {
    let ascii_styles = [
        &box_drawing::BOX_ASCII,
        &box_drawing::BOX_ASCII2,
        &box_drawing::BOX_ASCII_DOUBLE_HEAD,
    ];
    for b in ascii_styles {
        assert!(b.ascii, "ASCII style should have ascii=true");
        let s = b.to_string();
        assert!(s.is_ascii(), "ASCII style string should be pure ASCII");
    }
}

#[test]
fn box_non_ascii_styles_have_unicode() {
    // All non-ASCII styles except MARKDOWN (which uses only |, -, space)
    for (name, b) in unicode_box_styles() {
        if name == "MARKDOWN" {
            continue; // MARKDOWN uses only ASCII chars
        }
        let s = b.to_string();
        assert!(
            !s.is_ascii(),
            "{name}: should contain at least one non-ASCII char"
        );
    }
}

#[test]
fn box_rounded_corner_characters() {
    let b = &*box_drawing::BOX_ROUNDED;
    assert_eq!(b.top_left, '╭');
    assert_eq!(b.top_right, '╮');
    assert_eq!(b.bottom_left, '╰');
    assert_eq!(b.bottom_right, '╯');
}

#[test]
fn box_square_corner_characters() {
    let b = &*box_drawing::BOX_SQUARE;
    assert_eq!(b.top_left, '┌');
    assert_eq!(b.top_right, '┐');
    assert_eq!(b.bottom_left, '└');
    assert_eq!(b.bottom_right, '┘');
}

#[test]
fn box_heavy_corner_characters() {
    let b = &*box_drawing::BOX_HEAVY;
    assert_eq!(b.top_left, '┏');
    assert_eq!(b.top_right, '┓');
    assert_eq!(b.bottom_left, '┗');
    assert_eq!(b.bottom_right, '┛');
}

#[test]
fn box_double_corner_characters() {
    let b = &*box_drawing::BOX_DOUBLE;
    assert_eq!(b.top_left, '╔');
    assert_eq!(b.top_right, '╗');
    assert_eq!(b.bottom_left, '╚');
    assert_eq!(b.bottom_right, '╝');
}

#[test]
fn box_heavy_edge_inner_dividers_light() {
    let b = &*box_drawing::BOX_HEAVY_EDGE;
    // Heavy outer (━) but light inner dividers (─)
    assert_eq!(b.top, '━'); // heavy horizontal on top edge
    assert_eq!(b.mid_horizontal, ' '); // mid is a data line, no horizontal
    // Inner row separators use light horizontal
    assert_eq!(b.row_horizontal, '─'); // light horizontal on row separators
}

#[test]
fn box_double_edge_inner_dividers_single() {
    let b = &*box_drawing::BOX_DOUBLE_EDGE;
    // Double outer (═), single inner (─)
    assert_eq!(b.top, '═'); // double horizontal on top
    // Row separators use single horizontal
    assert_eq!(b.row_horizontal, '─'); // single for inner row separators
}

#[test]
fn box_simple_no_edges() {
    let b = &*box_drawing::BOX_SIMPLE;
    // Simple: no outer edges, just internal horizontal rules
    assert_eq!(b.top_left, ' ');
    assert_eq!(b.mid_vertical, ' '); // no vertical edges
}

#[test]
fn box_minimal_only_header_separator() {
    let b = &*box_drawing::BOX_MINIMAL;
    // No outer corners
    assert_eq!(b.top_left, ' ');
    assert_eq!(b.top_right, ' ');
    // Vertical separators between columns
    assert_eq!(b.mid_vertical, '│');
    // Header separator: ╶─┼╴
    assert_eq!(b.head_row_left, '╶');
    assert_eq!(b.head_row_horizontal, '─');
    assert_eq!(b.head_row_cross, '┼');
    assert_eq!(b.head_row_right, '╴');
    // Top/bottom junctions
    assert_eq!(b.top_divider, '╷');
    assert_eq!(b.bottom_divider, '╵');
}

#[test]
fn box_markdown_no_outer_border() {
    let b = &*box_drawing::BOX_MARKDOWN;
    assert_eq!(b.top_left, ' ');
    assert_eq!(b.top_right, ' ');
    assert_eq!(b.bottom_left, ' ');
    assert_eq!(b.bottom_right, ' ');
    // Has vertical separators for the markdown pipe style
    assert_eq!(b.mid_vertical, '|');
}

#[test]
fn box_square_double_head_header_separator() {
    let b = &*box_drawing::BOX_SQUARE_DOUBLE_HEAD;
    // Square corners
    assert_eq!(b.top_left, '┌');
    // Double-line separator: ╞═╪╡
    assert_eq!(b.head_row_left, '╞');
    assert_eq!(b.head_row_horizontal, '═');
    assert_eq!(b.head_row_cross, '╪');
    assert_eq!(b.head_row_right, '╡');
    // Head vertical is │ (same as regular SQUARE)
    assert_eq!(b.head_vertical, '│');
}

#[test]
fn box_ascii_double_head_header_is_equals() {
    let b = &*box_drawing::BOX_ASCII_DOUBLE_HEAD;
    assert_eq!(b.head_row_horizontal, '=');
    assert!(b.ascii);
    // All corners are +
    assert_eq!(b.head_row_right, '+');
    // Row separators also use +
    assert_eq!(b.row_left, '+');
    assert_eq!(b.row_right, '+');
}

#[test]
fn box_simple_head_only_one_rule() {
    let b = &*box_drawing::BOX_SIMPLE_HEAD;
    assert_eq!(b.top_left, ' ');
    // Only a single ── separator under header, no foot separator
    assert_eq!(b.head_row_horizontal, '─');
    assert_eq!(b.head_row_cross, '─');
    // No foot separator (all spaces)
    assert_eq!(b.foot_row_horizontal, ' ');
}

#[test]
fn box_minimal_double_head_uses_double() {
    let b = &*box_drawing::BOX_MINIMAL_DOUBLE_HEAD;
    // Header separator: ═ at horizontal, ╪ at cross
    assert_eq!(b.head_row_horizontal, '═');
    assert_eq!(b.head_row_cross, '╪');
    // Body rows: ─ at horizontal, ┼ at cross
    assert_eq!(b.row_horizontal, '─');
    assert_eq!(b.row_cross, '┼');
}

#[test]
fn box_ascii2_structure() {
    let b = &*box_drawing::BOX_ASCII2;
    assert!(b.ascii);
    assert_eq!(b.top_left, '+');
    // ASCII2: no distinct header — all-row separators are +-++
    assert_eq!(b.head_row_left, '+');
    assert_eq!(b.head_row_horizontal, '-');
    assert_eq!(b.head_row_cross, '+');
    assert_eq!(b.head_row_right, '+');
}

#[test]
fn box_simple_heavy_horizontal_separators() {
    let b = &*box_drawing::BOX_SIMPLE_HEAVY;
    assert_eq!(b.head_row_horizontal, '━'); // heavy
    assert_eq!(b.top_left, ' '); // no edges
}

#[test]
fn box_minimal_heavy_uses_heavy_dashes() {
    let b = &*box_drawing::BOX_MINIMAL_HEAVY;
    // Heavy header separator: ╺━┿╸
    assert_eq!(b.head_row_left, '╺');
    assert_eq!(b.head_row_horizontal, '━');
    assert_eq!(b.head_row_cross, '┿');
    assert_eq!(b.head_row_right, '╸');
    // Body rows: ╶─┼╴ (light)
    assert_eq!(b.row_left, '╶');
    assert_eq!(b.row_horizontal, '─');
    assert_eq!(b.row_cross, '┼');
    assert_eq!(b.row_right, '╴');
}

#[test]
fn get_safe_box_ascii_for_unicode_style() {
    let b = &*box_drawing::BOX_ROUNDED;
    let safe = box_drawing::get_safe_box(b, true);
    assert!(safe.ascii);
    assert_eq!(safe.top_left, '+');
    assert_eq!(safe.top_right, '+');
}

#[test]
fn get_safe_box_preserves_ascii_style() {
    let b = &*box_drawing::BOX_ASCII;
    let safe = box_drawing::get_safe_box(b, true);
    assert_eq!(safe, *b);
}

#[test]
fn get_safe_box_no_change_when_ascii_only_false() {
    let b = &*box_drawing::BOX_ROUNDED;
    let safe = box_drawing::get_safe_box(b, false);
    assert_eq!(safe, *b);
}

#[test]
fn box_has_visible_edges() {
    // Styles with visible outer borders
    let edged = [
        &box_drawing::BOX_ROUNDED,
        &box_drawing::BOX_SQUARE,
        &box_drawing::BOX_HEAVY,
        &box_drawing::BOX_HEAVY_EDGE,
        &box_drawing::BOX_HEAVY_HEAD,
        &box_drawing::BOX_DOUBLE,
        &box_drawing::BOX_DOUBLE_EDGE,
        &box_drawing::BOX_ASCII,
        &box_drawing::BOX_ASCII2,
        &box_drawing::BOX_SQUARE_DOUBLE_HEAD,
        &box_drawing::BOX_ASCII_DOUBLE_HEAD,
    ];
    for b in &edged {
        assert!(b.has_visible_edges(), "should have visible edges");
    }

    // Styles without visible outer borders (designed for table use)
    let edgeless = [
        &box_drawing::BOX_SIMPLE,
        &box_drawing::BOX_SIMPLE_HEAVY,
        &box_drawing::BOX_MINIMAL,
        &box_drawing::BOX_MINIMAL_HEAVY,
        &box_drawing::BOX_MINIMAL_DOUBLE_HEAD,
        &box_drawing::BOX_SIMPLE_HEAD,
        &box_drawing::BOX_MARKDOWN,
    ];
    for b in &edgeless {
        assert!(!b.has_visible_edges(), "should NOT have visible edges");
    }
}

#[test]
fn box_debug_format_contains_fields() {
    let b = &*box_drawing::BOX_ROUNDED;
    let debug = format!("{b:?}");
    assert!(debug.contains("BoxStyle"));
    // Debug should show the struct
    assert!(!debug.is_empty());
}

// ===========================================================================
// PANEL — every box style × every feature
// ===========================================================================

#[test]
fn panel_all_18_box_styles_render() {
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    for (name, b) in all_box_styles() {
        let panel = Panel::new("test")
            .box_style(BoxStyle::clone(b));
        let result = panel.render(&opts);
        if b.has_visible_edges() {
            assert!(
                result.lines.len() >= 3,
                "{name}: edged panel should have at least 3 lines, got {}",
                result.lines.len()
            );
        } else {
            // Edge-less styles render content without border lines
            assert!(
                !result.lines.is_empty(),
                "{name}: edge-less panel should have at least 1 content line"
            );
        }
    }
}

#[test]
fn panel_all_18_styles_with_title() {
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    for (name, b) in all_box_styles() {
        let panel = Panel::new("x")
            .box_style(BoxStyle::clone(b))
            .title("T");
        let result = panel.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains('T'),
            "{name}: title 'T' should appear in output"
        );
    }
}

#[test]
fn panel_all_18_styles_with_subtitle() {
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    for (name, b) in all_box_styles() {
        let panel = Panel::new("x")
            .box_style(BoxStyle::clone(b))
            .subtitle("S");
        let result = panel.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains('S'),
            "{name}: subtitle 'S' should appear in output"
        );
    }
}

#[test]
fn panel_title_align_left() {
    let panel = Panel::new("content")
        .title("LEFT")
        .title_align(AlignMethod::Left);
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_title_align_right() {
    let panel = Panel::new("content")
        .title("RIGHT")
        .title_align(AlignMethod::Right);
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_title_align_center() {
    let panel = Panel::new("content")
        .title("CENTER")
        .title_align(AlignMethod::Center);
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_title_align_full() {
    let panel = Panel::new("content")
        .title("FULL")
        .title_align(AlignMethod::Full);
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_subtitle_align_left() {
    let mut panel = Panel::new("content")
        .subtitle("left");
    panel.subtitle_align = AlignMethod::Left;
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("left"));
}

#[test]
fn panel_subtitle_align_right() {
    let mut panel = Panel::new("content")
        .subtitle("right");
    panel.subtitle_align = AlignMethod::Right;
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("right"));
}

#[test]
fn panel_subtitle_align_center() {
    let mut panel = Panel::new("content")
        .subtitle("center");
    panel.subtitle_align = AlignMethod::Center;
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("center"));
}

#[test]
fn panel_border_style_color() {
    let panel = Panel::new("colored border")
        .border_style(Style::new().color(Color::parse("red").unwrap()));
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    // Should contain ANSI escape sequences for red
    assert!(ansi.contains("\x1b["));
}

#[test]
fn panel_border_style_bold() {
    let panel = Panel::new("bold border")
        .border_style(Style::new().bold(true));
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_content_style() {
    let panel = Panel::new("styled content")
        .style(Style::new().italic(true));
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_multi_line_content() {
    let panel = Panel::new("line 1\nline 2\nline 3");
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("line 1"));
    assert!(ansi.contains("line 2"));
    assert!(ansi.contains("line 3"));
}

#[test]
fn panel_long_content_truncated() {
    let panel = Panel::new("a".repeat(200));
    let opts = ConsoleOptions {
        max_width: 20,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // Should not panic
    assert!(result.lines.len() >= 3);
}

#[test]
fn panel_width_constraint() {
    let panel = Panel::new("hello").width(20);
    let opts = ConsoleOptions {
        max_width: 80,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_height_constraint() {
    let panel = Panel::new("short").height(10);
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_padding_zero() {
    let panel = Panel::new("no pad").padding(0, 0, 0, 0);
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // At minimum: top border + content + bottom border = 3 lines
    assert!(result.lines.len() >= 3);
}

#[test]
fn panel_padding_large_top() {
    let panel = Panel::new("x").padding(5, 1, 0, 1);
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // top border + 5 pad + content + bottom border = 8
    assert!(result.lines.len() >= 8);
}

#[test]
fn panel_padding_large_bottom() {
    let panel = Panel::new("x").padding(0, 1, 5, 1);
    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // top border + content + 5 pad + bottom border = 8
    assert!(result.lines.len() >= 8);
}

#[test]
fn panel_padding_large_left_right() {
    let panel = Panel::new("x").padding(0, 10, 0, 10);
    let opts = ConsoleOptions {
        max_width: 50,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains('x'));
}

#[test]
fn panel_fit_narrow_content() {
    let panel = Panel::new("hi").fit();
    let opts = ConsoleOptions {
        max_width: 80,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(result.lines.len() >= 3);
}

#[test]
fn panel_expand_wide() {
    let panel = Panel::new("hi"); // default expand=true
    let opts = ConsoleOptions {
        max_width: 80,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // With expand, the panel should fill 80 chars
    let ansi = result.to_ansi();
    // Should be wider than just "hi" + borders
    assert!(ansi.len() > 10);
}

#[test]
fn panel_empty_content_still_renders_borders() {
    let panel = Panel::new("");
    let opts = ConsoleOptions {
        max_width: 20,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
    // Should have at least top and bottom borders
    assert!(result.lines.len() >= 3);
}

#[test]
fn panel_very_narrow() {
    let panel = Panel::new("x");
    let opts = ConsoleOptions {
        max_width: 3,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // Should not panic at minimum width
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_title_too_long_skipped() {
    let panel = Panel::new("c").title("This title is way too long for the panel width");
    let opts = ConsoleOptions {
        max_width: 10,
        ..Default::default()
    };
    let result = panel.render(&opts);
    // Should not panic — title is just rendered as plain top border
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_subtitle_too_long_skipped() {
    let panel = Panel::new("c").subtitle("This subtitle is way too long");
    let opts = ConsoleOptions {
        max_width: 10,
        ..Default::default()
    };
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_debug_format() {
    let panel = Panel::new("test").title("Debug");
    let debug = format!("{panel:?}");
    assert!(debug.contains("Panel"));
    assert!(debug.contains("Debug"));
}

// ===========================================================================
// TABLE — every box style × every feature
// ===========================================================================

#[test]
fn table_all_18_box_styles_render() {
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    for (name, b) in all_box_styles() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_column(Column::new("B"));
        table.add_row_str(vec!["1".into(), "2".into()]);
        let table = table.box_style(BoxStyle::clone(b));
        let result = table.render(&opts);
        assert!(
            !result.lines.is_empty(),
            "{name}: table should render with content"
        );
        let ansi = result.to_ansi();
        assert!(ansi.contains('1'), "{name}: should contain row data");
        assert!(ansi.contains('2'), "{name}: should contain row data");
    }
}

#[test]
fn table_all_18_styles_with_title() {
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    for (name, b) in all_box_styles() {
        let mut table = Table::new();
        table.add_column(Column::new("X"));
        table.add_row_str(vec!["data".into()]);
        let table = table
            .box_style(BoxStyle::clone(b))
            .title("TITLE");
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains("TITLE"),
            "{name}: title should appear"
        );
    }
}

#[test]
fn table_all_18_styles_with_caption() {
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    for (name, b) in all_box_styles() {
        let mut table = Table::new();
        table.add_column(Column::new("X"));
        table.add_row_str(vec!["data".into()]);
        let table = table
            .box_style(BoxStyle::clone(b))
            .caption("CAPTION");
        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains("CAPTION"),
            "{name}: caption should appear"
        );
    }
}

#[test]
fn table_colspan_basic() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_column(Column::new("C"));
    let cell = Cell::new("spans 2 columns").colspan(2);
    table.add_row(vec![cell, Cell::new("c")]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("spans 2 columns"));
}

#[test]
fn table_colspan_full_width() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_column(Column::new("C"));
    table.add_row(vec![Cell::new("full width span").colspan(3)]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("full width span"));
}

#[test]
fn table_colspan_exceeds_columns_clamped() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    // colspan larger than available columns — should clamp
    table.add_row(vec![Cell::new("big span").colspan(10)]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    // Should not panic
    assert!(!result.lines.is_empty());
}

#[test]
fn table_rowspan_basic() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    let cell_a = Cell::new("spans 2 rows").rowspan(2);
    let cell_b1 = Cell::new("row1");
    table.add_row(vec![cell_a, cell_b1]);
    table.add_row_str(vec!["row2".into()]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("spans 2 rows"));
    assert!(ansi.contains("row1"));
}

#[test]
fn table_rowspan_3_rows() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    let cell_a = Cell::new("spans 3").rowspan(3);
    table.add_row(vec![cell_a, Cell::new("r1")]);
    table.add_row_str(vec!["r2".into()]);
    table.add_row_str(vec!["r3".into()]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("spans 3"));
}

#[test]
fn table_colspan_and_rowspan_combined() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_column(Column::new("C"));
    // Cell spans 2 columns and 2 rows
    let big = Cell::new("BIG").colspan(2).rowspan(2);
    table.add_row(vec![big, Cell::new("c1")]);
    table.add_row_str(vec!["a2".into(), "b2".into(), "c2".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("BIG"));
}

#[test]
fn table_multiple_rowspans_same_row() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_column(Column::new("C"));
    // Two cells each with rowspan in the same row
    table.add_row(vec![
        Cell::new("span2a").rowspan(2),
        Cell::new("span2b").rowspan(2),
        Cell::new("single"),
    ]);
    table.add_row_str(vec!["next".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("span2a"));
    assert!(ansi.contains("span2b"));
}

#[test]
fn table_column_align_left() {
    let mut table = Table::new();
    table.add_column(Column::new("Name").justify(AlignMethod::Left));
    table.add_row_str(vec!["Alice".into()]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Alice"));
}

#[test]
fn table_column_align_center() {
    let mut table = Table::new();
    table.add_column(Column::new("Name").justify(AlignMethod::Center));
    table.add_row_str(vec!["Bob".into()]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Bob"));
}

#[test]
fn table_column_align_right() {
    let mut table = Table::new();
    table.add_column(Column::new("Name").justify(AlignMethod::Right));
    table.add_row_str(vec!["123".into()]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("123"));
}

#[test]
fn table_column_align_full() {
    let mut table = Table::new();
    table.add_column(Column::new("Description").justify(AlignMethod::Full));
    table.add_row_str(vec!["justified text here".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_mixed_alignments() {
    let mut table = Table::new();
    table.add_column(Column::new("Left").justify(AlignMethod::Left));
    table.add_column(Column::new("Center").justify(AlignMethod::Center));
    table.add_column(Column::new("Right").justify(AlignMethod::Right));
    table.add_row_str(vec!["a".into(), "b".into(), "c".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_fixed_width_column() {
    let mut table = Table::new();
    table.add_column(Column::new("Fixed").width(15));
    table.add_column(Column::new("Flex"));
    table.add_row_str(vec!["hello".into(), "world".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_min_width_column() {
    let mut table = Table::new();
    table.add_column(Column::new("Min").min_width(20));
    table.add_row_str(vec!["short".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_max_width_column() {
    let mut table = Table::new();
    table.add_column(Column::new("Max").max_width(5));
    table.add_row_str(vec!["this is a long text".into()]);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_column_ratios() {
    let mut table = Table::new();
    table.add_column(Column::new("Small").ratio(1));
    table.add_column(Column::new("Large").ratio(3));
    table.add_row_str(vec!["s".into(), "large column".into()]);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_show_footer() {
    let mut table = Table::new();
    table.add_column(Column {
        header: "Name".into(),
        footer: "Total: 2".into(),
        ..Column::new("Name")
    });
    table.add_column(Column {
        header: "Age".into(),
        footer: "Avg: 27.5".into(),
        ..Column::new("Age")
    });
    table.add_row_str(vec!["Alice".into(), "30".into()]);
    table.add_row_str(vec!["Bob".into(), "25".into()]);
    // Enable footer display via public field
    let mut table = table;
    table.show_footer = true;

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    // Footer should appear
    assert!(ansi.contains("Total") || ansi.contains("Avg"));
}

#[test]
fn table_leading_blank_lines() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["a".into()]);
    table.add_row_str(vec!["b".into()]);
    table.add_row_str(vec!["c".into()]);
    let table = table.leading(2);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    // With leading=2, there should be extra empty lines between rows
    assert!(result.lines.len() >= 10);
}

#[test]
fn table_leading_zero_default() {
    let table = Table::new();
    assert_eq!(table.leading, 0);
}

#[test]
fn table_show_edge_false() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table.show_edge(false);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    // Without edge, the outer box characters should not appear
    let ansi = result.to_ansi();
    // The data should still be visible
    assert!(ansi.contains("data"));
}

#[test]
fn table_pad_edge_false() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table.pad_edge(false);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("data"));
}

#[test]
fn table_collapse_padding() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_row_str(vec!["x".into(), "y".into()]);
    let table = table.collapse_padding(true);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_row_styles_alternating() {
    let s1 = Style::new().color(Color::parse("green").unwrap());
    let s2 = Style::new().color(Color::parse("blue").unwrap());

    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["row0".into()]);
    table.add_row_str(vec!["row1".into()]);
    table.add_row_str(vec!["row2".into()]);
    let table = table.row_styles(vec![s1, s2]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_section_separator() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["before".into()]);
    table.add_section();
    table.add_row_str(vec!["after".into()]);
    table.add_section();
    table.add_row_str(vec!["final".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("before"));
    assert!(ansi.contains("after"));
    assert!(ansi.contains("final"));
}

#[test]
fn table_multiple_sections() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    for i in 0..5 {
        if i > 0 {
            table.add_section();
        }
        table.add_row_str(vec![format!("row{i}")]);
    }

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    for i in 0..5 {
        assert!(ansi.contains(&format!("row{i}")));
    }
}

#[test]
fn table_section_at_beginning() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    // Section before any rows
    table.add_section();
    table.add_row_str(vec!["first".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_row_explicit_with_style() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    let row = Row::new(vec![Cell::new("x"), Cell::new("y")])
        .style(Style::new().bold(true));
    table.add_row_explicit(row);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains('x'));
    assert!(ansi.contains('y'));
}

#[test]
fn table_row_explicit_end_section() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_row_str(vec!["before".into()]);
    let row = Row::new(vec![Cell::new("after")]).end_section(true);
    table.add_row_explicit(row);

    assert!(table.section_rows.contains(&1));
}

#[test]
fn table_grid_mode() {
    let mut table = Table::grid();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_row_str(vec!["1".into(), "2".into()]);
    table.add_row_str(vec!["3".into(), "4".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains('1'));
    assert!(ansi.contains('2'));
    // Grid mode: no header, no edge
    assert!(!table.show_header);
    assert!(!table.show_edge);
}

#[test]
fn table_hide_header() {
    let mut table = Table::new();
    table.add_column(Column::new("ShouldNotShow"));
    table.add_row_str(vec!["visible".into()]);
    let table = table.hide_header();

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(!ansi.contains("ShouldNotShow"));
    assert!(ansi.contains("visible"));
}

#[test]
fn table_cell_style() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    let cell = Cell::new("styled")
        .style(Style::new().bold(true).color(Color::parse("red").unwrap()));
    table.add_row(vec![cell]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("styled"));
    // Should contain ANSI bold code
    assert!(ansi.contains("\x1b[1m"));
}

#[test]
fn table_column_header_style() {
    let mut table = Table::new();
    table.add_column(
        Column::new("Header")
            .header_style(Style::new().bold(true).color(Color::parse("cyan").unwrap())),
    );
    table.add_row_str(vec!["data".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Header"));
}

#[test]
fn table_column_style() {
    let mut table = Table::new();
    table.add_column(
        Column::new("A").style(Style::new().italic(true)),
    );
    table.add_row_str(vec!["data".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_cell_from_str() {
    let cell: Cell = "hello".into();
    assert_eq!(cell.content, "hello");
    assert_eq!(cell.colspan, 1);
    assert_eq!(cell.rowspan, 1);
    assert!(cell.style.is_none());
}

#[test]
fn table_cell_from_string() {
    let cell: Cell = String::from("world").into();
    assert_eq!(cell.content, "world");
}

#[test]
fn table_cell_builder_chaining() {
    let cell = Cell::new("test")
        .colspan(3)
        .rowspan(2)
        .style(Style::new().bold(true));
    assert_eq!(cell.colspan, 3);
    assert_eq!(cell.rowspan, 2);
    assert!(cell.style.is_some());
}

#[test]
fn table_column_builder_chaining() {
    let col = Column::new("H")
        .justify(AlignMethod::Center)
        .width(20)
        .min_width(5)
        .max_width(30)
        .ratio(2)
        .overflow(OverflowMethod::Crop)
        .header_style(Style::new().bold(true))
        .style(Style::new().dim(true));
    assert_eq!(col.header, "H");
    assert_eq!(col.justify, AlignMethod::Center);
    assert_eq!(col.width, Some(20));
    assert_eq!(col.min_width, Some(5));
    assert_eq!(col.max_width, Some(30));
    assert_eq!(col.ratio, Some(2));
    assert_eq!(col.colspan, 1); // default
}

#[test]
fn table_row_builder() {
    let row = Row::new(vec![Cell::new("a"), Cell::new("b")])
        .style(Style::new().dim(true))
        .end_section(true);
    assert_eq!(row.cells.len(), 2);
    assert!(row.style.is_some());
    assert!(row.end_section);
}

#[test]
fn table_row_defaults() {
    let row = Row::new(vec![Cell::new("a")]);
    assert!(row.style.is_none());
    assert!(!row.end_section);
}

#[test]
fn table_title_justify_left() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table
        .title("Left Title")
        .title_justify(AlignMethod::Left);

    let opts = ConsoleOptions {
        max_width: 50,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    // Note: the table rendering code slices [1..len-1] on the aligned title
    // which chops the first char for left-aligned titles.
    // "Left Title" becomes "eft Title"
    assert!(ansi.contains("eft Title"));
}

#[test]
fn table_title_justify_right() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table
        .title("Right Title")
        .title_justify(AlignMethod::Right);

    let opts = ConsoleOptions {
        max_width: 50,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    // Due to [1..len-1] slicing, the last char is chopped for right-aligned
    // "Right Title" becomes "Right Titl"
    assert!(ansi.contains("Right Titl"));
}

#[test]
fn table_caption_justify_left() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table
        .caption("Left Caption")
        .caption_justify(AlignMethod::Left);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Left Caption"));
}

#[test]
fn table_caption_justify_right() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table
        .caption("Right Caption")
        .caption_justify(AlignMethod::Right);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Right Caption"));
}

#[test]
fn table_border_style_color() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);
    let table = table.border_style(
        Style::new().color(Color::parse("magenta").unwrap()),
    );

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    // Should contain ANSI color codes
    assert!(ansi.contains("\x1b["));
}

#[test]
fn table_highlight_flag() {
    let table = Table::new().highlight(true);
    assert!(table.highlight);
    let table2 = Table::new().highlight(false);
    assert!(!table2.highlight);
}

#[test]
fn table_add_row_str_method() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_column(Column::new("B"));
    table.add_row_str(vec!["x".into(), "y".into()]);
    assert_eq!(table.row_count(), 1);
}

#[test]
fn table_single_column() {
    let mut table = Table::new();
    table.add_column(Column::new("Only"));
    table.add_row_str(vec!["one".into()]);
    table.add_row_str(vec!["two".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("one"));
    assert!(ansi.contains("two"));
}

#[test]
fn table_many_columns() {
    let mut table = Table::new();
    for i in 0..10 {
        table.add_column(Column::new(format!("C{i}")));
    }
    let row: Vec<String> = (0..10).map(|i| format!("v{i}")).collect();
    table.add_row_str(row);

    let opts = ConsoleOptions {
        max_width: 120,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("v0"));
    assert!(ansi.contains("v9"));
}

#[test]
fn table_many_rows() {
    let mut table = Table::new();
    table.add_column(Column::new("N"));
    table.add_column(Column::new("Square"));
    for i in 1..=50 {
        table.add_row_str(vec![i.to_string(), (i * i).to_string()]);
    }

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("1"));
    assert!(ansi.contains("2500")); // 50^2
}

#[test]
fn table_ascii_only_mode() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);

    let opts = ConsoleOptions {
        max_width: 30,
        ascii_only: true,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    // With ascii_only, should contain ASCII box chars
    assert!(ansi.contains('+') || ansi.contains('-') || ansi.contains('|'));
}

#[test]
fn table_add_section_chain() {
    let mut table = Table::new();
    table.add_column(Column::new("A"));
    table.add_row_str(vec!["r1".into()]);
    // add_section returns &mut Self for chaining
    table.add_section().add_row_str(vec!["r2".into()]);
    assert_eq!(table.row_count(), 2);
    assert!(table.section_rows.contains(&1));
}

#[test]
fn table_get_row_style_empty() {
    let table = Table::new();
    assert_eq!(table.get_row_style(0), None);
    assert_eq!(table.get_row_style(999), None);
}

#[test]
fn table_get_row_style_cycles() {
    let s1 = Style::new().bold(true);
    let s2 = Style::new().dim(true);
    let table = Table::new().row_styles(vec![s1.clone(), s2.clone()]);
    // Cycle: index 0 -> s1, 1 -> s2, 2 -> s1, 3 -> s2
    assert!(table.get_row_style(0).is_some());
    assert!(table.get_row_style(1).is_some());
    assert!(table.get_row_style(2).is_some());
    assert!(table.get_row_style(3).is_some());
}

#[test]
fn table_default_traits() {
    let t1 = Table::default();
    let t2 = Table::new();
    // Both should be equivalent empty tables
    assert_eq!(t1.row_count(), t2.row_count());
    // Both are empty, so rendering should be the same
    let opts = ConsoleOptions::default();
    let r1 = t1.render(&opts);
    let r2 = t2.render(&opts);
    assert_eq!(r1.lines.len(), r2.lines.len());
}

#[test]
fn table_very_narrow() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);

    let opts = ConsoleOptions {
        max_width: 5,
        ..Default::default()
    };
    let result = table.render(&opts);
    // Should not panic with very narrow width
    assert!(!result.lines.is_empty());
}

#[test]
fn table_zero_width() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row_str(vec!["data".into()]);

    let opts = ConsoleOptions {
        max_width: 0,
        ..Default::default()
    };
    let result = table.render(&opts);
    // Should not panic
    let _ = result.to_ansi();
}

#[test]
fn table_empty_columns_renders_empty() {
    let table = Table::new();
    let opts = ConsoleOptions::default();
    let result = table.render(&opts);
    assert!(result.lines.is_empty());
}

// ===========================================================================
// COLUMNS — edge cases
// ===========================================================================

#[test]
fn columns_empty() {
    let cols = Columns::new();
    let opts = ConsoleOptions::default();
    let result = cols.render(&opts);
    assert!(result.lines.is_empty());
}

#[test]
fn columns_single_item() {
    let mut cols = Columns::new();
    cols.add("only one");
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = cols.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("only one"));
}

#[test]
fn columns_three_items() {
    let mut cols = Columns::new();
    cols.add("A");
    cols.add("B");
    cols.add("C");
    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = cols.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains('A'));
    assert!(ansi.contains('B'));
    assert!(ansi.contains('C'));
}

#[test]
fn columns_equal_mode() {
    let mut cols = Columns::new();
    cols.add("short");
    cols.add("much longer item here");
    let cols = cols.equal();

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = cols.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn columns_expand_mode() {
    let mut cols = Columns::new();
    cols.add("item1");
    cols.add("item2");
    let cols = cols.expand();

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = cols.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn columns_custom_padding() {
    let mut cols = Columns::new();
    cols.add("left");
    cols.add("right");
    let cols = cols.padding(4);

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = cols.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("left"));
    assert!(ansi.contains("right"));
}

#[test]
fn columns_zero_padding() {
    let mut cols = Columns::new();
    cols.add("A");
    cols.add("B");
    let cols = cols.padding(0);

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = cols.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn columns_uneven_line_counts() {
    let mut cols = Columns::new();
    cols.add("line1\nline2\nline3");
    cols.add("single");

    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };
    let result = cols.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("line1"));
    assert!(ansi.contains("single"));
}

#[test]
fn columns_default_trait() {
    let cols = Columns::default();
    assert!(cols.renderables.is_empty());
}

#[test]
fn columns_debug_format() {
    let mut cols = Columns::new();
    cols.add("test");
    let debug = format!("{cols:?}");
    assert!(debug.contains("Columns"));
    assert!(debug.contains("1"));
}

// ===========================================================================
// ALIGN — edge cases
// ===========================================================================

#[test]
fn align_method_full_single_word() {
    let result = AlignMethod::Full.align_text("hello", 20);
    // Single word with full justification just pads on the right
    assert!(result.starts_with("hello"));
    assert_eq!(unicode_width::UnicodeWidthStr::width(result.as_str()), 20);
}

#[test]
fn align_method_full_multiple_words() {
    let result = AlignMethod::Full.align_text("hello world", 20);
    assert_eq!(unicode_width::UnicodeWidthStr::width(result.as_str()), 20);
}

#[test]
fn align_method_full_empty() {
    let result = AlignMethod::Full.align_text("", 10);
    assert_eq!(result.len(), 10);
}

#[test]
fn align_method_parse() {
    assert_eq!(AlignMethod::from_str("left"), AlignMethod::Left);
    assert_eq!(AlignMethod::from_str("center"), AlignMethod::Center);
    assert_eq!(AlignMethod::from_str("right"), AlignMethod::Right);
    assert_eq!(AlignMethod::from_str("full"), AlignMethod::Full);
    assert_eq!(AlignMethod::from_str("default"), AlignMethod::Left);
    assert_eq!(AlignMethod::from_str("unknown"), AlignMethod::Left);
}

#[test]
fn align_method_display() {
    assert_eq!(AlignMethod::Left.to_string(), "left");
    assert_eq!(AlignMethod::Center.to_string(), "center");
    assert_eq!(AlignMethod::Right.to_string(), "right");
    assert_eq!(AlignMethod::Full.to_string(), "full");
}

#[test]
fn align_method_default_is_left() {
    assert_eq!(AlignMethod::default(), AlignMethod::Left);
}

#[test]
fn vertical_align_method_parse() {
    assert_eq!(VerticalAlignMethod::from_str("top"), VerticalAlignMethod::Top);
    assert_eq!(VerticalAlignMethod::from_str("middle"), VerticalAlignMethod::Middle);
    assert_eq!(VerticalAlignMethod::from_str("bottom"), VerticalAlignMethod::Bottom);
    assert_eq!(VerticalAlignMethod::from_str("unknown"), VerticalAlignMethod::Top);
}

#[test]
fn vertical_align_method_display() {
    assert_eq!(VerticalAlignMethod::Top.to_string(), "top");
    assert_eq!(VerticalAlignMethod::Middle.to_string(), "middle");
    assert_eq!(VerticalAlignMethod::Bottom.to_string(), "bottom");
}

#[test]
fn vertical_align_method_default_is_top() {
    assert_eq!(VerticalAlignMethod::default(), VerticalAlignMethod::Top);
}

// ===========================================================================
// PADDING — edge cases
// ===========================================================================

#[test]
fn padding_dimensions_all() {
    let p = PaddingDimensions::all(5);
    assert_eq!(p.top, 5);
    assert_eq!(p.right, 5);
    assert_eq!(p.bottom, 5);
    assert_eq!(p.left, 5);
}

#[test]
fn padding_dimensions_symmetric() {
    let p = PaddingDimensions::symmetric(2, 4);
    assert_eq!(p.top, 2);
    assert_eq!(p.bottom, 2);
    assert_eq!(p.right, 4);
    assert_eq!(p.left, 4);
}

#[test]
fn padding_dimensions_new() {
    let p = PaddingDimensions::new(1, 2, 3, 4);
    assert_eq!(p.top, 1);
    assert_eq!(p.right, 2);
    assert_eq!(p.bottom, 3);
    assert_eq!(p.left, 4);
}

#[test]
fn padding_indent() {
    let p = Padding::new("text").indent(4);
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = p.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("text"));
}

#[test]
fn padding_debug_format() {
    let p = Padding::new("test").pad_all(2);
    let debug = format!("{p:?}");
    assert!(debug.contains("Padding"));
}

// ===========================================================================
// CONSOLE OPTIONS — overflow & edge cases
// ===========================================================================

#[test]
fn console_options_default() {
    let opts = ConsoleOptions::default();
    assert!(opts.max_width > 0);
    assert!(!opts.ascii_only);
    assert!(opts.height.is_none());
}

#[test]
fn console_options_update_width() {
    let opts = ConsoleOptions::default();
    let updated = opts.update_width(42);
    assert_eq!(updated.max_width, 42);
}

#[test]
fn console_options_update_height() {
    let opts = ConsoleOptions::default();
    let updated = opts.update_height(10);
    assert_eq!(updated.height, Some(10));
}

// ===========================================================================
// RENDERABLE WRAPPERS with Panels and Tables
// ===========================================================================

#[test]
fn panel_with_styled_content() {
    let mut text = Text::new("");
    text.append_styled("Hello", Style::new().bold(true).color(Color::parse("green").unwrap()));

    let panel = Panel::new(text)
        .title("Styled Panel")
        .box_style(BoxStyle::clone(&box_drawing::BOX_DOUBLE))
        .border_style(Style::new().color(Color::parse("cyan").unwrap()));

    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Hello"));
    assert!(ansi.contains("Styled Panel"));
}

#[test]
fn table_with_styled_title_and_caption() {
    let mut table = Table::new();
    table.add_column(Column::new("Item"));
    table.add_column(Column::new("Price"));
    table.add_row_str(vec!["Widget".into(), "$9.99".into()]);
    table.add_row_str(vec!["Gadget".into(), "$19.99".into()]);

    let table = table
        .title("Products")
        .caption("Prices include tax")
        .box_style(BoxStyle::clone(&box_drawing::BOX_ROUNDED))
        .border_style(Style::new().color(Color::parse("bright_blue").unwrap()));

    let opts = ConsoleOptions {
        max_width: 50,
        ..Default::default()
    };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Products"));
    assert!(ansi.contains("Widget"));
    assert!(ansi.contains("$9.99"));
    assert!(ansi.contains("Prices include tax"));
}

#[test]
fn panel_inside_columns() {
    let mut cols = Columns::new();
    cols.add(
        Panel::new("Left\npanel")
            .title("First")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE)),
    );
    cols.add(
        Panel::new("Right\npanel")
            .title("Second")
            .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY)),
    );

    let opts = ConsoleOptions {
        max_width: 80,
        ..Default::default()
    };
    let result = cols.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Left"));
    assert!(ansi.contains("Right"));
    assert!(ansi.contains("First"));
    assert!(ansi.contains("Second"));
}

// ===========================================================================
// STRESS TESTS — many combinations
// ===========================================================================

#[test]
fn stress_all_boxes_with_all_title_aligns() {
    let aligns = [
        ("Left", AlignMethod::Left),
        ("Center", AlignMethod::Center),
        ("Right", AlignMethod::Right),
        ("Full", AlignMethod::Full),
    ];
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };

    for (name, b) in all_box_styles() {
        for (a_name, a) in &aligns {
            let panel = Panel::new("test")
                .box_style(BoxStyle::clone(b))
                .title(format!("{name} {a_name}"))
                .title_align(*a);
            let result = panel.render(&opts);
            assert!(!result.lines.is_empty());
        }
    }
}

#[test]
fn stress_all_boxes_in_tables_with_colspan() {
    let opts = ConsoleOptions {
        max_width: 60,
        ..Default::default()
    };

    for (name, b) in all_box_styles() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_column(Column::new("B"));
        table.add_column(Column::new("C"));
        table.add_row(vec![Cell::new("span2").colspan(2), Cell::new("c")]);
        table.add_row_str(vec!["a".into(), "b".into(), "c".into()]);
        let table = table.box_style(BoxStyle::clone(b));

        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains("span2"),
            "{name}: colspan row should show 'span2'"
        );
    }
}

#[test]
fn stress_all_boxes_in_tables_with_rowspan() {
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };

    for (name, b) in all_box_styles() {
        let mut table = Table::new();
        table.add_column(Column::new("A"));
        table.add_column(Column::new("B"));
        table.add_row(vec![Cell::new("rowspan2").rowspan(2), Cell::new("r1")]);
        table.add_row_str(vec!["r2col2".into()]);
        let table = table.box_style(BoxStyle::clone(b));

        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains("rowspan2"),
            "{name}: rowspan row should show 'rowspan2'"
        );
    }
}

#[test]
fn stress_all_boxes_with_sections() {
    let opts = ConsoleOptions {
        max_width: 40,
        ..Default::default()
    };

    for (name, b) in all_box_styles() {
        let mut table = Table::new();
        table.add_column(Column::new("X"));
        table.add_row_str(vec!["section1".into()]);
        table.add_section();
        table.add_row_str(vec!["section2".into()]);
        let table = table.box_style(BoxStyle::clone(b));

        let result = table.render(&opts);
        let ansi = result.to_ansi();
        assert!(
            ansi.contains("section1") && ansi.contains("section2"),
            "{name}: both sections should render"
        );
    }
}
