# Tree

`Tree` renders hierarchical data as an indented tree structure with guide lines connecting parent and child nodes. It is useful for file system trees, organisational charts, nested menus, AST visualisations, and any other parent-child data.

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("Root");
tree.add("Child 1");
tree.add("Child 2");
console.println(&tree);
```

```
Root
├── Child 1
└── Child 2
```

---

## new(label)

`Tree::new()` creates a new root node with the given label. The label can be any `String` or `&str`.

```rust
use rusty_rich::Tree;

let tree = Tree::new("Project Root");
```

The default configuration:
- `style`: no style
- `guide_style`: no style
- `expanded`: `true` (children are visible)
- `highlight`: `false`
- `hide_root`: `false`
- `children`: empty

---

## add(label)

`tree.add(label)` appends a child node and returns `&mut Tree` — a mutable reference to the newly added child. This return value enables method chaining to build nested structures without naming intermediate variables.

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("Root");
let child = tree.add("Branch");
child.add("Leaf 1");
child.add("Leaf 2");
```

Because `add` returns `&mut Tree`, children can be added to the child directly in a chain:

```rust
let mut tree = Tree::new("Root");
tree.add("Branch")
    .add("Leaf 1");
tree.add("Leaf 2");
```

This is the primary mechanism for constructing arbitrary-depth trees.

---

## hide_root

`.hide_root()` hides the root node label from the output. Only the children of the hidden root are rendered, each connected to the top-level guide lines. This is useful when you want a forest of top-level items without a root heading.

```rust
let mut tree = Tree::new("");   // label is hidden
tree.hide_root();
tree.add("Item 1");
tree.add("Item 2");
tree.add("Item 3");
```

```
├── Item 1
├── Item 2
└── Item 3
```

---

## expanded

The `expanded` field controls whether a node's children are visible. When `expanded` is `false`, the node renders as a leaf regardless of how many children it has.

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("Collapsible Root");
let branch = tree.add("Collapsed Branch");
branch.add("Hidden Leaf 1");
branch.add("Hidden Leaf 2");
branch.expanded = false;
tree.add("Visible Leaf");
```

```
Collapsible Root
├── Collapsed Branch
└── Visible Leaf
```

Setting `expanded` is a direct field mutation — there is no builder method for it (the field is public).

---

## guide_style

`.guide_style(style)` applies a `Style` to the guide line characters (the connectors and vertical lines) without affecting the node labels.

```rust
use rusty_rich::Tree;
use rusty_rich::style::Style;
use rusty_rich::color::Color;

let mut tree = Tree::new("Root")
    .guide_style(Style::new().color(Color::parse("bright_black").unwrap()));
tree.add("Child 1");
tree.add("Child 2");
```

In this example the tree connectors (`├──`, `└──`, `│`) render in bright black while the labels appear in the default terminal colour.

---

## style

`.style(style)` applies a `Style` to the node's label text. This can be chained during construction.

```rust
use rusty_rich::Tree;
use rusty_rich::style::Style;

let tree = Tree::new("Bold Root")
    .style(Style::new().bold(true));
```

When nodes are added via `add()`, each node carries its own `style` field, allowing different styles per node.

---

## TreeGuides

`TreeGuides` defines the four character sequences used to draw the guide lines at each tree position. The struct has four fields:

| Field            | Description                                    |
|------------------|------------------------------------------------|
| `space`          | Indentation for the final child (no line).     |
| `continue_line`  | Vertical line used for non-last siblings.      |
| `fork`           | Branch connector for non-last siblings.        |
| `end`            | Branch connector for the last sibling.         |

### Unicode (default)

The `TREE_GUIDES` constant uses Unicode box-drawing characters:

| Constant       | Characters      | Description                       |
|----------------|-----------------|-----------------------------------|
| `space`        | `"    "`        | Four spaces, empty continuation.  |
| `continue_line`| `"│   "`        | Vertical bar followed by spaces.  |
| `fork`         | `"├── "`        | Branch with a horizontal arm.     |
| `end`          | `"└── "`        | Corner turning down and right.    |

### ASCII

The `ASCII_GUIDES` constant uses pure ASCII characters, safe for legacy terminals or ASCII-only output modes:

| Constant       | Characters      | Description                       |
|----------------|-----------------|-----------------------------------|
| `space`        | `"    "`        | Four spaces.                      |
| `continue_line`| `"|   "`        | Pipe followed by spaces.          |
| `fork`         | `"+-- "`        | Plus sign with dashes.            |
| `end`          | "`-- "`         | Backtick with dashes.             |

### Automatic fallback

When the console's `ConsoleOptions.ascii_only` is `true`, the tree automatically switches to `ASCII_GUIDES`. No manual configuration is needed — pass the tree to any `Console` and the renderer selects the appropriate guides based on terminal capabilities.

To force ASCII guides programmatically, set `options.ascii_only = true` on the console options.

```rust
use rusty_rich::tree::{TREE_GUIDES, ASCII_GUIDES, TreeGuides};

// Unicode (default)
println!("space: {:?}", TREE_GUIDES.space);          // "    "
println!("fork:  {:?}", TREE_GUIDES.fork);           // "├── "

// ASCII fallback
println!("fork:  {:?}", ASCII_GUIDES.fork);          // "+-- "
```

### Custom guides

Construct a `TreeGuides` literal directly to use custom connector characters:

```rust
let dashed: TreeGuides = TreeGuides {
    space:         "    ",
    continue_line: ":   ",   // colon instead of pipe
    fork:          ":- ",    // simpler branch
    end:           ":_ ",    // last child
};
```

Custom guides are not used automatically — they must be wired into the rendering logic or applied at the console level.

---

## Examples

### Simple tree

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("Shopping List");
tree.add("Apples");
tree.add("Bananas");
tree.add("Milk");
tree.add("Bread");

console.println(&tree);
```

```
Shopping List
├── Apples
├── Bananas
├── Milk
└── Bread
```

### Nested tree with chaining

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("Project");
tree.add("src")
    .add("main.rs")
    .add("lib.rs");
tree.add("tests")
    .add("test_main.rs");
tree.add("Cargo.toml");
tree.add("README.md");

console.println(&tree);
```

```
Project
├── src
│   ├── main.rs
│   └── lib.rs
├── tests
│   └── test_main.rs
├── Cargo.toml
└── README.md
```

Deep nesting is supported by continuing the chain:

```rust
let mut tree = Tree::new("data");
tree.add("2025")
    .add("Q1")
        .add("january.csv")
        .add("february.csv")
        .add("march.csv");
tree.add("2026")
    .add("Q1")
        .add("january.csv");
```

```
data
├── 2025
│   └── Q1
│       ├── january.csv
│       ├── february.csv
│       └── march.csv
└── 2026
    └── Q1
        └── january.csv
```

### Hidden root

```rust
use rusty_rich::Tree;

let mut tree = Tree::new("Invisible");
tree.hide_root();
tree.add("Chapter 1: Introduction");
tree.add("Chapter 2: Background");
tree.add("Chapter 3: Methods");
tree.add("Chapter 4: Results");
tree.add("Chapter 5: Conclusion");

console.println(&tree);
```

```
├── Chapter 1: Introduction
├── Chapter 2: Background
├── Chapter 3: Methods
├── Chapter 4: Results
└── Chapter 5: Conclusion
```

### Styled tree

```rust
use rusty_rich::Tree;
use rusty_rich::style::Style;
use rusty_rich::color::Color;

// Dim guides, bold root label
let mut tree = Tree::new("Styled Hierarchy")
    .style(Style::new().bold(true))
    .guide_style(Style::new().color(Color::parse("bright_black").unwrap()));

let branch = tree.add("Section A");
branch.style = Style::new().color(Color::parse("cyan").unwrap());
branch.add("Item 1");
branch.add("Item 2");

let branch = tree.add("Section B");
branch.style = Style::new().color(Color::parse("green").unwrap());
branch.add("Item A");
branch.add("Item B");

console.println(&tree);
```

The root label renders in bold, the guide lines appear in dim grey, and each top-level section has its own distinct colour.

---

## Import Paths

```rust
use rusty_rich::Tree;                                          // The Tree renderable
use rusty_rich::tree::TreeGuides;                              // Guide character definition
use rusty_rich::tree::{TREE_GUIDES, ASCII_GUIDES};             // Predefined guide sets
use rusty_rich::style::Style;                                  // Node and guide styling
use rusty_rich::color::Color;                                  // Colour values for styles
```
