# rusty-rich

**Rich text and beautiful formatting for the terminal** — a Rust port of Python's [Rich](https://github.com/Textualize/rich) library.

![rusty-rich logo](../assets/logo.svg)

## What is rusty-rich?

rusty-rich brings the power of Python Rich to Rust — beautiful terminal output with minimal effort. Whether you're building a CLI tool, a TUI application, or just want better debug output, rusty-rich has you covered.

## Key Features

- 🎨 **Rich styling** — 13 text attributes, TrueColor support, named colors
- 📝 **Console markup** — BBCode-like `[bold red]text[/bold red]` inline styling
- 📊 **Tables** — with colspan/rowspan, headers, footers, sections, 17 box styles
- ⏳ **Progress** — multi-task bars, file tracking, 7 column types
- 🌈 **Syntax highlighting** — 100+ languages via syntect
- 📝 **Markdown** — headings, code blocks, lists, blockquotes
- ⌨️ **Prompts** — string, int, float, confirm, select, password
- 🔴 **Tracebacks** — rich error rendering with source code and locals
- 🖥️ **Full-screen** — alternate screen, live updating displays

## Quick Example

```rust
use rusty_rich::Console;

fn main() {
    let mut console = Console::new();
    console.print_str("[bold green]Hello, [red]World![/red][/bold green]");
}
```

## Project Status

rusty-rich achieves ~90% feature parity with Python Rich 13.x, with 171 passing tests.
