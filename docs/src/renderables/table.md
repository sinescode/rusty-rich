# Table

`Table` renders tabular data with columns, headers, footers, and borders. It supports
colspan/rowspan, section separators, alternating row styles, configurable box-drawing
styles, and title/caption labels.

The module provides three primary types: `Column`, `Cell`, and `Table`.

---

## Column

A `Column` defines the properties for one column in a table.

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `header` | `String` | (required) | Header text rendered in the header row. |
| `footer` | `String` | `""` | Footer text rendered in the footer row. |
| `header_style` | `Style` | `bold(true)` | Style applied to the header text. |
| `footer_style` | `Style` | plain | Style applied to the footer text. |
| `style` | `Style` | plain | Default style for cells in this column. |
| `justify` | `AlignMethod` | `Left` | Horizontal text alignment (`Left`, `Center`, `Right`). |
| `vertical` | `VerticalAlignMethod` | `Top` | Vertical alignment (`Top`, `Middle`, `Bottom`). |
| `overflow` | `OverflowMethod` | `Ellipsis` | How to handle overflow (`Ellipsis`, `Fold`, `Crop`). |
| `width` | `Option<usize>` | `None` | Fixed column width in characters. |
| `min_width` | `Option<usize>` | `None` | Minimum column width (used in ratio distribution). |
| `max_width` | `Option<usize>` | `None` | Maximum column width. |
| `ratio` | `Option<usize>` | `None` | Proportional width weight for flexible distribution. |
| `colspan` | `usize` | `1` | Number of columns this header spans. |

### Construction

```rust
use rusty_rich::table::Column;
use rusty_rich::align::AlignMethod;
use rusty_rich::style::Style;

let col = Column::new("Name")
    .justify(AlignMethod::Left)
    .width(20)
    .min_width(10)
    .max_width(30)
    .ratio(2)
    .style(Style::new().cyan())
    .header_style(Style::new().bold(true).white())
    .overflow(OverflowMethod::Ellipsis);
```

### Builder methods

- `justify(m)` -- set horizontal alignment.
- `width(w)` -- set a fixed width.
- `min_width(w)` -- set a minimum width.
- `max_width(w)` -- set a maximum width.
- `style(s)` -- set the default cell style.
- `header_style(s)` -- set the header style.
- `ratio(r)` -- set the ratio weight.
- `overflow(o)` -- set the overflow method.

---

## Cell

A `Cell` represents a single cell in a data row. Cells are the unit of content passed
to `add_row`. String values passed to `add_row_str` are automatically wrapped in a
default `Cell`.

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `content` | `String` | (required) | The cell text. |
| `style` | `Option<Style>` | `None` | Optional per-cell style (overrides column defaults). |
| `colspan` | `usize` | `1` | Number of columns this cell spans horizontally. |
| `rowspan` | `usize` | `1` | Number of rows this cell spans vertically. |

### Construction

```rust
use rusty_rich::table::Cell;
use rusty_rich::style::Style;

let cell = Cell::new("Sales")
    .colspan(2)
    .rowspan(1)
    .style(Style::new().bold(true).green());
```

`Cell` implements `From<&str>` and `From<String>`, so plain strings convert
automatically where a `Cell` is expected.

### Builder methods

- `style(s)` -- set the cell style.
- `colspan(c)` -- set the colspan.
- `rowspan(r)` -- set the rowspan.

---

## Table construction

### `Table::new()`

Creates a standard table with outer borders, a visible header row, and the
`BOX_HEAVY_HEAD` box style as default.

```rust
use rusty_rich::table::{Table, Column};

let mut table = Table::new();
table.add_column(Column::new("Name"));
table.add_column(Column::new("Value"));
```

### `Table::grid()`

Creates a grid-style table without outer borders, without a header, and without a
footer. Equivalent to calling `new()` then chaining `.hide_header()` and setting
`show_edge: false` with `BOX_SIMPLE` as the box style. Useful for lightweight layouts
where only column separators are desired.

```rust
use rusty_rich::table::{Table, Column};

let mut table = Table::grid();
table.add_column(Column::new("Key"));
table.add_column(Column::new("Value"));
table.add_row_str(vec!["Host".into(), "localhost".into()]);
table.add_row_str(vec!["Port".into(), "8080".into()]);
```

---

## Adding columns

### `add_column(column: Column)`

Appends a `Column` definition. Columns must be added before rows. The number of
columns determines the expected width of each row.

```rust
table.add_column(
    Column::new("Product")
        .justify(AlignMethod::Left)
        .ratio(2)
);
table.add_column(
    Column::new("Price")
        .justify(AlignMethod::Right)
        .ratio(1)
);
```

The builder-style equivalent is `column(col)`:

```rust
let table = Table::new()
    .column(Column::new("A"))
    .column(Column::new("B"));
```

---

## Adding rows

### `add_row(row: Vec<Cell>)`

Adds a row of `Cell` objects, supporting colspan and rowspan.

```rust
use rusty_rich::table::Cell;

table.add_row(vec![
    Cell::new("North Region").colspan(2),
    Cell::new("$42,000"),
]);
table.add_row(vec![
    Cell::new("East"),
    Cell::new("West"),
    Cell::new("$38,000"),
]);
```

### `add_row_str(row: Vec<String>)`

Convenience method that wraps each `String` in a default `Cell` (colspan=1, rowspan=1,
no style) and adds the row.

```rust
table.add_row_str(vec![
    "Widget A".into(),
    "250".into(),
    "$12.99".into(),
]);
```

The builder-style equivalent is `row(cells)` and `row_str(strings)`:

```rust
let table = Table::new()
    .column(Column::new("Item"))
    .column(Column::new("Qty"))
    .row_str(vec!["Hammer".into(), "10".into()])
    .row_str(vec!["Nails".into(), "500".into()]);
```

---

## Title and caption

A **title** is rendered above the table. A **caption** is rendered below the table.
Both are centered by default.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `title` | `Option<String>` | `None` | Text displayed above the top border. |
| `caption` | `Option<String>` | `None` | Text displayed below the bottom border. |
| `title_style` | `Style` | `bold(true)` | Style for the title. |
| `caption_style` | `Style` | `dim(true)` | Style for the caption. |
| `title_justify` | `AlignMethod` | `Center` | Justification of the title. |
| `caption_justify` | `AlignMethod` | `Center` | Justification of the caption. |

```rust
let table = Table::new()
    .title("Sales Report 2025")
    .caption("All figures in USD")
    .column(Column::new("Quarter"))
    .column(Column::new("Revenue"))
    .row_str(vec!["Q1".into(), "$100K".into()])
    .row_str(vec!["Q2".into(), "$120K".into()]);
```

---

## Visibility toggles

| Method/Field | Default | Description |
|-------------|---------|-------------|
| `show_header` | `true` | Show or hide the header row. |
| `show_footer` | `false` | Show or hide the footer row. |
| `show_edge` | `true` | Show or hide the outer border. |
| `show_lines` | `false` | Show lines between every row. |
| `.hide_header()` | -- | Builder method: sets `show_header = false`. |
| `.show_lines()` | -- | Builder method: sets `show_lines = true`. |

Hide the header:

```rust
let table = Table::new()
    .hide_header()
    .column(Column::new("Key"))
    .column(Column::new("Value"));
```

Show row lines:

```rust
let table = Table::new()
    .show_lines()
    .column(Column::new("A"))
    .column(Column::new("B"));
```

---

## Box style

The table's border appearance is controlled by the `box_style` field. It accepts any
`BoxStyle` from the `box_drawing` module. The default is `BOX_HEAVY_HEAD` (heavy
borders on the top and header separator, light elsewhere).

```rust
use rusty_rich::box_drawing::{BOX_SQUARE, BOX_ROUNDED, BOX_DOUBLE, BOX_MINUS, BOX_SIMPLE};

let table = Table::new()
    .box_style(BOX_ROUNDED.clone())
    .column(Column::new("Item"))
    .column(Column::new("Count"));
```

Available box styles:

| Constant | Description |
|----------|-------------|
| `BOX_SQUARE` | Standard light unicode box drawing. |
| `BOX_ROUNDED` | Light with rounded corners. |
| `BOX_HEAVY` | Heavy (thick) lines everywhere. |
| `BOX_HEAVY_HEAD` | Heavy top/header, light body. |
| `BOX_DOUBLE` | Double-line borders. |
| `BOX_MINUS` | Minimal: dashes and pipes. |
| `BOX_SIMPLE` | Simple single-line style. |
| `BOX_ASCII` | Pure ASCII (`+`, `-`, `|`). |

The `border_style` field controls the ANSI style applied to the border characters:

```rust
use rusty_rich::style::Style;

table.border_style = Style::new().cyan();
```

---

## Row styles

`row_styles` is a `Vec<Style>` that defines alternating row styling. When exactly two
styles are provided, they cycle (even/odd rows). When more are provided, each index
maps to that row position.

```rust
use rusty_rich::style::Style;

let mut table = Table::new();
table.row_styles = vec![
    Style::new(),                       // odd rows
    Style::new().dim(true).cyan(),     // even rows
];
```

---

## Section separators

`add_section()` inserts a horizontal rule before the next row that is added. This is
useful for grouping rows into visually separated sections.

```rust
let mut table = Table::new();
table.add_column(Column::new("Department"));
table.add_column(Column::new("Headcount"));

table.add_row_str(vec!["Engineering".into(), "42".into()]);
table.add_row_str(vec!["Design".into(), "15".into()]);

table.add_section();

table.add_row_str(vec!["Marketing".into(), "28".into()]);
table.add_row_str(vec!["Sales".into(), "34".into()]);
```

Renders with a separator line between Design and Marketing.

---

## Leading

`leading` controls the number of blank lines between rows. Default is `0`.

```rust
let table = Table::new()
    .column(Column::new("Item"))
    .column(Column::new("Price"))
    .row_str(vec!["A".into(), "10".into()])
    .row_str(vec!["B".into(), "20".into()])
    .leading(1);
```

---

## Padding

`padding` is a 4-tuple `(top, right, bottom, left)` in characters. Default is
`(0, 1, 0, 1)`, meaning one space on each side horizontally, no vertical padding.

```rust
// 1 space vertically, 2 spaces horizontally
table.padding = (1, 2, 1, 2);
```

---

## Complete examples

### Basic table

```rust
use rusty_rich::table::{Table, Column};
use rusty_rich::style::Style;

let mut table = Table::new();
table.add_column(Column::new("Name").style(Style::new().bold(true)));
table.add_column(Column::new("Role"));
table.add_column(Column::new("Salary").justify(rusty_rich::align::AlignMethod::Right));

table.add_row_str(vec![
    "Alice".into(),
    "Engineer".into(),
    "$120,000".into(),
]);
table.add_row_str(vec![
    "Bob".into(),
    "Designer".into(),
    "$95,000".into(),
]);
table.add_row_str(vec![
    "Carol".into(),
    "Manager".into(),
    "$140,000".into(),
]);
```

### Grid table

```rust
use rusty_rich::table::{Table, Column};

let mut table = Table::grid();
table.add_column(Column::new("Setting"));
table.add_column(Column::new("Value"));

table.add_row_str(vec!["Host".into(), "127.0.0.1".into()]);
table.add_row_str(vec!["Port".into(), "5432".into()]);
table.add_row_str(vec!["User".into(), "admin".into()]);
table.add_row_str(vec!["DB".into(), "inventory".into()]);
```

### Table with title and caption

```rust
use rusty_rich::table::{Table, Column};

let table = Table::new()
    .title("Q4 Financial Summary")
    .caption("Unaudited figures — for internal use only")
    .column(Column::new("Metric"))
    .column(Column::new("Target"))
    .column(Column::new("Actual"))
    .row_str(vec!["Revenue".into(), "$500K".into(), "$540K".into()])
    .row_str(vec!["Costs".into(),  "$300K".into(), "$285K".into()])
    .row_str(vec!["Margin".into(),  "40%".into(),   "47%".into()]);
```

### Table with colspan

```rust
use rusty_rich::table::{Table, Column, Cell};
use rusty_rich::align::AlignMethod;
use rusty_rich::style::Style;

let mut table = Table::new();
table.add_column(Column::new("Region"));
table.add_column(Column::new("Q1"));
table.add_column(Column::new("Q2"));
table.add_column(Column::new("Q3"));
table.add_column(Column::new("Q4"));

// Header row: "Region" spans 1, "Quarterly Revenue" spans 4 columns
table.add_row(vec![
    Cell::new("Region").style(Style::new().bold(true)),
    Cell::new("Quarterly Revenue").colspan(4)
        .style(Style::new().bold(true))
        .justify(rusty_rich::align::AlignMethod::Center),
]);

table.add_row_str(vec![
    "North".into(),
    "$80K".into(),
    "$90K".into(),
    "$95K".into(),
    "$110K".into(),
]);

table.add_row_str(vec![
    "South".into(),
    "$60K".into(),
    "$65K".into(),
    "$70K".into(),
    "$85K".into(),
]);
```

### Table with sections

```rust
use rusty_rich::table::{Table, Column};
use rusty_rich::style::Style;

let mut table = Table::new();
table.add_column(Column::new("Product"));
table.add_column(Column::new("Stock"));
table.add_column(Column::new("Price"));

table.add_row_str(vec!["Widget A".into(), "150".into(), "$9.99".into()]);
table.add_row_str(vec!["Widget B".into(), "200".into(), "$14.99".into()]);

table.add_section();

table.add_row_str(vec!["Gadget X".into(), "75".into(), "$29.99".into()]);
table.add_row_str(vec!["Gadget Y".into(), "120".into(), "$39.99".into()]);

table.add_section();

table.add_row_str(vec!["Service Plan".into(), "—".into(), "$99.00".into()]);
```

The section rows render with a horizontal separator line between each group.
