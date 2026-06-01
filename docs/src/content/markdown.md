# Markdown

`MarkdownRender` renders Markdown text as styled terminal output. It uses [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark) for parsing and supports headings, code blocks, lists, blockquotes, links, horizontal rules, and inline formatting.

```rust
use rusty_rich::{render_markdown, MarkdownRender};
use rusty_rich::Console;

let mut console = Console::new();
let md = render_markdown("# Hello\n\nThis is **rich** markdown.");
console.println(&md);
```

---

## render_markdown(text)

```rust
pub fn render_markdown(md: &str) -> MarkdownRender
```

The entry point. Takes a Markdown source string and returns a `MarkdownRender` builder with default settings:

- Width: auto (terminal width or parent container width)
- Code theme: `"default"`
- Hyperlinks: enabled

```rust
let md = render_markdown("Hello **world**");
console.println(&md);
```

---

## MarkdownRender

The builder struct returned by `render_markdown()`. Configures rendering options before display.

### width()

Override the output width. By default the renderable inherits the terminal width or parent container width.

```rust
let md = render_markdown("Some text")
    .width(60);
```

A narrower width causes text to wrap earlier; a wider width allows longer lines. This is useful when rendering Markdown inside a `Panel`, `Table`, or `Layout` where the available space is constrained.

---

## Supported Elements

### Headings (H1 -- H6)

All six heading levels are supported.

**H1 and H2** are rendered with styled prefixes and a horizontal rule beneath them:

- **H1**: `# ` prefix with `markdown.h1` style (bold + bright cyan in the default theme), followed by a double-line rule (`═`).
- **H2**: `## ` prefix with `markdown.h2` style (bold + cyan), followed by a single-line rule (`─`).

**H3 through H6** use bold text with a `#`-prefix matching the heading level.

```rust
let md = render_markdown(
    "# Heading 1\n\
     ## Heading 2\n\
     ### Heading 3\n\
     #### Heading 4\n\
     ##### Heading 5\n\
     ###### Heading 6"
);
console.println(&md);
```

Visual output (conceptual):

```
# Heading 1
══════════════════════════════════════════

## Heading 2
──────────────────────────────────────────

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6
```

The rule under H1 and H2 spans the full configured width and uses the theme style `rule.line`.

### Code Blocks

Fenced code blocks (with or without a language label) and indented code blocks are rendered in a box-drawing frame.

```rust
let md = render_markdown(
    "```rust\n\
     fn hello() {\n\
         println!(\"Hello!\");\n\
     }\n\
     ```"
);
console.println(&md);
```

Output:

```
┌─ Code: rust ───────────────────────┐
│ fn hello() {                       │
│     println!("Hello!");            │
│ }                                  │
└────────────────────────────────────┘
```

- The top border shows `┌─ Code: {language} ` with a trailing horizontal line.
- Code lines are prefixed with `│ `.
- The bottom border is `└` followed by horizontal rules to the full width.
- If no language is specified, the label reads `Code` instead of `Code: {language}`.
- The code box uses the `markdown.code` theme style (black background + yellow foreground in the default theme).

### Inline Code

Text wrapped in backticks is styled inline using the same `markdown.code` style.

```rust
let md = render_markdown("Use the `render_markdown()` function.");
console.println(&md);
```

Inline code appears with the code style applied to the backtick-delimited text.

### Lists

Both ordered and unordered lists are supported with bullet characters and indentation.

- **Top-level items**: `•` bullet character
- **Nested items (depth >= 2)**: `◦` bullet character with two-space indentation per level

```rust
let md = render_markdown(
    "- Item one\n\
     - Item two\n\
       - Nested item\n\
         - Deeply nested"
);
console.println(&md);
```

Output:

```
• Item one
• Item two
  ◦ Nested item
    ◦ Deeply nested
```

Ordered (numbered) lists from the Markdown source are parsed and rendered with the same bullet characters; the numeric prefixes from the source are not preserved in the terminal output.

### Blockquotes

Blockquotes render with a styled `▌` character on the left margin using the `markdown.blockquote` style.

```rust
let md = render_markdown("> This is a blockquote.\n> It can span multiple lines.");
console.println(&md);
```

Output:

```
▌ This is a blockquote.
▌ It can span multiple lines.
```

### Links

Links are rendered with the `markdown.link` style (bright blue + underline in the default theme) as `text (url)`.

```rust
let md = render_markdown("Visit [rusty-rich](https://github.com/example/rusty-rich) today!");
console.println(&md);
```

Output:

```
Visit rusty-rich (https://github.com/example/rusty-rich) today!
```

When no display text is provided (e.g., `<https://example.com>` syntax), the URL itself is shown.

### Horizontal Rules

A Markdown thematic break (`---`, `***`, `___`) renders as a horizontal line using the `Rule` component with default `─` characters.

```rust
let md = render_markdown("Above the line\n\n---\n\nBelow the line");
console.println(&md);
```

The rule spans the full available width and uses the `rule.line` style from the theme.

### Emphasis, Strong, Strikethrough

- **Emphasis** (`*text*` or `_text_`): italic styling is applied.
- **Strong** (`**text**` or `__text__`): bold styling is applied.
- **Strikethrough** (`~~text~~`): handled by the parser as a `Strikethrough` tag. Text inside strikethrough uses a style with `strikethrough(true)`.

```rust
let md = render_markdown(
    "This is *italic*, this is **bold**, and this is ~~strikethrough~~."
);
console.println(&md);
```

---

## Width Configuration

The render width can be set explicitly via `.width()` on the builder, or it falls back to `ConsoleOptions.max_width` (typically the terminal width).

```rust
// Use terminal width (default)
let md = render_markdown("Some text");

// Fixed width
let md_60 = render_markdown("Some text").width(60);

// Narrow width for side-by-side layouts
let md_narrow = render_markdown("Some text").width(30);
```

Width affects:
- The length of horizontal rules under H1/H2 headings
- The extent of the code block bottom border
- Text wrapping behavior (though the current implementation does not perform word wrapping -- lines longer than the width extend beyond the boundary)

---

## Theme Styles

The Markdown renderer uses theme style keys that users can customize via the `Theme` system.

| Style Key               | Default Style                    | Used For                         |
|-------------------------|----------------------------------|----------------------------------|
| `markdown.h1`           | bold + bright cyan               | H1 heading prefix `# `          |
| `markdown.h2`           | bold + cyan                      | H2 heading prefix `## `         |
| `markdown.code`         | yellow on black                  | Code blocks and inline code      |
| `markdown.link`         | bright blue + underline          | Link text and URL display        |
| `markdown.blockquote`   | (inherits default)               | Blockquote `▌` marker           |
| `markdown.item`         | (inherits default)               | List item bullets                |
| `rule.line`             | bright black                     | Horizontal rule characters       |

### Customizing styles

Override any style by setting it on the `Console`'s theme:

```rust
use rusty_rich::{Console, Style, Color};

let mut console = Console::new();
console.theme.set(
    "markdown.h1",
    Style::new().bold(true).color(Color::parse("magenta").unwrap()),
);
console.theme.set(
    "markdown.code",
    Style::new().color(Color::parse("green").unwrap()).bgcolor(Color::parse("grey").unwrap()),
);
```

---

## Example: Rendering a README

The following example loads a Markdown file and renders it to the console:

```rust
use rusty_rich::{render_markdown, Console};
use std::fs;

fn main() {
    let mut console = Console::new();

    // Read a README file from disk
    let readme = fs::read_to_string("README.md")
        .expect("Failed to read README.md");

    // Render it with a generous width
    let md = render_markdown(&readme).width(72);
    console.println(&md);
}
```

For a more complete example that demonstrates multiple Markdown features in a single render:

```rust
use rusty_rich::{render_markdown, Console};

fn main() {
    let mut console = Console::new();

    let md_content = r#"
# rusty-rich

A Rust port of Python's [Rich](https://github.com/Textualize/rich) library.

## Features

- **Styling**: bold, italic, underline, strikethrough, colors
- **Tables**: tabular data with headers and alignment
- **Markdown rendering**: headings, code, lists, blockquotes
- **Syntax highlighting**: powered by syntect

### Code Example

```rust
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
```

### Notes

> rusty-rich aims for feature parity with Python Rich 13.x.
> It is not a drop-in replacement, but a faithful port.

---

*Built with Rust and pulldown-cmark.*
"#;

    let md = render_markdown(md_content).width(72);
    console.println(&md);
}
```

Output (conceptual, within a 72-column terminal):

```
# rusty-rich
══════════════════════════════════════════════════════════════════════════════

A Rust port of Python's Rich (https://github.com/Textualize/rich) library.

## Features
──────────────────────────────────────────────────────────────────────────────

• Styling: bold, italic, underline, strikethrough, colors
• Tables: tabular data with headers and alignment
• Markdown rendering: headings, code, lists, blockquotes
• Syntax highlighting: powered by syntect

### Code Example

┌─ Code: rust ───────────────────────────────────────────────────────────────┐
│ fn greet(name: &str) -> String {                                          │
│     format!("Hello, {name}!")                                              │
│ }                                                                          │
└────────────────────────────────────────────────────────────────────────────┘

### Notes

▌ rusty-rich aims for feature parity with Python Rich 13.x.
▌ It is not a drop-in replacement, but a faithful port.

──────────────────────────────────────────────────────────────────────────────

Built with Rust and pulldown-cmark.

```

### Integrating with other renderables

Since `MarkdownRender` implements the `Renderable` trait, it can be composed with other components:

```rust
use rusty_rich::{render_markdown, Panel, Console};

let mut console = Console::new();

let readme = render_markdown("# Project\n\nDescription here.").width(40);

let panel = Panel::new(readme)
    .title("README Preview")
    .padding(0, 2, 0, 2);

console.println(&panel);
```

The Markdown content is rendered inside a bordered panel with the title "README Preview".

---

## Import Paths

```rust
use rusty_rich::render_markdown;          // Builder function
use rusty_rich::MarkdownRender;            // The builder struct
use rusty_rich::Theme;                     // For customizing markdown styles
use rusty_rich::Style;                     // For constructing style overrides
use rusty_rich::Color;                     // For color values in custom styles
```
