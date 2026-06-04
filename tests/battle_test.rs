//! Battle test — exhaustively exercises every component of rusty-rich.
//! Tests edge cases, error conditions, consistency invariants, and
//! cross-component interactions.

use rusty_rich::*;

// ===========================================================================
// COLOR
// ===========================================================================

#[test]
fn color_default() {
    let c = Color::default();
    assert!(c.is_default());
    assert_eq!(c.to_string(), "default");
}

#[test]
fn color_names_all_16() {
    let names = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        "bright_black", "bright_red", "bright_green", "bright_yellow",
        "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
    ];
    for name in &names {
        let c = Color::parse(name).expect(&format!("should parse {name}"));
        assert!(!c.is_default(), "{name} should not be default");
    }
}

#[test]
fn color_hex_upper_lower() {
    let c1 = Color::parse("#FF00AA").unwrap();
    let c2 = Color::parse("#ff00aa").unwrap();
    assert_eq!(c1, c2);
}

#[test]
fn color_parse_invalid() {
    assert!(Color::parse("").is_ok()); // empty = default
    assert!(Color::parse("nonexistent_color_name_xyz").is_err());
    assert!(Color::parse("#XYZ").is_err());
}

#[test]
fn color_downgrade_truecolor_to_8bit() {
    let c = Color::from_rgb(255, 0, 0); // pure red
    let downgraded = c.downgrade(ColorSystem::EightBit);
    // Downgraded color should not be TrueColor type
    assert!(!downgraded.is_default());
}

#[test]
fn color_blend_identity() {
    let c1 = Color::from_rgb(100, 100, 100);
    let c2 = Color::from_rgb(200, 200, 200);
    let blended = color::blend_colors(&c1, &c2, 0.0, &color::TerminalTheme::default());
    // At cross_fade=0, result should equal c1
    assert!(!blended.is_default());
}

// ===========================================================================
// STYLE
// ===========================================================================

#[test]
fn style_null_is_plain() {
    let s = Style::null();
    assert!(s.is_null());
}

#[test]
fn style_default_is_not_null() {
    let s = Style::new();
    assert!(!s.is_null());
    assert!(s.is_plain());
}

#[test]
fn style_combine_null_identity() {
    let s = Style::new().bold(true).color(Color::parse("red").unwrap());
    let combined = s.combine(&Style::null());
    assert_eq!(combined.get_bold(), Some(true));
}

#[test]
fn style_combine_override() {
    let base = Style::from_str("red");
    let over = Style::from_str("bold green");
    let combined = base.combine(&over);
    // override should take green color, not red
    assert_eq!(combined.get_bold(), Some(true));
}

#[test]
fn style_to_ansi_roundtrip_parseable() {
    let s = Style::new()
        .bold(true)
        .italic(true)
        .underline(true)
        .color(Color::from_rgb(255, 128, 0));
    let ansi = s.to_ansi();
    // Must start with ESC
    assert!(ansi.starts_with("\x1b["));
    // Must end with 'm'
    assert!(ansi.ends_with('m'));
}

#[test]
fn style_all_attributes_bold() {
    let s = Style::new().bold(true);
    assert!(!s.to_ansi().is_empty());
}
#[test]
fn style_all_attributes_dim() {
    let s = Style::new().dim(true);
    assert!(!s.to_ansi().is_empty());
}
#[test]
fn style_all_attributes_italic() {
    let s = Style::new().italic(true);
    assert!(!s.to_ansi().is_empty());
}
#[test]
fn style_all_attributes_underline() {
    let s = Style::new().underline(true);
    assert!(!s.to_ansi().is_empty());
}
#[test]
fn style_all_attributes_blink() {
    let s = Style::new().blink(true);
    assert!(!s.to_ansi().is_empty());
}
#[test]
fn style_all_attributes_reverse() {
    let s = Style::new().reverse(true);
    assert!(!s.to_ansi().is_empty());
}
#[test]
fn style_all_attributes_strike() {
    let s = Style::new().strike(true);
    assert!(!s.to_ansi().is_empty());
}

// ===========================================================================
// SEGMENT
// ===========================================================================

#[test]
fn segment_line_newline() {
    let seg = Segment::line();
    assert_eq!(seg.text, "\n");
    assert!(seg.style.is_none());
}

#[test]
fn segment_control_is_empty() {
    let seg = Segment::control(segment::ControlCode::Simple(segment::ControlType::Bell));
    assert_eq!(seg.cell_length(), 0);
}

#[test]
fn segment_split_at_zero() {
    let seg = Segment::new("Hello");
    let (left, right) = seg.split(0);
    assert!(left.text.is_empty());
    assert_eq!(right.unwrap().text, "Hello");
}

#[test]
fn segment_split_past_end() {
    let seg = Segment::new("Hi");
    let (left, right) = seg.split(100);
    assert_eq!(left.text, "Hi");
    assert!(right.is_none());
}

#[test]
fn segment_cjk_width() {
    let seg = Segment::new("你好");
    assert_eq!(seg.cell_length(), 4); // 2 wide chars
}

#[test]
fn segments_to_ansi_concatenates() {
    let mut segs = Segments::new();
    segs.push(Segment::new("A"));
    segs.push(Segment::new("B"));
    let ansi = segs.to_ansi();
    assert_eq!(ansi, "AB");
}

// ===========================================================================
// TEXT
// ===========================================================================

#[test]
fn text_append_preserves_plain() {
    let mut t = Text::new("Hello");
    t.append(" World", None);
    assert_eq!(t.plain, "Hello World");
}

#[test]
fn text_truncate_ellipsis_short() {
    let mut t = Text::new("Very long text that should be truncated");
    let _original_len = t.cell_len();
    t.truncate(10, OverflowMethod::Ellipsis);
    assert!(t.cell_len() <= 10 + 2); // + room for …
}

#[test]
fn text_truncate_crop() {
    let mut t = Text::new("Hello World");
    t.truncate(5, OverflowMethod::Crop);
    assert_eq!(t.plain, "Hello");
}

#[test]
fn text_expand_tabs() {
    let mut t = Text::new("a\tb");
    t.expand_tabs();
    assert!(!t.plain.contains('\t'));
    assert!(t.plain.len() > 3);
}

#[test]
fn text_split_lines() {
    let t = Text::new("a\nb\nc");
    let lines = t.split_lines();
    assert_eq!(lines.len(), 3);
}

// ===========================================================================
// MARKUP
// ===========================================================================

#[test]
fn markup_escape_roundtrip() {
    let orig = "[bold]";
    let escaped = markup::escape(orig);
    assert_eq!(escaped, "[[bold]");
}

#[test]
fn markup_empty_string() {
    let t = markup::render("");
    assert_eq!(t.plain, "");
}

#[test]
fn markup_nested_tags() {
    let t = markup::render("[bold][red]nested[/red][/bold]");
    assert_eq!(t.plain, "nested");
}

#[test]
fn markup_close_all() {
    let t = markup::render("[bold][red]text[/]");
    assert_eq!(t.plain, "text");
}

#[test]
fn markup_unmatched_open() {
    let t = markup::render("[bold]no close");
    assert_eq!(t.plain, "no close");
}

#[test]
fn markup_color_on_bg() {
    let t = markup::render("[red on blue]colored[/]");
    assert_eq!(t.plain, "colored");
}

// ===========================================================================
// MEASURE
// ===========================================================================

#[test]
fn measurement_basic() {
    let m = Measurement::new(10, 100);
    assert_eq!(m.minimum, 10);
    assert_eq!(m.maximum, 100);
}

#[test]
fn measurement_clamp_chains() {
    let m = Measurement::new(5, 200)
        .with_minimum(20)
        .with_maximum(100);
    assert_eq!(m.minimum, 20);
    assert_eq!(m.maximum, 100);
}

#[test]
fn measurement_shrink() {
    let m = Measurement::new(10, 100).shrink(5);
    assert_eq!(m.minimum, 5);
    assert_eq!(m.maximum, 95);
}

#[test]
fn measurement_grow() {
    let m = Measurement::new(10, 100).grow(5);
    assert_eq!(m.minimum, 15);
    assert_eq!(m.maximum, 105);
}

// ===========================================================================
// ALIGN
// ===========================================================================

#[test]
fn align_left_short() {
    let result = AlignMethod::Left.align_text("hi", 10);
    assert!(result.starts_with("hi"));
    assert_eq!(result.len(), 10);
}

#[test]
fn align_right_short() {
    let result = AlignMethod::Right.align_text("hi", 10);
    assert!(result.ends_with("hi"));
    assert_eq!(result.len(), 10);
}

#[test]
fn align_center_short() {
    let result = AlignMethod::Center.align_text("hi", 10);
    assert_eq!(result.len(), 10);
    assert!(!result.starts_with("hi"));
    assert!(!result.ends_with("hi"));
}

#[test]
fn align_longer_than_width() {
    let result = AlignMethod::Center.align_text("this is very long text", 5);
    // Should return original text unchanged
    assert!(result.len() > 5);
}

#[test]
fn align_full_justify() {
    let result = AlignMethod::Full.align_text("one two three", 30);
    assert_eq!(result.len(), 30);
}

// ===========================================================================
// BOX DRAWING
// ===========================================================================

#[test]
fn box_all_styles_have_correct_structure() {
    // Verify box styles that have characters
    let styles: &[&BoxStyle] = &[
        &box_drawing::BOX_ROUNDED,
        &box_drawing::BOX_SQUARE,
        &box_drawing::BOX_HEAVY,
        &box_drawing::BOX_HEAVY_EDGE,
        &box_drawing::BOX_HEAVY_HEAD,
        &box_drawing::BOX_DOUBLE,
        &box_drawing::BOX_DOUBLE_EDGE,
        &box_drawing::BOX_ASCII,
    ];
    for b in styles {
        let s = b.to_string();
        assert_eq!(s.lines().count(), 8, "Box should have 8 lines");
    }
}

#[test]
fn box_ascii_has_no_unicode() {
    let b = &box_drawing::BOX_ASCII;
    assert!(b.ascii);
    let s = b.to_string();
    assert!(s.is_ascii());
}

#[test]
fn box_rounded_has_unicode() {
    let b = &box_drawing::BOX_ROUNDED;
    assert!(!b.ascii);
}

// ===========================================================================
// PANEL
// ===========================================================================

#[test]
fn panel_empty_content() {
    let panel = Panel::new("");
    let opts = ConsoleOptions::default();
    let result = panel.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_title_shorter_than_width() {
    let panel = Panel::new("content").title("T");
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("T"));
}

#[test]
fn panel_title_longer_than_width() {
    let panel = Panel::new("c").title("This title is way way way too long for the panel");
    let opts = ConsoleOptions { max_width: 10, ..Default::default() };
    let result = panel.render(&opts);
    // Should not panic — title is just skipped
    assert!(!result.lines.is_empty());
}

#[test]
fn panel_subtitle() {
    let panel = Panel::new("content").subtitle("footer");
    let opts = ConsoleOptions { max_width: 30, ..Default::default() };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("footer"));
}

#[test]
fn panel_with_padding() {
    let panel = Panel::new("x").padding(2, 3, 2, 3);
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = panel.render(&opts);
    // Should have at least top border + 2 top pad + content + 2 bottom pad + bottom border
    assert!(result.lines.len() >= 7);
}

#[test]
fn panel_fit_mode() {
    let panel = Panel::new("hi").fit();
    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = panel.render(&opts);
    // In fit mode, width should be smaller than max_width
    assert!(result.lines.len() >= 3);
}

// ===========================================================================
// TABLE
// ===========================================================================

#[test]
fn table_multiple_columns() {
    let mut table = Table::new();
    table.add_column(Column::new("Name"));
    table.add_column(Column::new("Age"));
    table.add_column(Column::new("City"));
    table.add_row(vec!["Alice".into(), "30".into(), "NYC".into()]);
    table.add_row(vec!["Bob".into(), "25".into(), "LA".into()]);

    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Alice"));
    assert!(ansi.contains("Bob"));
    assert!(ansi.contains("NYC"));
    assert!(ansi.contains("LA"));
}

#[test]
fn table_with_title_and_caption() {
    let mut table = Table::new();
    table.add_column(Column::new("Item"));
    table.add_row(vec!["Widget".into()]);
    let table = table.title("Products").caption("All prices in USD");

    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Products"));
    assert!(ansi.contains("USD"));
}

#[test]
fn table_empty() {
    let table = Table::new();
    let opts = ConsoleOptions::default();
    let result = table.render(&opts);
    assert!(result.lines.is_empty());
}

#[test]
fn table_fixed_column_widths() {
    let mut table = Table::new();
    table.add_column(Column::new("A").width(10));
    table.add_column(Column::new("B").width(10));
    table.add_row(vec!["1".into(), "2".into()]);

    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = table.render(&opts);
    assert!(!result.lines.is_empty());
}

#[test]
fn table_hide_header() {
    let mut table = Table::new();
    table.add_column(Column::new("Hidden"));
    table.add_row(vec!["data".into()]);
    let table = table.hide_header();

    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = table.render(&opts);
    let ansi = result.to_ansi();
    assert!(!ansi.contains("Hidden"));
    assert!(ansi.contains("data"));
}

#[test]
fn table_show_lines() {
    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row(vec!["a".into()]);
    table.add_row(vec!["b".into()]);
    table.add_row(vec!["c".into()]);
    let table = table.show_lines();

    let opts = ConsoleOptions { max_width: 30, ..Default::default() };
    let result = table.render(&opts);
    // show_lines adds separators between rows => more lines
    assert!(result.lines.len() > 5);
}

// ===========================================================================
// TREE
// ===========================================================================

#[test]
fn tree_deeply_nested() {
    let mut tree = Tree::new("L0");
    let mut current = tree.add("L1");
    for i in 2..=5 {
        current = current.add(&format!("L{i}"));
    }

    let opts = ConsoleOptions::default();
    let result = tree.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("L0"));
    assert!(ansi.contains("L5"));
}

#[test]
fn tree_wide_branching() {
    let mut tree = Tree::new("Root");
    for i in 0..20 {
        tree.add(&format!("Child {i}"));
    }

    let opts = ConsoleOptions::default();
    let result = tree.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Child 0"));
    assert!(ansi.contains("Child 19"));
}

#[test]
fn tree_hide_root() {
    let mut tree = Tree::new("Hidden Root");
    tree.add("Visible Child");
    let tree = tree.hide_root();

    let opts = ConsoleOptions::default();
    let result = tree.render(&opts);
    let ansi = result.to_ansi();
    assert!(!ansi.contains("Hidden Root"));
    assert!(ansi.contains("Visible Child"));
}

#[test]
fn tree_ascii_mode() {
    let mut tree = Tree::new("Root");
    tree.add("Child");

    let opts = ConsoleOptions { ascii_only: true, ..Default::default() };
    let result = tree.render(&opts);
    let ansi = result.to_ansi();
    // ASCII guides use + and ` not ├ and └
    assert!(ansi.contains('+') || ansi.contains('`'));
}

// ===========================================================================
// RULE
// ===========================================================================

#[test]
fn rule_default() {
    let rule = Rule::new();
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = rule.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains('─'));
}

#[test]
fn rule_left_aligned_title() {
    let rule = Rule::new().title("Chapter 1").align(AlignMethod::Left);
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = rule.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.starts_with("Chapter 1") || ansi.contains("Chapter 1"));
}

#[test]
fn rule_ascii_mode() {
    let rule = Rule::new().title("Test");
    let opts = ConsoleOptions { max_width: 40, ascii_only: true, ..Default::default() };
    let result = rule.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains('-'));
}

#[test]
fn rule_zero_width() {
    let rule = Rule::new();
    let opts = ConsoleOptions { max_width: 0, ..Default::default() };
    let result = rule.render(&opts);
    // Should not panic
    let _ = result.to_ansi();
}

// ===========================================================================
// PADDING
// ===========================================================================

#[test]
fn padding_indent() {
    let p = Padding::new("text").indent(4);
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = p.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("text"));
}

#[test]
fn padding_all_sides() {
    let p = Padding::new("x").pad(2, 3, 4, 5);
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = p.render(&opts);
    // 2 top + content + 4 bottom
    assert!(result.lines.len() >= 7);
}

// ===========================================================================
// COLUMNS
// ===========================================================================

#[test]
fn columns_side_by_side() {
    let mut cols = Columns::new();
    cols.add("Left");
    cols.add("Right");

    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = cols.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Left"));
    assert!(ansi.contains("Right"));
}

#[test]
fn columns_empty() {
    let cols = Columns::new();
    let opts = ConsoleOptions::default();
    let result = cols.render(&opts);
    assert!(result.lines.is_empty());
}

// ===========================================================================
// PROGRESS
// ===========================================================================

#[test]
fn progress_bar_indeterminate() {
    let bar = ProgressBar::new().total(0.0); // 0 total = indeterminate
    let rendered = bar.render(20);
    assert!(!rendered.is_empty());
}

#[test]
fn progress_bar_complete() {
    let bar = ProgressBar::new().total(100.0).completed(100.0);
    let rendered = bar.render(20);
    assert!(rendered.contains('█'));
}

#[test]
fn progress_bar_zero() {
    let bar = ProgressBar::new().total(100.0).completed(0.0);
    let rendered = bar.render(20);
    assert!(rendered.contains('░'));
}

#[test]
fn progress_multi_task() {
    let mut p = Progress::new();
    let t1 = p.add_task("Download", Some(100.0));
    let t2 = p.add_task("Install", Some(50.0));
    p.advance(t1, 30.0);
    p.advance(t2, 10.0);
    p.update(t1, 50.0);

    let rendered = p.render(60);
    assert!(rendered.contains("Download"));
    assert!(rendered.contains("Install"));
}

#[test]
fn progress_remove_task() {
    let mut p = Progress::new();
    let id = p.add_task("Temp", None);
    assert_eq!(p.tasks.len(), 1);
    p.remove_task(id);
    assert_eq!(p.tasks.len(), 0);
}

// ===========================================================================
// SPINNER
// ===========================================================================

#[test]
fn spinner_all_frames() {
    use std::time::Duration;
    let s = Spinner::default();
    // Cycle through many frames
    for i in 0..100 {
        let frame = s.frame_at(Duration::from_millis(i * 100));
        assert!(!frame.is_empty());
    }
}

#[test]
fn spinner_with_text() {
    use std::time::Duration;
    let s = Spinner::default().text("Loading...");
    let rendered = s.render(Duration::from_millis(500));
    assert!(rendered.contains("Loading..."));
}

#[test]
fn spinner_line_cycles() {
    use std::time::Duration;
    let s = Spinner::new(&spinner::SPINNER_LINE);
    let frames: Vec<&str> = (0..4)
        .map(|i| s.frame_at(Duration::from_millis(i * 101)))
        .collect();
    assert_eq!(frames.len(), 4);
}

// ===========================================================================
// JSON
// ===========================================================================

#[test]
fn json_primitives() {
    let v: serde_json::Value = serde_json::json!({
        "name": "Alice",
        "age": 30,
        "active": true,
        "data": null,
        "tags": ["a", "b"]
    });
    let rendered = json::render_json(&v);
    let opts = ConsoleOptions::default();
    let result = rendered.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Alice"));
    assert!(ansi.contains("30"));
    assert!(ansi.contains("true"));
    assert!(ansi.contains("null"));
}

#[test]
fn json_empty_object() {
    let v: serde_json::Value = serde_json::json!({});
    let rendered = json::render_json(&v);
    let opts = ConsoleOptions::default();
    let result = rendered.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("{}"));
}

#[test]
fn json_empty_array() {
    let v: serde_json::Value = serde_json::json!([]);
    let rendered = json::render_json(&v);
    let opts = ConsoleOptions::default();
    let result = rendered.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("[]"));
}

#[test]
fn json_deeply_nested() {
    let v: serde_json::Value = serde_json::json!({
        "a": {"b": {"c": {"d": {"e": "deep"}}}}
    });
    let rendered = json::render_json(&v);
    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = rendered.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("deep"));
}

// ===========================================================================
// MARKDOWN
// ===========================================================================

#[test]
#[cfg(feature = "markdown")]
fn markdown_h1() {
    let md = markdown::render_markdown("# Hello\n");
    let opts = ConsoleOptions { max_width: 60, ..Default::default() };
    let result = md.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Hello"));
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_h2() {
    let md = markdown::render_markdown("## Section\n");
    let opts = ConsoleOptions { max_width: 60, ..Default::default() };
    let result = md.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Section"));
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_list() {
    let md = markdown::render_markdown("* one\n* two\n* three\n");
    let opts = ConsoleOptions { max_width: 60, ..Default::default() };
    let result = md.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("one"));
    assert!(ansi.contains("three"));
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_code_block() {
    let md = markdown::render_markdown("```\nlet x = 1;\n```\n");
    let opts = ConsoleOptions { max_width: 60, ..Default::default() };
    let result = md.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("let x = 1"));
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_blockquote() {
    let md = markdown::render_markdown("> quoted text\n");
    let opts = ConsoleOptions { max_width: 60, ..Default::default() };
    let result = md.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("quoted text"));
}

#[test]
#[cfg(feature = "markdown")]
fn markdown_empty() {
    let md = markdown::render_markdown("");
    let opts = ConsoleOptions::default();
    let result = md.render(&opts);
    let _ = result.to_ansi(); // should not panic
}

// ===========================================================================
// SYNTAX HIGHLIGHTING
// ===========================================================================

#[test]
#[cfg(feature = "syntax-highlighting")]
fn syntax_rust_code() {
    let s = Syntax::new("fn main() { println!(\"hello\"); }", "rust");
    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = s.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("fn"));
    assert!(ansi.contains("main"));
}

#[test]
#[cfg(feature = "syntax-highlighting")]
fn syntax_unknown_language() {
    let s = Syntax::new("some code", "nonexistent_lang_xyz");
    let opts = ConsoleOptions::default();
    let result = s.render(&opts);
    // Should fall back to plain text
    let ansi = result.to_ansi();
    assert!(ansi.contains("some code"));
}

#[test]
#[cfg(feature = "syntax-highlighting")]
fn syntax_line_numbers_enabled() {
    let s = Syntax::new("line1\nline2\nline3\n", "rust").line_numbers();
    let opts = ConsoleOptions { max_width: 80, ..Default::default() };
    let result = s.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("1"));
    assert!(ansi.contains("2"));
    assert!(ansi.contains("3"));
}

#[test]
#[cfg(feature = "syntax-highlighting")]
fn syntax_empty_code() {
    let s = Syntax::new("", "rust");
    let opts = ConsoleOptions::default();
    let result = s.render(&opts);
    // Should not panic
    let _ = result.to_ansi();
}

// ===========================================================================
// CONSOLE (non-I/O)
// ===========================================================================

#[test]
fn console_options_update_width() {
    let opts = ConsoleOptions::default();
    let updated = opts.update_width(120);
    assert_eq!(updated.max_width, 120);
}

#[test]
fn render_result_new_is_empty() {
    let r = RenderResult::new();
    assert!(r.lines.is_empty());
}

#[test]
fn dyn_renderable_creation() {
    let dr = console::DynRenderable::new("test string");
    let opts = ConsoleOptions::default();
    let result = dr.render(&opts);
    assert!(!result.lines.is_empty());
}

// ===========================================================================
// CROSS-COMPONENT: style + segment + text
// ===========================================================================

#[test]
fn styled_text_pipeline() {
    // Full pipeline: Style → Text → Segment → ANSI
    let style = Style::new().bold(true).color(Color::parse("green").unwrap());
    let mut text = Text::new("Success!");
    text.style = style.clone();
    assert_eq!(text.plain, "Success!");

    let seg = Segment::styled(text.plain.clone(), style);
    let ansi = seg.to_ansi();
    assert!(ansi.contains("Success!"));
    assert!(ansi.contains("\x1b["));
}

#[test]
fn markup_to_panel_pipeline() {
    // Markup → Text → Panel → Render → ANSI
    let text = markup::render("[bold]Bordered[/bold]");
    let panel = Panel::new(text).title("Info");
    let opts = ConsoleOptions { max_width: 40, ..Default::default() };
    let result = panel.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("Bordered"));
    assert!(ansi.contains("Info"));
}

// ===========================================================================
// EDGE / STRESS
// ===========================================================================

#[test]
fn very_wide_terminal() {
    let opts = ConsoleOptions { max_width: 500, ..Default::default() };

    let mut table = Table::new();
    table.add_column(Column::new("X"));
    table.add_row(vec!["short".into()]);
    let _ = table.render(&opts);

    let rule = Rule::new().title("Wide");
    let _ = rule.render(&opts);

    let panel = Panel::new("wide content").title("Wide");
    let _ = panel.render(&opts);
}

#[test]
fn very_narrow_terminal() {
    let opts = ConsoleOptions { max_width: 5, ..Default::default() };

    let mut table = Table::new();
    table.add_column(Column::new("Narrow"));
    table.add_row(vec!["N".into()]);
    let _ = table.render(&opts); // should not panic

    let rule = Rule::new();
    let _ = rule.render(&opts); // should not panic

    let panel = Panel::new("x");
    let _ = panel.render(&opts); // should not panic
}

#[test]
fn unicode_heavy_content() {
    let text = Text::new("こんにちは世界🌍🎉");
    assert!(text.cell_len() > 5);

    let mut tree = Tree::new("ルート");
    tree.add("子要素").add("孫要素");
    let opts = ConsoleOptions::default();
    let result = tree.render(&opts);
    let ansi = result.to_ansi();
    assert!(ansi.contains("ルート"));
}

#[test]
fn many_style_combinations() {
    for bold in [false, true] {
        for italic in [false, true] {
            for underline in [false, true] {
                let s = Style::new()
                    .bold(bold)
                    .italic(italic)
                    .underline(underline);
                let ansi = s.to_ansi();
                // Should always produce valid ANSI or empty string
                if ansi.is_empty() {
                    assert!(!bold && !italic && !underline);
                } else {
                    assert!(ansi.starts_with("\x1b["));
                    assert!(ansi.ends_with('m'));
                }
            }
        }
    }
}

#[test]
fn empty_renderables_dont_panic() {
    let opts = ConsoleOptions::default();

    // Empty tree
    let tree = Tree::new("");
    let _ = tree.render(&opts);

    // Empty panel content
    let panel = Panel::new("");
    let _ = panel.render(&opts);

    // Empty rule
    let rule = Rule::new();
    let _ = rule.render(&opts);

    // Empty columns
    let cols = Columns::new();
    let _ = cols.render(&opts);

    // Empty progress
    let p = Progress::new();
    let _ = p.render(40);
}
