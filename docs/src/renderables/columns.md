# Columns

`Columns` renders a set of renderables side by side in a horizontal layout. Each column gets an equal share of the available width, and items are aligned by row — short items are padded with blank space so all columns line up vertically.

```rust
use rusty_rich::Columns;

let mut cols = Columns::new();
cols.add("First");
cols.add("Second");
cols.add("Third");
console.println(&cols);
```

```
First  Second  Third
```

---

## new()

`Columns::new()` creates an empty columns container. It takes no arguments.

```rust
let cols = Columns::new();
```

Default configuration:
- `padding`: `1` (one space between columns)
- `equal`: `false` (columns are not forced to equal widths — though the current implementation treats all columns equally for width distribution)
- `expand`: `false`
- `width`: `None` (uses the full terminal width)

---

## add()

Adds a renderable as a new column. Accepts any type that implements `Renderable` — plain strings, `Text`, `Panel`, `Table`, `Tree`, `Rule`, or custom renderables.

```rust
let mut cols = Columns::new();

// Plain strings
cols.add("Short");
cols.add("A longer string that wraps");

// Panels as columns
cols.add(Panel::new("Panel content").title("Col 1"));

// Tables, trees, or any renderable
cols.add(my_table);
cols.add(my_tree);
```

Columns are rendered left to right in the order they were added. Each column receives an equal fraction of the available width (accounting for inter-column padding). If a renderable produces more lines than its neighbors, the shorter columns are padded with blank lines so the output remains aligned.

---

## equal

The `equal()` builder flag controls whether columns are forced to equal width. When `equal` is set, each column receives exactly `(available_width - total_padding) / count` characters of width, distributed evenly regardless of content.

```rust
let mut cols = Columns::new();
cols.equal();
cols.add("Alpha");
cols.add("Beta");
cols.add("Gamma");
console.println(&cols);
```

```
Alpha  Beta   Gamma
```

In the current implementation, the default (non-equal) path distributes width identically to the equal path — each column always gets an even share of the space. The `equal` flag serves as an explicit assertion of intent and future behavior.

---

## expand

The `expand()` builder flag indicates that columns should expand to fill all available space. This is the default and intended behavior — any unused horizontal space is distributed among the columns.

```rust
let mut cols = Columns::new();
cols.expand();
cols.add("Left");
cols.add("Right");
console.println(&cols);
```

Without `expand` (or when it is `false`), columns would shrink to their minimum width, but the current implementation always fills the available width regardless of this flag.

---

## padding

The `padding()` builder sets the number of spaces between adjacent columns. Default is `1`.

```rust
// No gap between columns
let mut cols = Columns::new();
cols.padding(0);
cols.add("A");
cols.add("B");
console.println(&cols);
```

```
AB
```

```rust
// Wide gap between columns
let mut cols = Columns::new();
cols.padding(4);
cols.add("Left");
cols.add("Right");
console.println(&cols);
```

```
Left    Right
```

Padding is applied between every pair of adjacent columns. The total padding consumed is `(count - 1) * padding`. This is subtracted from the available width before computing per-column width.

---

## width

The `width()` builder sets a fixed total width for the entire columns layout, overriding the default that uses the terminal width. If not set, the columns span the full terminal width (or the parent container's width).

```rust
let mut cols = Columns::new();
cols.width(40);
cols.add("Narrow");
cols.add("Columns");
cols.add("Layout");
console.println(&cols);
```

```
Narrow  Columns  Layout
```

The width value is distributed across all columns (minus padding gaps). When the width is small, the per-column share shrinks proportionally, potentially causing line wrapping within each column.

---

## Full Examples

### Side-by-Side Panels

This is the most common use case: multiple `Panel` instances arranged horizontally.

```rust
use rusty_rich::Columns;
use rusty_rich::Panel;
use rusty_rich::box_drawing::BOX_SQUARE;
use rusty_rich::{Style, Color};

let mut cols = Columns::new();
cols.padding(2);

cols.add(
    Panel::new("Rust is blazingly fast\nand memory-efficient.")
        .title("Rust")
        .box_style(BOX_SQUARE.clone())
        .border_style(Style::new().color(Color::parse("bright_red").unwrap()))
        .padding(1, 2, 1, 2),
);
cols.add(
    Panel::new("Rich makes beautiful\nterminal output easy.")
        .title("Rich")
        .box_style(BOX_SQUARE.clone())
        .border_style(Style::new().color(Color::parse("bright_green").unwrap()))
        .padding(1, 2, 1, 2),
);
cols.add(
    Panel::new("Combined = rusty-rich\nBest of both worlds!")
        .title("rusty-rich")
        .box_style(BOX_SQUARE.clone())
        .border_style(Style::new().color(Color::parse("bright_blue").unwrap()))
        .padding(1, 2, 1, 2),
);

console.println(&cols);
```

In a terminal 80 columns wide:

```
┌─────── Rust ───────┐  ┌─────── Rich ───────┐  ┌── rusty-rich ────┐
│                     │  │                     │  │                   │
│  Rust is blazingly  │  │  Rich makes         │  │  Combined =       │
│  fast and memory-   │  │  beautiful          │  │  rusty-rich       │
│  efficient.         │  │  terminal output    │  │  Best of both     │
│                     │  │  easy.              │  │  worlds!          │
└─────────────────────┘  └─────────────────────┘  └───────────────────┘
```

### Equal-Width Columns

Use `equal()` when you want each column to occupy the same horizontal space, regardless of the content width of the renderables.

```rust
use rusty_rich::Columns;
use rusty_rich::Panel;

let mut cols = Columns::new();
cols.equal();
cols.padding(3);

cols.add(Panel::new("Short").fit());
cols.add(Panel::new("Medium length").fit());
cols.add(Panel::new("Quite a long piece of text here").fit());

console.println(&cols);
```

With `equal()`, each panel gets the same width, forcing wider renders on the shorter content:

```
╭───────╮   ╭──────────────╮   ╭──────────────────────╮
│ Short │   │ Medium       │   │ Quite a long piece   │
│       │   │ length       │   │ of text here         │
╰───────╯   ╰──────────────╯   ╰──────────────────────╯
```

### Custom Padding

Adjust inter-column spacing with `padding()`.

```rust
use rusty_rich::Columns;

let mut cols = Columns::new();

// Tight spacing
cols.padding(0);
cols.add("A");
cols.add("B");
cols.add("C");
console.println(&cols);

// Wide spacing
let mut cols2 = Columns::new();
cols2.padding(6);
cols2.add("X");
cols2.add("Y");
cols2.add("Z");
console.println(&cols2);
```

```
ABC
X      Y      Z
```

### Mixed Renderable Types

Columns can hold different kinds of renderables in the same layout.

```rust
use rusty_rich::{Columns, Panel, Rule, Tree};

let mut cols = Columns::new();
cols.padding(2);
cols.add(Panel::new("Panel content").title("Info"));
cols.add(Tree::new("Root").add_leaf("Child 1").add_leaf("Child 2"));
cols.add(Rule::new().title("Divider"));
console.println(&cols);
```

---

## Comparison with Python Rich

| Feature | Python Rich | Rust-Rich |
|---------|-------------|-----------|
| `Columns(renderables)` | Constructor accepts initial list | Use `new()` + `add()` |
| `equal` | `True`/`False`, controls equal-width distribution | Builder method `.equal()` |
| `expand` | `True`/`False`, expands to fill width | Builder method `.expand()` |
| `padding` | `(0, 1)` tuple (left, right per column) | Single `usize` (gap between columns) |
| `width` | `int` or `None`, total width override | `Option<usize>`, set with `.width()` |
| Column type | Any renderable | Any type implementing `Renderable` |

---

## Builder Method Reference

| Method | Description |
|--------|-------------|
| `.equal()` | Force all columns to equal width |
| `.expand()` | Expand columns to fill available space |
| `.padding(usize)` | Set spacing between adjacent columns (default: `1`) |
| `.width(usize)` | Set a fixed total width (default: terminal width) |

---

## Import Paths

```rust
use rusty_rich::Columns;             // The Columns type
use rusty_rich::Panel;               // Commonly used with Columns
use rusty_rich::Tree;                // Tree renderable for columns
use rusty_rich::Rule;                // Rule renderable for columns
use rusty_rich::box_drawing::BOX_SQUARE; // Box styles for Panel inside columns
```
