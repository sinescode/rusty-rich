# Layout

`Layout` is a split-pane layout system that recursively divides a rectangular area into named regions. It is the equivalent of Python Rich's `layout.py`. Use it to build terminal user interfaces composed of side-by-side or stacked panes with proportional sizing.

```rust
use rusty_rich::{Layout, LayoutNode, Direction, Region};

let mut layout = Layout::new(LayoutNode::split(
    Direction::Horizontal,
    vec![
        LayoutNode::Leaf { name: "left".into(), renderable: None, size: None },
        LayoutNode::Leaf { name: "right".into(), renderable: None, size: None },
    ],
));

let regions = layout.compute(80, 24);
```

---

## Region

`Region` is a rectangular area on screen defined by its top-left corner and dimensions.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
```

| Field    | Description                     |
|----------|---------------------------------|
| `x`      | Column coordinate (left edge).  |
| `y`      | Row coordinate (top edge).      |
| `width`  | Width in columns.               |
| `height` | Height in rows.                 |

All coordinates are zero-based. The origin `(0, 0)` is the top-left corner of the terminal or parent container. Regions are returned by `Layout::compute()` and describe the final position and size of each named leaf in the layout tree.

---

## Direction

`Direction` selects the axis along which a split divides space.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}
```

| Variant       | Behavior                                                    |
|---------------|-------------------------------------------------------------|
| `Horizontal`  | Children are placed side-by-side; available width is divided. A 1-column gutter is inserted between adjacent children. |
| `Vertical`    | Children are stacked top-to-bottom; available height is divided. No gutter is inserted. |

---

## LayoutNode

`LayoutNode` is a recursive enum that describes the layout tree. Every layout is a tree of splits and leaves.

```rust
#[derive(Debug, Clone)]
pub enum LayoutNode {
    Split {
        direction: Direction,
        sizes: Vec<usize>,       // relative size ratios
        children: Vec<LayoutNode>,
    },
    Leaf {
        name: String,
        renderable: Option<String>, // reserved
        size: Option<usize>,        // optional fixed size
    },
}
```

### Leaf

A `Leaf` is a terminal node — it occupies a region and is identified by its `name`. The `renderable` field is reserved for attaching actual renderable content. The optional `size` field clamps the region's width and height to the given value (minimum of computed size and the fixed size):

```rust
// A leaf that will be at most 20 columns wide and 20 rows tall
let leaf = LayoutNode::Leaf {
    name: "sidebar".into(),
    renderable: None,
    size: Some(20),
};
```

When `size` is `None` (the default), the leaf occupies the full region it is allocated, subject to a minimum width of 2 columns and minimum height of 1 row.

### Split

A `Split` divides space among its children along the given `Direction`. Each child receives a portion of the available space proportional to its entry in the `sizes` vector.

```rust
// Two children with a 3:1 width ratio
let split = LayoutNode::Split {
    direction: Direction::Horizontal,
    sizes: vec![3, 1],
    children: vec![
        LayoutNode::Leaf { name: "main".into(), renderable: None, size: None },
        LayoutNode::Leaf { name: "sidebar".into(), renderable: None, size: None },
    ],
};
```

If you do not provide explicit `sizes`, the `split()` constructor assigns equal ratios:

```rust
// Every child gets equal space
let equal_split = LayoutNode::split(Direction::Vertical, children);
```

The helper methods `LayoutNode::split()` and `LayoutNode::sizes()` provide a builder-style API:

```rust
LayoutNode::split(Direction::Horizontal, vec![left, right])
    .sizes(vec![2, 1])  // left gets 2/3, right gets 1/3
```

---

## Layout

`Layout` is the compute engine that takes a `LayoutNode` tree and a screen area, then produces a list of named regions.

```rust
#[derive(Debug, Clone)]
pub struct Layout {
    pub root: LayoutNode,
    pub visible: bool,
    pub minimum_size: usize,
}
```

| Field          | Default | Description                                           |
|----------------|---------|-------------------------------------------------------|
| `root`         | —       | The root `LayoutNode` of the layout tree.             |
| `visible`      | `true`  | Whether the layout is visible. When `false`, `compute()` returns an empty list. |
| `minimum_size` | `1`     | Minimum pixel/character size for the layout (reserved for future use). |

### new()

```rust
let layout = Layout::new(root_node);
```

Creates a `Layout` with the given root node. `visible` defaults to `true` and `minimum_size` defaults to `1`.

### compute()

```rust
pub fn compute(&self, total_width: usize, total_height: usize) -> Vec<(String, Region)>
```

Recursively traverses the layout tree and partitions the area `(total_width, total_height)` into regions. Returns a vector of `(name, region)` tuples — one per leaf node — in depth-first order.

```rust
let layout = Layout::new(LayoutNode::split(
    Direction::Horizontal,
    vec![
        LayoutNode::Leaf { name: "left".into(), renderable: None, size: None },
        LayoutNode::Leaf { name: "right".into(), renderable: None, size: None },
    ],
));

let regions = layout.compute(80, 24);
// regions[0] = ("left",  Region { x: 0,  y: 0, width: 39, height: 24 })
// regions[1] = ("right", Region { x: 40, y: 0, width: 40, height: 24 })
```

The compute algorithm:
1. On a `Leaf`: clamp to optional `size`, enforce minimums, record the region.
2. On a `Split`:
   - **Horizontal**: total spacing = `(count - 1)` for 1-column gutters. Remaining width is divided by the ratio `sizes[i] / total_sizes`. Each child is placed at increasing `x` with `height` equal to the parent region height.
   - **Vertical**: total height is divided by the ratio `sizes[i] / total_sizes`. No gutters. Each child is placed at increasing `y` with `width` equal to the parent region width.

---

## Ratio-Based Sizing

The `sizes` field on a `Split` node uses `Vec<usize>` where each element is a relative weight. The actual pixel/character allocation for child `i` is:

```
child_size = (available_space * sizes[i]) / sum(sizes)
```

A horizontal split in an 80-column area with sizes `[3, 1]`:

```
|←────────── 80 columns ─────────→|
|←──── 60 ───→|←──── 20 ────→|
|   Child 0     |   Child 1     |   (3:1 ratio, gutter at column 60)
```

Equal ratios `[1, 1, 1]` in a vertical split of height 24 produce three children of height 8 each.

Leaf nodes also accept an optional `size` field that acts as a maximum clamp — a leaf's region will never exceed that dimension regardless of what the parent split allocates:

```rust
// This leaf will be at most 30 units even if the split gives it more
LayoutNode::Leaf { name: "nav".into(), renderable: None, size: Some(30) }
```

When `size` is `None` (the default), the leaf expands to fill its allocated space subject to the built-in minimums (width >= 2, height >= 1).

---

## Named Regions

Every leaf in the layout tree must have a unique `name`. The `compute()` method returns these names as the first element of each `(String, Region)` tuple, allowing you to map renderables to screen positions by name.

```rust
let regions = layout.compute(80, 24);
for (name, region) in &regions {
    println!("{} is at x={}, y={}, {}x{}", name, region.x, region.y, region.width, region.height);
}
```

You can then look up a region by name and render content into it:

```rust
fn render_region(name: &str, content: &dyn Renderable, regions: &[(String, Region)]) {
    for (n, region) in regions {
        if n == name {
            // Render content into this screen region
        }
    }
}
```

---

## Examples

### Side-by-side panes (horizontal split)

Divide the terminal into a sidebar and a main content area with a 1:3 width ratio:

```rust
use rusty_rich::{Layout, LayoutNode, Direction};

let layout = Layout::new(
    LayoutNode::split(
        Direction::Horizontal,
        vec![
            LayoutNode::Leaf {
                name: "sidebar".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "main".into(),
                renderable: None,
                size: None,
            },
        ],
    )
    .sizes(vec![1, 3]),
);

let regions = layout.compute(80, 24);
```

Result (conceptual, 80-column terminal):

```
|←── 20 cols ──→|←────────── 59 cols ──────────→|
|  sidebar       |  main                         |
|                |                               |
|                |                               |
|                |                               |
|                |                               |
+────────────────+───────────────────────────────+
```

The gutter between panes consumes 1 column, so the total used is `20 + 1 + 59 = 80`.

### Stacked panes (vertical split)

Stack a header, a body, and a status bar with a 1:6:1 height ratio:

```rust
use rusty_rich::{Layout, LayoutNode, Direction};

let layout = Layout::new(
    LayoutNode::split(
        Direction::Vertical,
        vec![
            LayoutNode::Leaf { name: "header".into(), renderable: None, size: Some(3) },
            LayoutNode::Leaf { name: "body".into(),   renderable: None, size: None    },
            LayoutNode::Leaf { name: "status".into(), renderable: None, size: Some(1) },
        ],
    )
    .sizes(vec![1, 6, 1]),
);

let regions = layout.compute(80, 24);
```

Result (conceptual):

```
┌──────────────────────────────────────────────┐
│ header                   (height: 3, clamped) │
├──────────────────────────────────────────────┤
│                                                │
│ body                     (height: 20)          │
│                                                │
├──────────────────────────────────────────────┤
│ status                   (height: 1, clamped) │
└──────────────────────────────────────────────┘
```

The header's `size: Some(3)` clamps it to 3 rows even if the ratio would allocate more. The body receives the remaining height. The status bar is clamped to 1 row.

### Nested split (grid layout)

Combine horizontal and vertical splits for a multi-pane layout — a tree view on the left, and a split-pane editor area on the right:

```rust
use rusty_rich::{Layout, LayoutNode, Direction};

// Build a tree view leaf
let tree = LayoutNode::Leaf { name: "tree".into(), renderable: None, size: Some(30) };

// Build the editor area (vertical split: edit pane + log pane)
let editor_area = LayoutNode::split(
    Direction::Vertical,
    vec![
        LayoutNode::Leaf { name: "editor".into(), renderable: None, size: None },
        LayoutNode::Leaf { name: "log".into(),    renderable: None, size: None },
    ],
).sizes(vec![3, 1]);

// Root split: tree on the left, editor area on the right
let layout = Layout::new(
    LayoutNode::split(Direction::Horizontal, vec![tree, editor_area])
        .sizes(vec![1, 3]),
);

let regions = layout.compute(80, 24);
// ("tree",   Region { x: 0,  y: 0, width: 19, height: 24 })
// ("editor", Region { x: 20, y: 0, width: 60, height: 18 })
// ("log",    Region { x: 20, y: 18, width: 60, height: 6 })
```

Visual representation:

```
|← tree →|←─────────── editor area ───────────→|
|         |                                      |
|         |  editor (height: 18)                 |
|         |                                      |
|         |                                      |
|         |                                      |
|         |                                      |
|         ├──────────────────────────────────────┤
|         |  log (height: 6)                     |
|         |                                      |
+─────────+──────────────────────────────────────+
```

### Fixed-size leaf

A leaf with `size: Some(n)` acts as a maximum bound. The leaf never exceeds `n` in either dimension, even if the parent split allocates more space:

```rust
use rusty_rich::{Layout, LayoutNode, Direction};

let narrow = LayoutNode::Leaf {
    name: "narrow".into(),
    renderable: None,
    size: Some(15), // max 15 columns wide
};

let rest = LayoutNode::Leaf {
    name: "rest".into(),
    renderable: None,
    size: None,
};

let layout = Layout::new(
    LayoutNode::split(Direction::Horizontal, vec![narrow, rest])
        .sizes(vec![1, 4]),
);

let regions = layout.compute(80, 24);
// ("narrow", Region { x: 0, y: 0, width: 15, height: 24 })  // clamped to 15
// ("rest",   Region { x: 16, y: 0, width: 64, height: 24 }) // receives remainder
```

If the allocated space is smaller than the clamped size, the leaf takes the smaller value — `size` never expands.

---

## Import Paths

```rust
use rusty_rich::Layout;                          // The layout compute engine
use rusty_rich::LayoutNode;                      // Tree node (Leaf or Split)
use rusty_rich::Direction;                       // Horizontal or Vertical
use rusty_rich::Region;                          // Rectangular screen region
```

---

## Differences from Python Rich

The rusty-rich `Layout` is structurally similar to Python Rich's `layout.py` but diverges in a few ways:

| Aspect                  | Python Rich                          | rusty-rich                           |
|-------------------------|--------------------------------------|--------------------------------------|
| Node type               | `Layout` class with `_children` and `_renderable` | `LayoutNode` enum: `Leaf` / `Split`  |
| Ratio API               | `ratio` property, fractional floats  | `sizes: Vec<usize>` integer ratios   |
| Fixed size              | `size` attribute                     | `size: Option<usize>` on `Leaf`      |
| Named lookup            | `Layout` stores all named children   | `compute()` returns `Vec<(String, Region)>` |
| Renderable attachment   | `update_renderable()` method         | `renderable: Option<String>` field (reserved) |
| Gutter (horizontal)     | 1-column gap when `column` is true   | Always a 1-column gutter in horizontal splits |
| Minimum size            | Configurable per-node                | Built-in minima: width >= 2, height >= 1 |

The rusty-rich `Layout` is a pure layout calculator — it computes regions but does not manage a render loop or screen refresh. It is designed to be integrated into a broader TUI framework or used ad-hoc with `Console` for static split-pane output.
