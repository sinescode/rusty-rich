//! Visual demo — renders every box style, table feature, and panel variation.
//! Usage: cargo run --example view_all

use rusty_rich::*;

fn main() {
    let mut console = Console::new();

    // =========================================================================
    // ALL 18 BOX STYLES — TABLES (every style works here)
    // =========================================================================
    let rule = Rule::new()
        .title(" ALL 18 BOX STYLES — Table ")
        .style(Style::new().bold(true).color(Color::parse("cyan").unwrap()));
    console.println(&rule);

    let all_box_styles: Vec<(&str, &box_drawing::BoxStyle)> = vec![
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
    ];

    for (name, bs) in &all_box_styles {
        let mut table = Table::new();
        table.add_column(Column::new("Item").justify(AlignMethod::Left));
        table.add_column(Column::new("Qty").justify(AlignMethod::Center));
        table.add_row_str(vec!["Widget".into(), "5".into()]);
        table.add_row_str(vec!["Gadget".into(), "12".into()]);
        let table = table
            .box_style(BoxStyle::clone(bs))
            .title(format!(" {name} "))
            .border_style(Style::new().dim(true));
        console.println(&table);
        console.print_str("\n");
    }

    // =========================================================================
    // EDGED BOX STYLES — PANELS (only styles with visible borders)
    // =========================================================================
    let rule = Rule::new()
        .title(" PANEL — Edged Box Styles ")
        .style(Style::new().bold(true).color(Color::parse("green").unwrap()));
    console.println(&rule);

    let edged_styles: Vec<(&str, &box_drawing::BoxStyle)> = vec![
        ("ROUNDED", &box_drawing::BOX_ROUNDED),
        ("SQUARE", &box_drawing::BOX_SQUARE),
        ("HEAVY", &box_drawing::BOX_HEAVY),
        ("HEAVY_EDGE", &box_drawing::BOX_HEAVY_EDGE),
        ("HEAVY_HEAD", &box_drawing::BOX_HEAVY_HEAD),
        ("DOUBLE", &box_drawing::BOX_DOUBLE),
        ("DOUBLE_EDGE", &box_drawing::BOX_DOUBLE_EDGE),
        ("ASCII", &box_drawing::BOX_ASCII),
        ("ASCII2", &box_drawing::BOX_ASCII2),
        ("SQUARE_DOUBLE_HEAD", &box_drawing::BOX_SQUARE_DOUBLE_HEAD),
        ("ASCII_DOUBLE_HEAD", &box_drawing::BOX_ASCII_DOUBLE_HEAD),
    ];

    for chunk in edged_styles.chunks(3) {
        let mut cols = Columns::new().equal();
        for (name, bs) in chunk {
            let panel = Panel::new(*name)
                .box_style(BoxStyle::clone(bs))
                .title(format!(" {name} "))
                .padding(1, 2, 1, 2);
            cols.add(panel);
        }
        console.println(&cols);
        console.print_str("\n");
    }

    // =========================================================================
    // PANEL FEATURES — Title & Subtitle Alignments
    // =========================================================================
    let rule = Rule::new()
        .title(" PANEL — Title Alignments ")
        .style(Style::new().bold(true).color(Color::parse("green").unwrap()));
    console.println(&rule);

    for align in &[AlignMethod::Left, AlignMethod::Center, AlignMethod::Right] {
        let panel = Panel::new(format!("Title aligned: {align}"))
            .title(format!(" {align} Title "))
            .title_align(*align)
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_blue").unwrap()))
            .padding(1, 2, 1, 2);
        console.println(&panel);
        console.print_str("\n");
    }

    // Subtitle alignments
    let rule = Rule::new()
        .title(" PANEL — Subtitle Alignments ")
        .style(Style::new().bold(true).color(Color::parse("green").unwrap()));
    console.println(&rule);

    for align in &[AlignMethod::Left, AlignMethod::Center, AlignMethod::Right] {
        let mut panel = Panel::new(format!("Subtitle aligned: {align}"))
            .subtitle(format!(" {align} Subtitle "))
            .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_HEAD))
            .border_style(Style::new().color(Color::parse("bright_magenta").unwrap()))
            .padding(1, 2, 1, 2);
        panel.subtitle_align = *align;
        console.println(&panel);
        console.print_str("\n");
    }

    // =========================================================================
    // PANEL — Colored Borders
    // =========================================================================
    let rule = Rule::new()
        .title(" PANEL — Colored Borders ")
        .style(Style::new().bold(true).color(Color::parse("green").unwrap()));
    console.println(&rule);

    let colors = ["red", "green", "yellow", "blue", "magenta", "cyan"];
    let mut cols = Columns::new().equal();
    for color_name in &colors {
        let panel = Panel::new(*color_name)
            .box_style(BoxStyle::clone(&box_drawing::BOX_ROUNDED))
            .border_style(Style::new().bold(true).color(Color::parse(color_name).unwrap()))
            .title(format!(" {color_name} "))
            .padding(1, 2, 1, 2);
        cols.add(panel);
    }
    console.println(&cols);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Colspan & Rowspan
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Colspan & Rowspan ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    // Colspan
    let mut t1 = Table::new();
    t1.add_column(Column::new("Product"));
    t1.add_column(Column::new("Details"));
    t1.add_column(Column::new("Price"));
    t1.add_row(vec![
        Cell::new("Laptop").colspan(1),
        Cell::new("16GB RAM, 512GB SSD").colspan(1),
        Cell::new("$999"),
    ]);
    t1.add_row(vec![Cell::new("FREE SHIPPING ON ALL ORDERS!").colspan(3)]);
    let t1 = t1
        .title(" Colspan Example ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_HEAD));
    console.println(&t1);
    console.print_str("\n");

    // Rowspan
    let mut t2 = Table::new();
    t2.add_column(Column::new("Category"));
    t2.add_column(Column::new("Item"));
    t2.add_column(Column::new("Price"));
    t2.add_row(vec![
        Cell::new("Electronics").rowspan(2),
        Cell::new("Laptop"),
        Cell::new("$999"),
    ]);
    t2.add_row_str(vec!["Phone".into(), "$699".into()]);
    t2.add_row(vec![
        Cell::new("Clothing").rowspan(2),
        Cell::new("Jacket"),
        Cell::new("$89"),
    ]);
    t2.add_row_str(vec!["Shoes".into(), "$59".into()]);
    let t2 = t2
        .title(" Rowspan Example ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE));
    console.println(&t2);
    console.print_str("\n");

    // Combined colspan + rowspan
    let mut t3 = Table::new();
    t3.add_column(Column::new("A"));
    t3.add_column(Column::new("B"));
    t3.add_column(Column::new("C"));
    t3.add_column(Column::new("D"));
    t3.add_row(vec![
        Cell::new("BIG CELL").colspan(2).rowspan(2),
        Cell::new("C1"),
        Cell::new("D1"),
    ]);
    t3.add_row_str(vec!["C2".into(), "D2".into()]);
    t3.add_row_str(vec!["A3".into(), "B3".into(), "C3".into(), "D3".into()]);
    let t3 = t3
        .title(" Combined colspan(2) + rowspan(2) ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_DOUBLE));
    console.println(&t3);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Column Alignments
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Column Alignments ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut ta = Table::new();
    ta.add_column(Column::new("Left").justify(AlignMethod::Left));
    ta.add_column(Column::new("Center").justify(AlignMethod::Center));
    ta.add_column(Column::new("Right").justify(AlignMethod::Right));
    ta.add_column(Column::new("Justified").justify(AlignMethod::Full));
    ta.add_row_str(vec!["short".into(), "mid".into(), "123".into(), "justified text goes here".into()]);
    ta.add_row_str(vec!["longer text".into(), "x".into(), "4567".into(), "another long para".into()]);
    let ta = ta
        .title(" Mixed Alignments ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_ROUNDED))
        .border_style(Style::new().dim(true));
    console.println(&ta);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Sections
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Sections ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut ts = Table::new();
    ts.add_column(Column::new("Phase"));
    ts.add_column(Column::new("Status"));
    ts.add_row_str(vec!["Planning".into(), "✓ Done".into()]);
    ts.add_row_str(vec!["Design".into(), "✓ Done".into()]);
    ts.add_section();
    ts.add_row_str(vec!["Development".into(), "⏳ In Progress".into()]);
    ts.add_row_str(vec!["Code Review".into(), "… Pending".into()]);
    ts.add_section();
    ts.add_row_str(vec!["Testing".into(), "… Pending".into()]);
    ts.add_row_str(vec!["Deploy".into(), "… Pending".into()]);
    let ts = ts
        .title(" Project Phases ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_EDGE))
        .border_style(Style::new().color(Color::parse("bright_cyan").unwrap()));
    console.println(&ts);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Show Lines
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Show Lines ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tl = Table::new();
    tl.add_column(Column::new("Name"));
    tl.add_column(Column::new("Score"));
    tl.add_column(Column::new("Rank"));
    tl.add_row_str(vec!["Alice".into(), "95".into(), "1st".into()]);
    tl.add_row_str(vec!["Bob".into(), "87".into(), "2nd".into()]);
    tl.add_row_str(vec!["Carol".into(), "82".into(), "3rd".into()]);
    let tl = tl
        .title(" Leaderboard ")
        .show_lines()
        .box_style(BoxStyle::clone(&box_drawing::BOX_SIMPLE))
        .border_style(Style::new().color(Color::parse("bright_green").unwrap()));
    console.println(&tl);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Row Styles (alternating)
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Alternating Row Styles ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tr = Table::new();
    tr.add_column(Column::new("#").justify(AlignMethod::Right));
    tr.add_column(Column::new("Name"));
    tr.add_column(Column::new("Value"));
    for i in 1..=6 {
        tr.add_row_str(vec![i.to_string(), format!("Item-{i}"), format!("${}", i * 10)]);
    }
    let tr = tr
        .title(" Inventory ")
        .row_styles(vec![
            Style::new(),
            Style::new().bgcolor(Color::parse("grey23").unwrap()),
        ])
        .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
        .border_style(Style::new().dim(true));
    console.println(&tr);
    console.print_str("\n");

    // =========================================================================
    // TABLE — With Footer
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Footer ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tf = Table::new();
    tf.add_column(Column {
        header: "Item".into(),
        footer: "TOTAL".into(),
        ..Column::new("Item")
    });
    tf.add_column(Column {
        header: "Qty".into(),
        footer: "7".into(),
        ..Column::new("Qty").justify(AlignMethod::Center)
    });
    tf.add_column(Column {
        header: "Price".into(),
        footer: "$249".into(),
        ..Column::new("Price").justify(AlignMethod::Right)
    });
    tf.add_row_str(vec!["Widget".into(), "3".into(), "$99".into()]);
    tf.add_row_str(vec!["Gadget".into(), "4".into(), "$150".into()]);
    tf.show_footer = true;
    let tf = tf
        .title(" Order Summary ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_HEAD));
    console.println(&tf);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Grid Mode (no borders)
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Grid Mode ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tg = Table::grid();
    tg.add_column(Column::new("Key"));
    tg.add_column(Column::new("Value"));
    tg.add_row_str(vec!["Host".into(), "localhost".into()]);
    tg.add_row_str(vec!["Port".into(), "8080".into()]);
    tg.add_row_str(vec!["Debug".into(), "true".into()]);
    console.println(&tg);
    console.print_str("\n");

    // =========================================================================
    // TABLE — Leading (blank rows between data)
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — Leading = 1 ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tld = Table::new();
    tld.add_column(Column::new("Step"));
    tld.add_column(Column::new("Action"));
    tld.add_row_str(vec!["1".into(), "Connect to DB".into()]);
    tld.add_row_str(vec!["2".into(), "Run migrations".into()]);
    tld.add_row_str(vec!["3".into(), "Start server".into()]);
    let tld = tld
        .title(" Deployment Steps ")
        .leading(1)
        .box_style(BoxStyle::clone(&box_drawing::BOX_ROUNDED));
    console.println(&tld);
    console.print_str("\n");

    // =========================================================================
    // PANEL — Nested (Panel inside Columns)
    // =========================================================================
    let rule = Rule::new()
        .title(" PANEL — Three Side-by-Side ")
        .style(Style::new().bold(true).color(Color::parse("green").unwrap()));
    console.println(&rule);

    let mut cols = Columns::new().equal();
    cols.add(
        Panel::new("Rust\nis\nfast!")
            .title(" 🦀 Rust ")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_red").unwrap()))
            .padding(1, 2, 1, 2),
    );
    cols.add(
        Panel::new("Rich\nterminal\noutput")
            .title(" 🎨 Rich ")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_green").unwrap()))
            .padding(1, 2, 1, 2),
    );
    cols.add(
        Panel::new("Best of\nboth\nworlds!")
            .title(" 💎 Combined ")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_blue").unwrap()))
            .padding(1, 2, 1, 2),
    );
    console.println(&cols);
    console.print_str("\n");

    // =========================================================================
    // PANEL — Styled Content (Markup inside Panel)
    // =========================================================================
    let rule = Rule::new()
        .title(" PANEL — Styled Content ")
        .style(Style::new().bold(true).color(Color::parse("green").unwrap()));
    console.println(&rule);

    let mut text = Text::new("");
    text.append_styled("bold red", Style::new().bold(true).color(Color::parse("red").unwrap()));
    text.append(" | ", None);
    text.append_styled("italic green", Style::new().italic(true).color(Color::parse("green").unwrap()));
    text.append(" | ", None);
    text.append_styled("underline blue", Style::new().underline(true).color(Color::parse("blue").unwrap()));
    text.append("\n", None);
    text.append_styled("dim text", Style::new().dim(true));
    text.append(" | ", None);
    text.append_styled("reverse video", Style::new().reverse(true));
    text.append(" | ", None);
    text.append_styled("strikethrough", Style::new().strike(true));

    let panel = Panel::new(text)
        .title(" Rich Text Inside Panel ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_DOUBLE))
        .border_style(Style::new().color(Color::parse("bright_magenta").unwrap()))
        .padding(1, 3, 1, 3);
    console.println(&panel);
    console.print_str("\n");

    // =========================================================================
    // TABLE — ASCII-only mode
    // =========================================================================
    let rule = Rule::new()
        .title(" TABLE — ASCII Only ")
        .style(Style::new().bold(true).color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tascii = Table::new();
    tascii.add_column(Column::new("Name"));
    tascii.add_column(Column::new("Value"));
    tascii.add_row_str(vec!["Color".into(), "Red".into()]);
    tascii.add_row_str(vec!["Size".into(), "Large".into()]);
    let tascii = tascii
        .title(" Config ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_ASCII_DOUBLE_HEAD));
    let opts = ConsoleOptions { max_width: 50, ascii_only: true, ..Default::default() };
    let result = tascii.render(&opts);
    console.print_str(&result.to_ansi());
    console.print_str("\n\n");

    // =========================================================================
    // FOOTER
    // =========================================================================
    let mut footer = Text::new("");
    footer.append_styled("✓ All 18 box styles, panels, and tables rendered successfully!",
        Style::new().bold(true).color(Color::parse("bright_green").unwrap()));

    let panel = Panel::new(footer)
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_EDGE))
        .border_style(Style::new().color(Color::parse("green").unwrap()))
        .title(" Done ")
        .title_align(AlignMethod::Center)
        .padding(1, 4, 1, 4);
    console.println(&panel);
}
