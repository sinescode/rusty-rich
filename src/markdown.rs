//! Markdown rendering — equivalent to Rich's `markdown.py`.
//!
//! Uses `pulldown-cmark` for parsing and renders headings, code blocks,
//! lists, tables, blockquotes, and inline formatting.

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::align::AlignMethod;
use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::rule::Rule;
use crate::segment::Segment;
use crate::style::Style;
use crate::table::{Cell, Column, Table};

/// Render markdown text.
pub fn render_markdown(md: &str) -> MarkdownRender {
    MarkdownRender {
        source: md.to_string(),
        width: None,
        code_theme: "default".to_string(),
        hyperlinks: true,
    }
}

/// Renders markdown text to styled terminal output via [`Renderable`].
#[derive(Debug, Clone)]
pub struct MarkdownRender {
    source: String,
    width: Option<usize>,
    code_theme: String,
    hyperlinks: bool,
}

impl MarkdownRender {
    /// Set a fixed rendering width (defaults to the console width).
    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Set the code syntax-highlighting theme (default: `"default"`).
    pub fn code_theme(mut self, theme: impl Into<String>) -> Self {
        self.code_theme = theme.into();
        self
    }

    /// Enable or disable hyperlink rendering (default: true).
    pub fn hyperlinks(mut self, enabled: bool) -> Self {
        self.hyperlinks = enabled;
        self
    }

    fn get_style(name: &str) -> Style {
        use crate::theme::default_theme;
        let theme = default_theme();
        theme.get(name).cloned().unwrap_or(Style::new())
    }

    /// Look up a code-block style that respects `self.code_theme`.
    fn code_style(&self) -> Style {
        use crate::theme::default_theme;
        let theme = default_theme();
        let key = format!("markdown.code.{}", self.code_theme);
        theme
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Self::get_style("markdown.code"))
    }
}

impl Renderable for MarkdownRender {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let width = self.width.unwrap_or(options.max_width);
        let parser = Parser::new_ext(&self.source, Options::all());

        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let mut current_line: Vec<Segment> = Vec::new();
        let mut in_code_block = false;
        let mut heading_level = 0u8;
        let mut list_depth = 0usize;
        let mut current_link: Option<String> = None;
        let mut link_text: Option<String> = None;
        let mut in_table = false;
        let mut table_alignments: Vec<Alignment> = Vec::new();
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut _table_is_header = false;
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    heading_level = level as u8;
                    let style = match level {
                        HeadingLevel::H1 => Self::get_style("markdown.h1"),
                        HeadingLevel::H2 => Self::get_style("markdown.h2"),
                        _ => Style::new().bold(true),
                    };
                    let prefix = "#".repeat(level as usize);
                    current_line.push(Segment::styled(format!("{prefix} "), style.clone()));
                }
                Event::End(TagEnd::Heading(_)) => {
                    lines.push(current_line.clone());
                    current_line.clear();
                    // Add a rule under H1/H2
                    if heading_level <= 2 {
                        let rule_char = if heading_level == 1 { '═' } else { '─' };
                        let rule_line = rule_char.to_string().repeat(width);
                        lines.push(vec![Segment::new(rule_line), Segment::line()]);
                    }
                }
                Event::Start(Tag::Paragraph) => {}
                Event::End(TagEnd::Paragraph) => {
                    if !current_line.is_empty() {
                        current_line.push(Segment::line());
                        lines.push(current_line.clone());
                        current_line.clear();
                    }
                    // Add blank line after paragraph
                    lines.push(vec![Segment::line()]);
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    let lang = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() {
                                String::new()
                            } else {
                                lang.to_string()
                            }
                        }
                        CodeBlockKind::Indented => String::new(),
                    };
                    let title = if lang.is_empty() {
                        "Code".to_string()
                    } else {
                        format!("Code: {lang}")
                    };
                    // Code block opening
                    let code_style = self.code_style();
                    current_line.push(Segment::styled(format!("┌─ {title} "), code_style.clone()));
                    current_line.push(Segment::line());
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    if !current_line.is_empty() {
                        lines.push(current_line.clone());
                        current_line.clear();
                    }
                    let code_style = self.code_style();
                    lines.push(vec![
                        Segment::styled(
                            format!("└{}", "─".repeat(width.saturating_sub(2))),
                            code_style,
                        ),
                        Segment::line(),
                    ]);
                }
                Event::Start(Tag::List(_)) => {
                    list_depth += 1;
                }
                Event::End(TagEnd::List(_)) => {
                    list_depth = list_depth.saturating_sub(1);
                }
                Event::Start(Tag::Item) => {
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    let bullet = if list_depth > 1 { "◦" } else { "•" };
                    current_line.push(Segment::new(format!("{indent}{bullet} ")));
                }
                Event::End(TagEnd::Item) => {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                Event::Start(Tag::BlockQuote) => {
                    let quote_style = Self::get_style("markdown.blockquote");
                    current_line.push(Segment::styled("▌ ", quote_style));
                }
                Event::End(TagEnd::BlockQuote) => {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                Event::Start(Tag::Emphasis) => {
                    current_line.push(Segment::styled("", Style::new().italic(true)));
                }
                Event::End(TagEnd::Emphasis) => {
                    // Inline — handled via style stack
                }
                Event::Start(Tag::Strong) => {
                    // handled inline
                }
                Event::End(TagEnd::Strong) => {}
                Event::Start(Tag::Link { dest_url, .. }) => {
                    current_link = Some(dest_url.to_string());
                    link_text = Some(String::new());
                }
                Event::End(TagEnd::Link) => {
                    if let (Some(url), Some(text)) = (current_link.take(), link_text.take()) {
                        let link_style = Self::get_style("markdown.link");
                        let display = if text.is_empty() {
                            url.clone()
                        } else if self.hyperlinks {
                            format!("{text} ({url})")
                        } else {
                            text
                        };
                        current_line.push(Segment::styled(display, link_style));
                    }
                }
                Event::Text(text) | Event::Code(text) => {
                    let s: &str = &text;
                    if in_table {
                        current_cell_text.push_str(s);
                        // Also handle link text collection inside table cells
                        if current_link.is_some() {
                            if let Some(ref mut lt) = link_text {
                                lt.push_str(s);
                            }
                        }
                    } else {
                        // Collect link text if we're inside a link
                        if current_link.is_some() {
                            if let Some(ref mut lt) = link_text {
                                lt.push_str(s);
                            }
                        }
                        if in_code_block {
                            // Indent code
                            for line in s.lines() {
                                current_line.push(Segment::new(format!("│ {line}")));
                                current_line.push(Segment::line());
                                lines.push(current_line.clone());
                                current_line.clear();
                            }
                        } else {
                            current_line.push(Segment::new(s));
                        }
                    }
                }
                Event::SoftBreak => {
                    current_line.push(Segment::new(" "));
                }
                Event::HardBreak => {
                    current_line.push(Segment::line());
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                Event::Rule => {
                    let rule = Rule::new().characters("─");
                    let res = rule.render(options);
                    lines.extend(res.lines);
                }
                Event::Start(Tag::Table(alignments)) => {
                    in_table = true;
                    table_alignments = alignments;
                    table_rows = Vec::new();
                }
                Event::End(TagEnd::Table) => {
                    in_table = false;
                    if !table_rows.is_empty() {
                        let mut table = Table::new();
                        table.show_header = false;
                        table.show_edge = true;
                        for align in &table_alignments {
                            let justify = match align {
                                Alignment::Left => AlignMethod::Left,
                                Alignment::Right => AlignMethod::Right,
                                Alignment::Center => AlignMethod::Center,
                                Alignment::None => AlignMethod::Left,
                            };
                            table.add_column(Column::new("").justify(justify));
                        }
                        for (i, row) in table_rows.iter().enumerate() {
                            let cells: Vec<Cell> = row
                                .iter()
                                .enumerate()
                                .map(|(_, c)| {
                                    if i == 0 {
                                        Cell::new(c.clone()).style(Style::new().bold(true))
                                    } else {
                                        Cell::new(c.clone())
                                    }
                                })
                                .collect();
                            table.add_row(cells);
                        }
                        let result = table.render(options);
                        lines.extend(result.lines);
                    }
                }
                Event::Start(Tag::TableHead) => {
                    _table_is_header = true;
                }
                Event::End(TagEnd::TableHead) => {
                    _table_is_header = false;
                }
                Event::Start(Tag::TableRow) => {
                    current_row = Vec::new();
                }
                Event::End(TagEnd::TableRow) => {
                    table_rows.push(current_row.clone());
                    current_row.clear();
                }
                Event::Start(Tag::Image {
                    dest_url, title, ..
                }) => {
                    // Render image as a styled placeholder with alt text and URL
                    let image_style = Self::get_style("markdown.image");
                    let title_str = if title.is_empty() {
                        String::new()
                    } else {
                        format!(" \"{title}\"")
                    };
                    let image_text = format!("🖼 [Image: {dest_url}{title_str}]");
                    current_line.push(Segment::styled(image_text, image_style));
                    current_line.push(Segment::line());
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                Event::End(TagEnd::Image) => {
                    // Image is self-contained; handled on Start
                }
                Event::Start(Tag::TableCell) => {
                    current_cell_text = String::new();
                }
                Event::End(TagEnd::TableCell) => {
                    current_row.push(current_cell_text.clone());
                    current_cell_text.clear();
                }
                _ => {}
            }
        }

        // Flush remaining
        if !current_line.is_empty() {
            current_line.push(Segment::line());
            lines.push(current_line);
        }

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_heading() {
        let md = render_markdown("# Hello\n\nWorld");
        let opts = ConsoleOptions::default();
        let result = md.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello"));
    }
}
