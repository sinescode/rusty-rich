# Panel

`Panel` draws a bordered container around any renderable. It is the most common way to visually group related content, add context with titles and subtitles, and control the box-drawing style.

```rust
use rusty_rich::Panel;

let panel = Panel::new("Hello, world!");
console.println(&panel);
```

```
╭──────────────╮
│ Hello, world!│
╰──────────────╯
```

---

## new(content)

`Panel::new()` takes any type that implements `Renderable`. This includes plain `&str`, `String`, `Text`, `Table`, `Tree`, `Rule`, another `Panel`, or any custom renderable.

```rust
use rusty_rich::Panel;

// Simple string content
let panel = Panel::new("Hello");

// Other renderables as content
let nested = Panel::new(Panel::new("Nested"));
let table_content = Panel::new(my_table);
```

The default configuration:
- Box style: `BOX_ROUNDED` (rounded corners)
- Title: none
- Subtitle: none
- Title alignment: `AlignMethod::Center`
- Subtitle alignment: `AlignMethod::Center`
- Expand: `true` (fills available width)
- Style: no style
- Border style: no style
- Width: none (auto)
- Height: none (auto)
- Padding: `(0, 1, 0, 1)` (top, right, bottom, left)
- Highlight: `false`

---

## Title and Subtitle

### title()

Sets a title displayed inside the top border.

```rust
use rusty_rich::Panel;

let panel = Panel::new("Content")
    .title("Section 1");
console.println(&panel);
```

```
╭─ Section 1 ───╮
│ Content        │
╰────────────────╯
```

### subtitle()

Sets a subtitle displayed inside the bottom border.

```rust
let panel = Panel::new("Content")
    .title("Network Stats")
    .subtitle("Updated 2s ago");
console.println(&panel);
```

```
╭─ Network Stats ─╮
│ Content          │
╰─ Updated 2s ago ╯
```

### title_align() / subtitle_align()

Control horizontal alignment of the title and subtitle. Uses `AlignMethod`:

| Variant  | Behavior                                |
|----------|-----------------------------------------|
| `Left`   | Aligned to the left side of the border  |
| `Center` | Centered in the border (default)         |
| `Right`  | Aligned to the right side of the border  |
| `Full`   | Justified (behaves like `Left` for titles containing no spaces) |

```rust
use rusty_rich::{Panel, AlignMethod};

let panel = Panel::new("Content")
    .title("Left Title")
    .title_align(AlignMethod::Left)
    .subtitle("Right Subtitle")
    .subtitle_align(AlignMethod::Right);
console.println(&panel);
```

```
╭─ Left Title ───────────╮
│ Content                 │
╰───────────────── Right Subtitle ╯
```

If the title or subtitle text is too long to fit within the panel width, it is omitted from the border (the border line renders as a plain line with no text).

---

## Box Style

The `box_style()` method selects the set of box-drawing characters used for the border. Pass a clone of one of the predefined `BoxStyle` constants from `rusty_rich::box_drawing`.

```rust
use rusty_rich::Panel;
use rusty_rich::box_drawing::BOX_DOUBLE;

let panel = Panel::new("Content")
    .box_style(BOX_DOUBLE.clone());
console.println(&panel);
```

```
╔═══════════╗
║ Content   ║
╚═══════════╝
```

### All box styles

| Constant                | Visual                            | ASCII-safe |
|-------------------------|-----------------------------------|------------|
| `BOX_ROUNDED`           | `╭─╮` / `╰─╯` rounded corners    | No         |
| `BOX_SQUARE`            | `┌─┐` / `└─┘` square corners     | No         |
| `BOX_HEAVY`             | `┏━┓` / `┗━┛` thick borders      | No         |
| `BOX_HEAVY_EDGE`        | `┏━┯┓` heavy outer, light inner  | No         |
| `BOX_HEAVY_HEAD`        | `┏━┳┓` heavy, with header band   | No         |
| `BOX_DOUBLE`            | `╔═╗` / `╚═╝` double lines       | No         |
| `BOX_DOUBLE_EDGE`       | `╔═╤╗` double outer, single inner| No         |
| `BOX_SQUARE_DOUBLE_HEAD`| `┌─┬┐` square, double header sep | No         |
| `BOX_MINIMAL`           | horizontal rules only (no sides)  | No         |
| `BOX_MINIMAL_HEAVY`     | heavy horizontal rules only       | No         |
| `BOX_MINIMAL_DOUBLE_HEAD`| minimal with double header sep  | No         |
| `BOX_SIMPLE`            | no visible border (content only)  | No         |
| `BOX_SIMPLE_HEAVY`      | heavy header rule only            | No         |
| `BOX_SIMPLE_HEAD`       | single rule under header          | No         |
| `BOX_MARKDOWN`          | pipe-markdown style (no top/bottom)| No        |
| `BOX_ASCII`             | `+-+` / `+-+` ASCII characters    | Yes        |
| `BOX_ASCII2`            | `+-++` alternative ASCII          | Yes        |
| `BOX_ASCII_DOUBLE_HEAD` | `+-++` ASCII with `=` header sep  | Yes        |

The default is `BOX_ROUNDED`.

### ASCII fallback

When the terminal's `ConsoleOptions.ascii_only` is `true`, any non-ASCII box style automatically falls back to `BOX_ASCII`. You can rely on this for cross-platform compatibility without conditional logic.

---

## border_style

Applies a `Style` to the border characters (the box-drawing glyphs, title, and subtitle text).

```rust
use rusty_rich::{Panel, Style, Color};

let panel = Panel::new("Content")
    .title("Styled Border")
    .border_style(Style::new().color(Color::parse("cyan").unwrap()));
console.println(&panel);
```

The border style affects:
- All border characters (corners, edges, horizontal/vertical lines)
- Title and subtitle text embedded in the borders

---

## style

Applies a `Style` to the content area (not the border). This sets the default foreground and background for everything inside the panel.

```rust
use rusty_rich::{Panel, Style, Color};

let panel = Panel::new("Dimmed content")
    .style(Style::new().dim(true));
console.println(&panel);
```

When both `style` and `border_style` are set, `style` affects the interior content and `border_style` affects the framing characters independently.

---

## Expand vs Fit

### expand (default: `true`)

By default, a `Panel` expands to fill the full available width of the terminal (or the width of its parent container).

```rust
// Default: panel stretches to terminal width
let panel = Panel::new("Expanded");
console.println(&panel);
```

In a terminal 40 columns wide:
```
╭────────────────────────────────────────╮
│ Expanded                               │
╰────────────────────────────────────────╯
```

### fit()

`.fit()` sets `expand` to `false`, causing the panel to shrink-wrap its content (plus padding and borders). The panel will be only as wide as needed.

```rust
let panel = Panel::new("Short")
    .fit();
console.println(&panel);
```

```
╭───────╮
│ Short │
╰───────╯
```

The panel's minimum width is 3 columns (for the two borders and at least one content column). If the content is empty, a fit panel still renders a minimum-width box.

---

## Width and Height

### width()

Set a fixed width for the panel. The content is rendered within this width (minus 2 columns for the border and any padding).

```rust
let panel = Panel::new("Fixed width content here")
    .width(20);
console.println(&panel);
```

```
╭──────────────────╮
│ Fixed width      │
│ content here     │
╰──────────────────╯
```

If `width` is smaller than the minimum necessary for content, the content is truncated.

### height()

Set a fixed height. If the content produces fewer lines than `height`, the panel is padded with empty lines. If content exceeds `height`, lines beyond the limit are clipped.

```rust
let panel = Panel::new("Tall panel\nLine 2\nLine 3")
    .height(5);
console.println(&panel);
```

```
╭──────────────────╮
│ Tall panel       │
│ Line 2           │
│ Line 3           │
│                  │
╰──────────────────╯
```

---

## Padding

`.padding(top, right, bottom, left)` adds space between the border and the content.

Default padding: `(0, 1, 0, 1)` -- zero vertical padding, one space of horizontal padding on each side.

```rust
// Spacious padding
let panel = Panel::new("Padded content")
    .padding(1, 3, 1, 3);
console.println(&panel);
```

```
╭──────────────────────╮
│                      │
│   Padded content     │
│                      │
╰──────────────────────╯
```

```rust
// Minimal padding (no right padding, some left)
let panel = Panel::new("Custom padding")
    .padding(0, 0, 0, 2);
console.println(&panel);
```

```
╭────────────────────╮
│  Custom padding    │
╰────────────────────╯
```

The padding values are applied in this order:
1. Top: blank lines above content
2. Right: spaces after content on each line
3. Bottom: blank lines below content
4. Left: spaces before content on each line

---

## Highlight

`.highlight(true)` enables console markup interpretation in the title and subtitle strings. When enabled, square-bracket markup syntax within the title or subtitle is styled accordingly.

```rust
let panel = Panel::new("Content")
    .title("[bold cyan]Important[/bold cyan] Section")
    .highlight(true);
console.println(&panel);
```

In this example, the title renders as "Important Section" with "Important" in bold cyan, while the remaining text uses the default border style.

When `highlight` is `false` (the default), markup tags in the title/subtitle are displayed verbatim as literal text.

---

## Full Examples

### Simple panel

```rust
use rusty_rich::Panel;

let panel = Panel::new("Hello, rusty-rich!");
console.println(&panel);
```

```
╭──────────────────────╮
│ Hello, rusty-rich!   │
╰──────────────────────╯
```

### Titled panel with borders styled

```rust
use rusty_rich::{Panel, Style, Color};

let panel = Panel::new("System status: all services running")
    .title("Dashboard")
    .border_style(Style::new().color(Color::parse("cyan").unwrap()))
    .padding(0, 2, 0, 2);
console.println(&panel);
```

### Panel with subtitle

```rust
use rusty_rich::{Panel, Style, Color, AlignMethod};

let panel = Panel::new("CPU: 12%  |  Mem: 3.2 GB / 16 GB  |  Disk: 45%")
    .title("System Monitor")
    .subtitle("Last updated: 12:34:56")
    .title_align(AlignMethod::Left)
    .subtitle_align(AlignMethod::Right)
    .border_style(Style::new().color(Color::parse("bright_black").unwrap()));
console.println(&panel);
```

### Fit vs expand

```rust
use rusty_rich::Panel;

// Expands to terminal width (default)
let wide = Panel::new("This panel stretches across the terminal");

// Fits tightly around content
let tight = Panel::new("Compact").fit();

console.println(&wide);
console.println(&tight);
```

In a 40-column terminal:
```
╭──────────────────────────────────────╮
│ This panel stretches across the      │
│ terminal                             │
╰──────────────────────────────────────╯
╭──────────╮
│ Compact  │
╰──────────╯
```

### Custom box style

```rust
use rusty_rich::{Panel, Style, Color};
use rusty_rich::box_drawing::{BOX_DOUBLE, BOX_HEAVY, BOX_ASCII};

// Double borders
let double = Panel::new("Double-lined panel")
    .box_style(BOX_DOUBLE.clone())
    .border_style(Style::new().color(Color::parse("green").unwrap()));

// Heavy borders
let heavy = Panel::new("Heavy border panel")
    .box_style(BOX_HEAVY.clone())
    .border_style(Style::new().color(Color::parse("yellow").unwrap()));

// ASCII-only borders (safe for legacy terminals)
let ascii = Panel::new("ASCII-safe panel")
    .box_style(BOX_ASCII.clone());

console.println(&double);
console.println(&heavy);
console.println(&ascii);
```

Output:
```
╔════════════════════════╗
║ Double-lined panel     ║
╚════════════════════════╝
┏━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Heavy border panel     ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━┛
+------------------------+
| ASCII-safe panel       |
+------------------------+
```

### Nested panels

Panels can be nested by passing another `Panel` as the content:

```rust
use rusty_rich::{Panel, Style, Color};
use rusty_rich::box_drawing::BOX_DOUBLE;

let inner = Panel::new("Inner content")
    .box_style(BOX_DOUBLE.clone())
    .border_style(Style::new().color(Color::parse("yellow").unwrap()))
    .padding(0, 1, 0, 1);

let outer = Panel::new(inner)
    .title("Outer Panel")
    .border_style(Style::new().color(Color::parse("magenta").unwrap()));

console.println(&outer);
```

Output (conceptual):
```
╭──────────────────────────────────╮
│ Outer Panel                      │
│ ╔══════════════════╗             │
│ ║ Inner content    ║             │
│ ╚══════════════════╝             │
╰──────────────────────────────────╯
```

---

## Builder Method Reference

| Method                    | Description                                            |
|---------------------------|--------------------------------------------------------|
| `.title("...")`           | Title text in the top border                           |
| `.subtitle("...")`        | Subtitle text in the bottom border                     |
| `.title_align(AlignMethod)` | Title alignment: `Left`, `Center` (default), `Right`, `Full` |
| `.subtitle_align(AlignMethod)` | Subtitle alignment (same options)                  |
| `.box_style(BoxStyle)`    | Box-drawing character set                              |
| `.border_style(Style)`    | Style applied to border characters and title/subtitle  |
| `.style(Style)`           | Default style for the content area                     |
| `.expand` / `.fit()`      | `expand` is default (fill width); `.fit()` shrink-wraps |
| `.width(usize)`           | Fixed panel width                                      |
| `.height(usize)`          | Fixed panel height                                     |
| `.padding(top, right, bottom, left)` | Inner padding (default `0, 1, 0, 1`)         |
| `.highlight(bool)`        | Enable markup interpretation in title/subtitle         |

---

## Import Paths

```rust
use rusty_rich::Panel;                                  // The Panel type
use rusty_rich::box_drawing::BOX_ROUNDED;               // Box style constants
use rusty_rich::box_drawing::BOX_DOUBLE;
use rusty_rich::box_drawing::BOX_HEAVY;
use rusty_rich::box_drawing::BOX_ASCII;
// ... and all other box style constants
use rusty_rich::AlignMethod;                            // Title/subtitle alignment
use rusty_rich::Style;                                  // Border and content styling
use rusty_rich::Color;                                  // Color values for styles
```
