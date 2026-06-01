# Padding & Align

rusty-rich provides two complementary families of helpers for positioning renderable content: **Padding** (spacing around content) and **Align** (horizontal and vertical positioning).

---

## Padding

`Padding` is a renderable wrapper that adds blank space around inner content. At its core is `PaddingDimensions`, a CSS-inspired specification that accepts 1, 2, or 4 values.

---

### PaddingDimensions

`PaddingDimensions` stores four `usize` fields: `top`, `right`, `bottom`, `left`.

```rust
use rusty_rich::PaddingDimensions;

// All sides equal (1-value)
let all = PaddingDimensions::all(2);
// top=2, right=2, bottom=2, left=2

// Vertical + horizontal (2-value)
let sym = PaddingDimensions::symmetric(1, 3);
// top=1, right=3, bottom=1, left=3

// Top, right, bottom, left (4-value)
let four = PaddingDimensions::new(1, 2, 3, 4);
// top=1, right=2, bottom=3, left=4
```

| Constructor | Arguments | CSS equivalent | Behavior |
|-------------|-----------|----------------|----------|
| `PaddingDimensions::all(n)` | 1 value | `padding: n` | `n` on all four sides |
| `PaddingDimensions::symmetric(v, h)` | 2 values | `padding: v h` | `v` on top/bottom, `h` on left/right |
| `PaddingDimensions::new(t, r, b, l)` | 4 values | `padding: t r b l` | Clockwise from top |

---

### Padding wrapper

`Padding` wraps any `Renderable` and adds space around it.

```rust
use rusty_rich::{Padding, PaddingDimensions};

// Construct with content, then configure
let padded = Padding::new("Hello, world!")
    .pad(1, 2, 1, 2);   // top, right, bottom, left
console.println(&padded);
```

```
  Hello, world!  
                 
```

(The outer padding is blank space; the example shows left/right/top/bottom spacing.)

When no padding is explicitly set, `.pad_all(0)` is the default.

#### pad() — 4-value

```rust
Padding::new("Content")
    .pad(1, 4, 2, 4)
```

Sets `(top, right, bottom, left)` independently.

#### pad_all() — uniform

```rust
Padding::new("Content")
    .pad_all(2)
```

Sets the same padding value on all four sides. Equivalent to `PaddingDimensions::all(2)`.

#### style() — padding background

Sets the `Style` applied to the padding space characters. This can be used to give the padded area a background color.

```rust
use rusty_rich::{Padding, Style, Color};

Padding::new("Content")
    .pad_all(1)
    .style(Style::new().bg(Color::new(0x2a, 0x2a, 0x2a)))
```

---

### indent()

`indent(level)` is a convenience that sets `left` padding to `level` and disables `expand`. This produces a left-indented block that is only as wide as its content plus the indent.

```rust
use rusty_rich::Padding;

let indented = Padding::new("Indented line")
    .indent(4);
console.println(&indented);
```

```
    Indented line
```

`indent()` is equivalent to `.pad(0, 0, 0, level).expand(false)`.

---

### expand

By default `expand` is `true`, meaning the padding wrapper fills the full available width of the terminal (or parent container). The padding space on the right side extends to the terminal edge.

```rust
use rusty_rich::Padding;

// expand: true (default) -- padding stretches to terminal width
let expanded = Padding::new("Hello")
    .pad_all(1);
console.println(&expanded);
```

Setting `expand` to `false` makes the padding wrapper shrink to fit its content width plus the padding dimensions.

```rust
// expand: false -- padding wraps tightly around content
let tight = Padding::new("Hello")
    .pad_all(1)
    .expand(false);
console.println(&tight);
```

The `indent()` method automatically sets `expand = false`.

---

### Builder Method Reference (Padding)

| Method | Description |
|--------|-------------|
| `Padding::new(renderable)` | Create a new Padding wrapper around any `Renderable` |
| `.pad(top, right, bottom, left)` | Set padding as 4 values |
| `.pad_all(n)` | Set uniform padding on all sides |
| `.indent(level)` | Left-indent by `level` spaces, sets `expand = false` |
| `.style(s)` | Apply a `Style` to the padding space characters |
| `.expand(true/false)` | When `true` (default), pad to full width; when `false`, shrink-wrap |

---

### Import Paths (Padding)

```rust
use rusty_rich::Padding;                // The Padding wrapper
use rusty_rich::PaddingDimensions;     // The dimensions struct
```

---

## Align

`Align` wraps a renderable and positions it horizontally and/or vertically within a bounding area. It provides two enums for specifying alignment and convenience constructors for common configurations.

---

### AlignMethod

`AlignMethod` controls horizontal text positioning.

| Variant | Description |
|---------|-------------|
| `AlignMethod::Left` | Aligns content to the left edge (default) |
| `AlignMethod::Center` | Centers content horizontally |
| `AlignMethod::Right` | Aligns content to the right edge |
| `AlignMethod::Full` | Full justification — distributes spaces between words |

```rust
use rusty_rich::AlignMethod;

let left   = AlignMethod::Left;
let center = AlignMethod::Center;
let right  = AlignMethod::Right;
let full   = AlignMethod::Full;
```

#### align_text()

`AlignMethod` has a utility method `align_text(text, width)` that pads a string to the given width using the chosen alignment.

```rust
use rusty_rich::AlignMethod;

let s = AlignMethod::Center.align_text("Hi", 10);
assert_eq!(s.chars().count(), 10);
// "    Hi    "
```

| Method | Input `"Hi"`, width 10 | Result |
|--------|------------------------|--------|
| `Left` | `.align_text("Hi", 10)` | `"Hi        "` |
| `Center` | `.align_text("Hi", 10)` | `"    Hi    "` |
| `Right` | `.align_text("Hi", 10)` | `"        Hi"` |
| `Full` | `.align_text("Hi there", 10)` | `"Hi   there"` (justified) |

`Full` justification distributes extra space between words. For single-word strings it behaves like `Left`.

You can also parse an `AlignMethod` from a string:

```rust
assert_eq!(AlignMethod::from_str("center"), AlignMethod::Center);
assert_eq!(AlignMethod::from_str("left"),   AlignMethod::Left);
assert_eq!(AlignMethod::from_str("right"),  AlignMethod::Right);
assert_eq!(AlignMethod::from_str("full"),   AlignMethod::Full);
```

---

### VerticalAlignMethod

`VerticalAlignMethod` controls vertical positioning.

| Variant | Description |
|---------|-------------|
| `VerticalAlignMethod::Top` | Aligns to the top (default) |
| `VerticalAlignMethod::Middle` | Centers vertically |
| `VerticalAlignMethod::Bottom` | Aligns to the bottom |

```rust
use rusty_rich::VerticalAlignMethod;

let top    = VerticalAlignMethod::Top;
let middle = VerticalAlignMethod::Middle;
let bottom = VerticalAlignMethod::Bottom;
```

Parse from string:

```rust
assert_eq!(VerticalAlignMethod::from_str("top"),    VerticalAlignMethod::Top);
assert_eq!(VerticalAlignMethod::from_str("middle"), VerticalAlignMethod::Middle);
assert_eq!(VerticalAlignMethod::from_str("bottom"), VerticalAlignMethod::Bottom);
```

---

### Align wrapper

`Align<T>` wraps a renderable of type `T` and applies horizontal and/or vertical alignment within an optional bounding `width` and `height`.

```rust
use rusty_rich::{Align, AlignMethod, VerticalAlignMethod};

// Basic construction (defaults: Left, Top)
let aligned = Align::new("Hello")
    .align(AlignMethod::Center)
    .vertical(VerticalAlignMethod::Middle);
console.println(&aligned);
```

The `width` and `height` fields constrain the bounding box. When left at `None`, the alignment operates within the available terminal or parent width/height.

---

### center()

`Align::center(renderable)` is a convenience constructor that applies `AlignMethod::Center` for horizontal centering.

```rust
use rusty_rich::Align;

let centered = Align::center("Centered text");
console.println(&centered);
```

In a terminal 40 columns wide:

```
            Centered text            
```

Equivalent to:

```rust
Align::new(renderable).align(AlignMethod::Center)
```

---

### middle()

`Align::middle(renderable)` is a convenience constructor that applies `VerticalAlignMethod::Middle` for vertical centering. It only takes effect when the `height` field is set.

```rust
use rusty_rich::{Align, VerticalAlignMethod};

let middle = Align::middle("Vertically centered")
    .height(5);
console.println(&middle);
```

```
                             
                             
     Vertically centered     
                             
                             
```

Equivalent to:

```rust
Align::new(renderable).vertical(VerticalAlignMethod::Middle)
```

---

### Combined center + middle

To center both horizontally and vertically, chain both builders:

```rust
use rusty_rich::{Align, AlignMethod, VerticalAlignMethod};

let centered = Align::new("Perfectly centered")
    .align(AlignMethod::Center)
    .vertical(VerticalAlignMethod::Middle)
    .width(40)
    .height(5);
console.println(&centered);
```

```
                                             
                                             
              Perfectly centered             
                                             
                                             
```

---

### left() and right() style

While there is no dedicated `Align::left()` or `Align::right()` constructor, you can achieve left or right alignment directly:

```rust
use rusty_rich::{Align, AlignMethod};

let left_aligned = Align::new("Left")
    .align(AlignMethod::Left);

let right_aligned = Align::new("Right")
    .align(AlignMethod::Right);
```

Since `Left` is the default alignment, `Align::new(...)` alone produces left-aligned output.

---

### Builder Method Reference (Align)

| Method | Description |
|--------|-------------|
| `Align::new(renderable)` | Create a new Align wrapper (defaults: Left, Top) |
| `.align(AlignMethod)` | Set horizontal alignment |
| `.vertical(VerticalAlignMethod)` | Set vertical alignment |
| `.width(usize)` | Set the bounding box width |
| `.height(usize)` | Set the bounding box height |
| `Align::center(renderable)` | Convenience: `new()` + `.align(Center)` |
| `Align::middle(renderable)` | Convenience: `new()` + `.vertical(Middle)` |

---

### Import Paths (Align)

```rust
use rusty_rich::Align;                   // The Align wrapper
use rusty_rich::AlignMethod;             // Horizontal alignment variants
use rusty_rich::VerticalAlignMethod;     // Vertical alignment variants
```

---

## Combining Padding and Align

Padding and Align compose naturally. Wrap an `Align` in `Padding` (or vice versa) to control both spacing and positioning.

```rust
use rusty_rich::{Padding, Align, AlignMethod, VerticalAlignMethod};

let content = Align::new("Hello, world!")
    .align(AlignMethod::Center)
    .vertical(VerticalAlignMethod::Middle)
    .width(30)
    .height(5);

let padded = Padding::new(content)
    .pad(1, 2, 1, 2);

console.println(&padded);
```

This renders centered content inside a 30x5 bounding box, then adds 1 row of padding above and below and 2 columns of padding on each side.

---

## Complete Examples

### Left-indented code block

```rust
use rusty_rich::Padding;

let code = Padding::new("fn main() {\n    println!(\"Hello\");\n}")
    .indent(4);
console.println(&code);
```

```
    fn main() {
        println!("Hello");
    }
```

### Centered title

```rust
use rusty_rich::{Align, AlignMethod, Style, Color, Panel};

let title = Panel::new("System Status Report")
    .border_style(Style::new().color(Color::parse("cyan").unwrap()))
    .padding(0, 2, 0, 2);

let centered = Align::new(title)
    .align(AlignMethod::Center);
console.println(&centered);
```

### Right-aligned annotation

```rust
use rusty_rich::{Align, AlignMethod};

let annotation = Align::new("[ Generated on 2026-06-01 ]")
    .align(AlignMethod::Right);
console.println(&annotation);
```

In a 40-column terminal:

```
                      [ Generated on 2026-06-01 ]
```

### Full-justified paragraph

```rust
use rusty_rich::{Align, AlignMethod};

let paragraph = Align::new(
    "This is an example of full justification applied to a longer text. "
)
    .align(AlignMethod::Full)
    .width(40);
console.println(&paragraph);
```

```
This  is  an  example  of  full
justification  applied  to  a
longer text.
```

The `Full` method distributes extra whitespace evenly between words to fill the line width.
