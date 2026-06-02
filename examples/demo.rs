//! rusty-rich end-to-end demo — showcases every major feature.
//!
//! Usage:  cargo run --example demo

use std::thread::sleep;
use std::time::{Duration, Instant};

use rusty_rich::*;

fn main() {
    let mut console = Console::new();

    // ── Title banner ──────────────────────────────────────────────────────
    console.clear();
    let mut title_text = Text::new("");
    title_text.append_styled("rusty-rich", Style::new().bold(true).color(Color::parse("bright_cyan").unwrap()));
    title_text.append_styled(" v0.4.0", Style::new().dim(true));

    let banner = Panel::new(title_text)
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_EDGE))
        .border_style(Style::new().color(Color::parse("green").unwrap()))
        .title(" 🦀 Rich Terminal Toolkit ")
        .title_align(AlignMethod::Center);
    console.println(&banner);

    // ── Console markup ────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Console Markup")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    console.print_str("Styles: [bold]bold[/bold]  [dim]dim[/dim]  [italic]italic[/italic]  [underline]underline[/underline]  [blink]blink[/blink]  [reverse]reverse[/reverse]  [strike]strike[/strike]\n");
    console.print_str("Colors: [red]red[/red]  [green]green[/green]  [blue]blue[/blue]  [yellow]yellow[/yellow]  [magenta]magenta[/magenta]  [cyan]cyan[/cyan]\n");
    console.print_str("Backgrounds: [on red]on red[/on red]  [on green]on green[/on green]  [on blue]on blue[/on blue]\n");
    console.print_str("Combined:  [bold red on bright_black]bold red on bright black[/]\n");
    console.print_str("Links:     [link=https://github.com/textualize/rich]Rich on GitHub[/link]\n");
    console.print_str("\n");

    // ── Tables ────────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Tables")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut table = Table::new();
    table.add_column(
        Column::new("Language")
            .justify(AlignMethod::Left)
            .header_style(Style::new().bold(true).color(Color::parse("cyan").unwrap())),
    );
    table.add_column(
        Column::new("Created")
            .justify(AlignMethod::Center),
    );
    table.add_column(
        Column::new("Typing")
            .justify(AlignMethod::Center),
    );
    table.add_column(
        Column::new("Performance")
            .justify(AlignMethod::Right),
    );
    table.add_row(vec![
        "Rust".into(), "2010".into(), "Static".into(), "★★★★★".into(),
    ]);
    table.add_row(vec![
        "Python".into(), "1991".into(), "Dynamic".into(), "★★★☆☆".into(),
    ]);
    table.add_row(vec![
        "Go".into(), "2009".into(), "Static".into(), "★★★★☆".into(),
    ]);
    table.add_row(vec![
        "TypeScript".into(), "2012".into(), "Gradual".into(), "★★★☆☆".into(),
    ]);
    let table = table
        .title("Programming Languages")
        .caption("Source: community consensus")
        .border_style(Style::new().color(Color::parse("bright_black").unwrap()));
    console.println(&table);
    console.print_str("\n");

    // ── Tree ──────────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Tree (Application Structure)")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut tree = Tree::new("my-app/".to_string());
    {
        let src = tree.add("src/");
        src.add("main.rs");
        let lib = src.add("lib.rs");
        lib.add("mod config;");
        lib.add("mod handlers;");
        src.add("config.rs");
        src.add("handlers.rs");
    }
    {
        let tests = tree.add("tests/");
        tests.add("integration_test.rs");
        tests.add("unit_tests.rs");
    }
    tree.add("Cargo.toml");
    tree.add("Cargo.lock");
    tree.add("README.md");
    console.println(&tree);
    console.print_str("\n");

    // ── Panel ─────────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Panels")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut info_text = Text::new("");
    info_text.append_styled("⚠ ", Style::new().bold(true).color(Color::parse("bright_yellow").unwrap()));
    info_text.append("This library is a faithful Rust port of the Python Rich library by Textualize. ", None);
    info_text.append("It runs natively, with zero Python dependencies.", None);

    let info = Panel::new(info_text)
        .title("About")
        .box_style(BoxStyle::clone(&box_drawing::BOX_ROUNDED))
        .border_style(Style::new().color(Color::parse("bright_cyan").unwrap()))
        .padding(1, 2, 1, 2);
    console.println(&info);
    console.print_str("\n");

    // ── Layout ────────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Columns (Side-by-Side Panels)")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let mut cols = Columns::new();
    cols.add(
        Panel::new("Rust is blazingly fast\nand memory-efficient.")
            .title("🦀 Rust")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_red").unwrap()))
            .padding(1, 2, 1, 2),
    );
    cols.add(
        Panel::new("Rich makes beautiful\nterminal output easy.")
            .title("🎨 Rich")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_green").unwrap()))
            .padding(1, 2, 1, 2),
    );
    cols.add(
        Panel::new("Combined = rusty-rich\nBest of both worlds!")
            .title("💎 Combined")
            .box_style(BoxStyle::clone(&box_drawing::BOX_SQUARE))
            .border_style(Style::new().color(Color::parse("bright_blue").unwrap()))
            .padding(1, 2, 1, 2),
    );
    console.println(&cols);
    console.print_str("\n");

    // ── JSON ──────────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("JSON Pretty Print")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let data: serde_json::Value = serde_json::json!({
        "name": "rusty-rich",
        "version": "0.1.0",
        "description": "Rich terminal output in Rust",
        "dependencies": {
            "syntect": "5.1",
            "pulldown-cmark": "0.10",
            "serde_json": "1.0"
        },
        "features": ["tables", "trees", "syntax", "markdown", "progress"],
        "active": true,
        "contributors": null
    });
    let json_render = json::render_json(&data);
    console.println(&json_render);
    console.print_str("\n");

    // ── Syntax Highlighting ──────────────────────────────────────────────
    let rule = Rule::new()
        .title("Syntax Highlighting")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let rust_code = r#"use std::collections::HashMap;

/// A simple cache backed by a HashMap.
pub struct Cache<K, V> {
    store: HashMap<K, V>,
    max_size: usize,
}

impl<K: Eq + Hash, V> Cache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            store: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.store.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.store.len() >= self.max_size {
            // Evict a random entry if full
            let _ = self.store.drain().next();
        }
        self.store.insert(key, value);
    }
}
"#;

    let syntax = Syntax::new(rust_code, "rust")
        .theme("base16-ocean.dark");
    let panel = Panel::new(syntax)
        .title(" src/cache.rs ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_HEAD))
        .border_style(Style::new().color(Color::parse("bright_black").unwrap()))
        .padding(0, 2, 0, 2);
    console.println(&panel);
    console.print_str("\n");

    // ── Markdown ──────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Markdown Rendering")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let markdown_src = r#"# Hello from rusty-rich!

This is a **Markdown** renderer built in Rust.

## Features

* Renders headings (H1–H6)
* Supports **bold**, *italic*, and `inline code`
* Fenced code blocks with language tags
* Block quotes for important notes
* Nested bullet lists

## Code Example

```rust
fn main() {
    println!("Hello, world!");
}
```

> **Note:** This is rendered using `pulldown-cmark`, a fast, spec-compliant CommonMark parser.

---

Made with ❤️ using Rust.
"#;

    let md = markdown::render_markdown(markdown_src);
    let panel = Panel::new(md)
        .title(" README.md ")
        .box_style(BoxStyle::clone(&box_drawing::BOX_ROUNDED))
        .border_style(Style::new().color(Color::parse("bright_magenta").unwrap()))
        .padding(1, 3, 1, 3);
    console.println(&panel);
    console.print_str("\n");

    // ── Progress ──────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Progress Bars")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    println!("Simulating a 3-step pipeline...\n");

    // We render progress manually here (the library supports it)
    let steps = ["Downloading dependencies", "Compiling source", "Running tests"];
    for (i, step) in steps.iter().enumerate() {
        let total_steps = steps.len() as f64;
        let pct = (i as f64 + 0.42) / total_steps;

        let bar = ProgressBar::new()
            .total(1.0)
            .completed(pct)
            .complete_style(Style::new().color(Color::parse("green").unwrap()))
            .remaining_style(Style::new().color(Color::parse("bright_black").unwrap()));

        let label = format!(" {:<30}", step);
        let rendered = bar.render(40);
        println!("{label} {rendered}");

        sleep(Duration::from_millis(300));
    }
    // Final: all complete
    let bar = ProgressBar::new()
        .total(1.0)
        .completed(1.0)
        .complete_style(Style::new().color(Color::parse("bright_green").unwrap()))
        .remaining_style(Style::new().color(Color::parse("bright_black").unwrap()));
    let label = format!(" {:<30}", "Done!");
    let rendered = bar.render(40);
    println!("{label} {rendered}");
    println!();

    // ── Spinner ───────────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Spinner Demo (animated, 2 seconds)")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    println!("Initializing...");
    let spinner = Spinner::new(&spinner::SPINNER_DOTS);
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        let rendered = spinner.render(start.elapsed());
        print!("\r  {} {}", rendered, "Working...            ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        sleep(Duration::from_millis(80));
    }
    // Clear the spinner line
    print!("\r\x1b[K");
    // Final success
    let check = Style::new()
        .color(Color::parse("bright_green").unwrap())
        .bold(true)
        .to_ansi();
    println!("  {check}✓ Done!{}\n", Style::new().reset_ansi());

    // ── Rule styles ───────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Rule Variants")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    for align in &[AlignMethod::Left, AlignMethod::Center, AlignMethod::Right] {
        let r = Rule::new()
            .title(&format!("{align} aligned"))
            .align(*align)
            .style(Style::new().color(Color::parse("bright_black").unwrap()));
        console.println(&r);
    }
    let r = Rule::new()
        .characters("━")
        .style(Style::new().color(Color::parse("bright_cyan").unwrap()));
    console.println(&r);

    // ── Color palette ─────────────────────────────────────────────────────
    let rule = Rule::new()
        .title("Color Palette (Standard 16)")
        .style(Style::new().color(Color::parse("yellow").unwrap()));
    console.println(&rule);

    let color_names = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        "bright_black", "bright_red", "bright_green", "bright_yellow",
        "bright_blue", "bright_magenta", "bright_cyan", "bright_white",
    ];
    for chunk in color_names.chunks(8) {
        let mut line = String::new();
        for name in chunk {
            let style = Style::new()
                .color(Color::parse(name).unwrap())
                .bold(true);
            let bg_style = Style::new()
                .bgcolor(Color::parse(name).unwrap());
            line.push_str(&format!(
                "{}{:>15}{} ",
                style.to_ansi(), name, Style::new().reset_ansi()
            ));
            line.push_str(&format!(
                "{}{}{} ",
                bg_style.to_ansi(), "   ", Style::new().reset_ansi()
            ));
        }
        println!("  {line}");
    }
    println!();

    // ── Footer ────────────────────────────────────────────────────────────
    let mut footer_text = Text::new("");
    footer_text.append_styled("rusty-rich", Style::new().bold(true).color(Color::parse("cyan").unwrap()));
    footer_text.append(" — Rust port of ", None);
    footer_text.append_styled("Textualize/rich", Style::new().underline(true));
    footer_text.append(" — 20 modules, 41+ unit tests, 103+ integration tests", None);

    let footer = Panel::new(footer_text)
        .box_style(BoxStyle::clone(&box_drawing::BOX_HEAVY_EDGE))
        .border_style(Style::new().color(Color::parse("green").unwrap()))
        .title(" ✓ End of Demo ")
        .title_align(AlignMethod::Center);
    console.println(&footer);
}
