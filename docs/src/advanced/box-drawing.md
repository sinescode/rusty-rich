# Box Drawing

The `BoxStyle` type defines the characters used to draw borders around panels, tables, and other framed renderables. Each style is an 8x4 grid of Unicode or ASCII glyphs that control corners, edges, dividers, and intersections.

---

## The BoxStyle Struct

`BoxStyle` stores 33 fields representing every character position in a bordered box layout. The positions map to an 8-line, 4-column definition string.

### 8-Line Layout

```text
┌─┬┐   line 0: top            (top_left, top, top_divider, top_right)
│ ││   line 1: head           (head_left, head_horizontal, head_vertical, head_right)
├─┼┤   line 2: head_row       (head_row_left, head_row_horizontal, head_row_cross, head_row_right)
│ ││   line 3: mid            (mid_left, mid_horizontal, mid_vertical, mid_right)
├─┼┤   line 4: row            (row_left, row_horizontal, row_cross, row_right)
├─┼┤   line 5: foot_row       (foot_row_left, foot_row_horizontal, foot_row_cross, foot_row_right)
│ ││   line 6: foot           (foot_left, foot_horizontal, foot_vertical, foot_right)
└─┴┘   line 7: bottom         (bottom_left, bottom, bottom_divider, bottom_right)
```

Each of the 8 lines has exactly 4 characters. The column positions across all 8 lines follow a consistent pattern:

| Column | Position   | Description                             |
|--------|------------|-----------------------------------------|
| 0      | Left edge  | Leftmost vertical or corner character   |
| 1      | Horizontal | Horizontal line character               |
| 2      | Divider    | Vertical divider / cross character      |
| 3      | Right edge | Rightmost vertical or corner character  |

### Fields

The 32 character fields plus the `ascii` boolean:

| Line       | Field                | Type    | Description                              |
|------------|----------------------|---------|------------------------------------------|
| top        | `top_left`           | `char`  | Top-left corner                          |
| top        | `top`                | `char`  | Top horizontal edge                      |
| top        | `top_divider`        | `char`  | Top vertical divider (column separator)  |
| top        | `top_right`          | `char`  | Top-right corner                         |
| head       | `head_left`          | `char`  | Left edge in header row                  |
| head       | `head_horizontal`    | `char`  | Horizontal fill in header row            |
| head       | `head_vertical`      | `char`  | Vertical divider in header row           |
| head       | `head_right`         | `char`  | Right edge in header row                 |
| head_row   | `head_row_left`      | `char`  | Separator under header, left junction    |
| head_row   | `head_row_horizontal`| `char`  | Separator under header, horizontal line  |
| head_row   | `head_row_cross`     | `char`  | Separator under header, cross            |
| head_row   | `head_row_right`     | `char`  | Separator under header, right junction   |
| mid        | `mid_left`           | `char`  | Left edge, mid row (no line separators)  |
| mid        | `mid_horizontal`     | `char`  | Horizontal fill, mid row                 |
| mid        | `mid_vertical`       | `char`  | Vertical divider, mid row                |
| mid        | `mid_right`          | `char`  | Right edge, mid row                      |
| row        | `row_left`           | `char`  | Row separator, left junction             |
| row        | `row_horizontal`     | `char`  | Row separator, horizontal line           |
| row        | `row_cross`          | `char`  | Row separator, cross intersection        |
| row        | `row_right`          | `char`  | Row separator, right junction            |
| foot_row   | `foot_row_left`      | `char`  | Separator before footer, left junction   |
| foot_row   | `foot_row_horizontal`| `char`  | Separator before footer, horizontal line |
| foot_row   | `foot_row_cross`     | `char`  | Separator before footer, cross           |
| foot_row   | `foot_row_right`     | `char`  | Separator before footer, right junction  |
| foot       | `foot_left`          | `char`  | Left edge, footer row                    |
| foot       | `foot_horizontal`    | `char`  | Horizontal fill, footer row              |
| foot       | `foot_vertical`      | `char`  | Vertical divider, footer row             |
| foot       | `foot_right`         | `char`  | Right edge, footer row                   |
| bottom     | `bottom_left`        | `char`  | Bottom-left corner                       |
| bottom     | `bottom`             | `char`  | Bottom horizontal edge                   |
| bottom     | `bottom_divider`     | `char`  | Bottom vertical divider                  |
| bottom     | `bottom_right`       | `char`  | Bottom-right corner                      |
|            | `ascii`              | `bool`  | `true` if this style uses only ASCII     |

---

## Predefined Styles

All predefined styles are exported as `Lazy<BoxStyle>` statics from `rusty_rich::box_drawing`. They are lazily initialized on first access.

### ROUNDED

The default box style for `Panel`. Uses rounded corner glyphs.

```
╭─┬╮
│ ││
├─┼┤
│ ││
├─┼┤
├─┼┤
│ ││
╰─┴╯
```

```rust
use rusty_rich::box_drawing::BOX_ROUNDED;
```

Example rendered panel:
```
╭──────────────────────╮
│ Rounded panel        │
╰──────────────────────╯
```

### SQUARE

Square corners with light-weight lines.

```
┌─┬┐
│ ││
├─┼┤
│ ││
├─┼┤
├─┼┤
│ ││
└─┴┘
```

```rust
use rusty_rich::box_drawing::BOX_SQUARE;
```

Example:
```
┌──────────────────────┐
│ Square panel         │
└──────────────────────┘
```

### HEAVY

Thick (heavy) lines throughout.

```
┏━┳┓
┃ ┃┃
┣━╋┫
┃ ┃┃
┣━╋┫
┣━╋┫
┃ ┃┃
┗━┻┛
```

```rust
use rusty_rich::box_drawing::BOX_HEAVY;
```

Example:
```
┏━━━━━━━━━━━━━━━━━━━━━━┓
┃ Heavy panel           ┃
┗━━━━━━━━━━━━━━━━━━━━━━┛
```

### HEAVY_EDGE

Heavy outer border with light inner dividers.

```
┏━┯┓
┃ │┃
┠─┼┨
┃ │┃
┠─┼┨
┠─┼┨
┃ │┃
┗━┷┛
```

```rust
use rusty_rich::box_drawing::BOX_HEAVY_EDGE;
```

### HEAVY_HEAD

Heavy border with a heavy header separator and light inner rows.

```
┏━┳┓
┃ ┃┃
┡━╇┩
│ ││
├─┼┤
├─┼┤
│ ││
└─┴┘
```

```rust
use rusty_rich::box_drawing::BOX_HEAVY_HEAD;
```

### DOUBLE

Double-line borders throughout.

```
╔═╦╗
║ ║║
╠═╬╣
║ ║║
╠═╬╣
╠═╬╣
║ ║║
╚═╩╝
```

```rust
use rusty_rich::box_drawing::BOX_DOUBLE;
```

Example:
```
╔══════════════════════╗
║ Double panel         ║
╚══════════════════════╝
```

### DOUBLE_EDGE

Double outer border with single-line inner dividers.

```
╔═╤╗
║ │║
╟─┼╢
║ │║
╟─┼╢
╟─┼╢
║ │║
╚═╧╝
```

```rust
use rusty_rich::box_drawing::BOX_DOUBLE_EDGE;
```

### SQUARE_DOUBLE_HEAD

Square corners with a double-line header separator.

```
┌─┬┐
│ ║│
├─╪┤
│ ││
├─┼┤
├─┼┤
│ ││
└─┴┘
```

```rust
use rusty_rich::box_drawing::BOX_SQUARE_DOUBLE_HEAD;
```

Notice the `head_vertical` (`║`) and `head_row_cross` (`╪`) positions use double-line characters while the rest of the box uses single lines.

### SIMPLE

No visible border -- only content. All characters are spaces.

```




```

```rust
use rusty_rich::box_drawing::BOX_SIMPLE;
```

### SIMPLE_HEAVY

No side borders, only a heavy horizontal rule under the header / before the footer.

```




   ━┿


```

```rust
use rusty_rich::box_drawing::BOX_SIMPLE_HEAVY;
```

### SIMPLE_HEAD

A single light horizontal rule under the header section.

```




  ━┿ 

```

```rust
use rusty_rich::box_drawing::BOX_SIMPLE_HEAD;
```

### MINIMAL

Minimal style with light horizontal rules (no side borders). Used for a clean, understated look.

```
  ╌
  ╌
  ╌



  ╌ 
```

```rust
use rusty_rich::box_drawing::BOX_MINIMAL;
```

### MINIMAL_HEAVY

Same as `MINIMAL` but with heavy horizontal rules (`╍`).

```
  ╍
  ╍
  ╍



  ╍ 
```

```rust
use rusty_rich::box_drawing::BOX_MINIMAL_HEAVY;
```

### MINIMAL_DOUBLE_HEAD

Minimal style with a double horizontal rule for the header separator.

```
  ═ 
  ═ 
  ═ 
    

    
  ═ 
    
```

```rust
use rusty_rich::box_drawing::BOX_MINIMAL_DOUBLE_HEAD;
```

### ASCII

Fully ASCII-compatible box using `+`, `-`, and `|` characters.

```
+--+
| ||
|-+|
| ||
|-+|
|-+|
| ||
+-++
```

```rust
use rusty_rich::box_drawing::BOX_ASCII;
```

Example:
```
+----------------------+
| ASCII panel          |
+----------------------+
```

This is the automatic fallback when the terminal is in ASCII-only mode.

### ASCII2

Alternative ASCII style with fewer intersections.

```
+-++
| ||
| ||
| ||
| ||
| ||
| ||
+-++
```

```rust
use rusty_rich::box_drawing::BOX_ASCII2;
```

### ASCII_DOUBLE_HEAD

ASCII style with an `=`-based double header separator.

```
+-++
| ||
+=+|
| ||
|-+|
|-+|
| ||
+-++
```

```rust
use rusty_rich::box_drawing::BOX_ASCII_DOUBLE_HEAD;
```

### MARKDOWN

Markdown pipe-table style -- no outer border, uses `|` and `-` characters like a Markdown table.

```

| ||
|-||
| ||
|-||
|-||
| ||

```

```rust
use rusty_rich::box_drawing::BOX_MARKDOWN;
```

Example table render:

```
| Name     | Value |
|----------|-------|
| Alpha    | 100   |
| Beta     | 200   |
```

### Quick Reference Table

| Constant                | Outer Border | Inner Dividers | Header Sep | ASCII-safe |
|-------------------------|-------------|----------------|------------|------------|
| `BOX_ROUNDED`           | `╭─╮╰─╯`    | single         | single     | No         |
| `BOX_SQUARE`            | `┌─┐└─┘`    | single         | single     | No         |
| `BOX_HEAVY`             | `┏━┓┗━┛`    | heavy          | heavy      | No         |
| `BOX_HEAVY_EDGE`        | heavy       | single         | single     | No         |
| `BOX_HEAVY_HEAD`        | heavy       | single         | heavy      | No         |
| `BOX_DOUBLE`            | `╔═╗╚═╝`    | double         | double     | No         |
| `BOX_DOUBLE_EDGE`       | double      | single         | single     | No         |
| `BOX_SQUARE_DOUBLE_HEAD`| square      | single         | double     | No         |
| `BOX_MINIMAL`           | none        | none           | light      | No         |
| `BOX_MINIMAL_HEAVY`     | none        | none           | heavy      | No         |
| `BOX_MINIMAL_DOUBLE_HEAD`| none       | none           | double     | No         |
| `BOX_SIMPLE`            | none        | none           | none       | Yes        |
| `BOX_SIMPLE_HEAVY`      | none        | none           | heavy      | Yes        |
| `BOX_SIMPLE_HEAD`       | none        | none           | light      | Yes        |
| `BOX_MARKDOWN`          | none        | pipe `\|`      | `-`        | Yes        |
| `BOX_ASCII`             | `+-+`       | `+`, `-`, `\|`  | `+`, `-`   | Yes        |
| `BOX_ASCII2`            | `+-+`       | minimal        | none       | Yes        |
| `BOX_ASCII_DOUBLE_HEAD` | `+-+`       | `+`, `-`, `\|`  | `=`        | Yes        |

---

## Import Paths

All box styles live under `rusty_rich::box_drawing`:

```rust
use rusty_rich::box_drawing::{
    BoxStyle,                // The struct
    BOX_ROUNDED,             // Predefined styles
    BOX_SQUARE,
    BOX_HEAVY,
    BOX_HEAVY_EDGE,
    BOX_HEAVY_HEAD,
    BOX_DOUBLE,
    BOX_DOUBLE_EDGE,
    BOX_SQUARE_DOUBLE_HEAD,
    BOX_SIMPLE,
    BOX_SIMPLE_HEAVY,
    BOX_SIMPLE_HEAD,
    BOX_MINIMAL,
    BOX_MINIMAL_HEAVY,
    BOX_MINIMAL_DOUBLE_HEAD,
    BOX_ASCII,
    BOX_ASCII2,
    BOX_ASCII_DOUBLE_HEAD,
    BOX_MARKDOWN,
    get_safe_box,            // ASCII fallback helper
};
```

---

## Using Box Styles

### With Panel

Pass a cloned `BoxStyle` to `Panel::box_style()`:

```rust
use rusty_rich::Panel;
use rusty_rich::box_drawing::BOX_DOUBLE;

let panel = Panel::new("Important notice")
    .box_style(BOX_DOUBLE.clone())
    .title("Notice");
```

### With Table

Set the box style on a `Table` to control its border glyphs:

```rust
use rusty_rich::table::Table;
use rusty_rich::box_drawing::BOX_HEAVY;

let table = Table::new()
    .box_style(BOX_HEAVY.clone());
```

---

## get_safe_box() -- ASCII Fallback

When you know the terminal supports only ASCII (for example, on Windows legacy consoles), use `get_safe_box()` to substitute an ASCII-compatible style:

```rust
use rusty_rich::box_drawing::{get_safe_box, BOX_ROUNDED, BOX_ASCII};

// If ascii_only is true, returns BOX_ASCII; otherwise returns the original
let safe = get_safe_box(&BOX_ROUNDED, ascii_only);
assert_eq!(safe.ascii, true || !ascii_only);
```

The function checks two things:

1. If `ascii_only` is `false`, it returns the input style unchanged.
2. If `ascii_only` is `true` and the input style already has `ascii == true`, it returns the input style unchanged.
3. If `ascii_only` is `true` and the input style has `ascii == false`, it returns `BOX_ASCII.clone()`.

This is used internally by `Panel` and `Table` when `ConsoleOptions.ascii_only` is set, so in most cases you do not need to call it manually.

The built-in ASCII-safe styles (those with `ascii: true`) are:

- `BOX_ASCII`
- `BOX_ASCII2`
- `BOX_ASCII_DOUBLE_HEAD`
- `BOX_SIMPLE`
- `BOX_SIMPLE_HEAVY`
- `BOX_SIMPLE_HEAD`
- `BOX_MARKDOWN`

---

## Custom Box Styles

Create a custom box style by passing an 8-line, 4-column string to `BoxStyle::from_str()`.

### from_str(string, ascii)

The string must have exactly 8 lines, each containing exactly 4 characters. The `ascii` parameter marks whether the style is compatible with ASCII-only terminals (used by `get_safe_box`).

```rust
use rusty_rich::box_drawing::BoxStyle;

let custom_def = "\
★★★★
★ ★★
★★★★
★ ★★
★★★★
★★★★
★ ★★
★★★★";

let custom_box = BoxStyle::from_str(custom_def, false);

// Use with a panel
use rusty_rich::Panel;

let panel = Panel::new("Star-bordered content")
    .box_style(custom_box);
```

The rendered panel would use star characters for all border positions:

```
★★★★★★★★★★★★★★★★★★★★
★ Star-bordered content ★
★★★★★★★★★★★★★★★★★★★★
```

### Another example -- dotted lines

```rust
use rusty_rich::box_drawing::BoxStyle;

let dotted = BoxStyle::from_str("\
┌─┬┐
│ ·│
├─┼┤
│ ·│
├─┼┤
├─┼┤
│ ·│
└─┴┘", false);

// This replaces the vertical fill in header/mid/foot rows
// with middle-dot characters while keeping standard corners.
```

### Tips for custom styles

- **Spaces are valid characters.** Styles like `SIMPLE` use spaces to create invisible borders.
- **Keep character widths consistent.** Mixing half-width and full-width characters may produce alignment artifacts.
- **Use the `ascii` flag correctly.** Set it to `true` only if every character in the definition is an ASCII printable character (code point < 128).
- **The `to_string()` method** round-trips the definition so you can inspect a style's raw definition at any time:

```rust
println!("{}", BOX_HEAVY.to_string());
// Output:
// ┏━┳┓
// ┃ ┃┃
// ┣━╋┫
// ┃ ┃┃
// ┣━╋┫
// ┣━╋┫
// ┃ ┃┃
// ┗━┻┛
```

---

## Implementation Details

- Box styles are parsed at runtime via `Lazy` initialization. The definition strings are stored as `&str` constants and parsed into `BoxStyle` structs on first access.
- The struct derives `Debug`, `Clone`, `PartialEq`, and `Eq`.
- `BoxStyle` implements `Display`, which delegates to `to_string()` to produce the 8-line definition.
- `get_safe_box()` is the only public free function in `box_drawing` (besides the style constructors).
