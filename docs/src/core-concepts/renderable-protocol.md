# The Renderable Protocol

At the heart of rusty-rich is the `Renderable` trait. Anything that can be
displayed by the `Console` implements `Renderable`. The trait defines a single
required method, `render()`, and one optional hook, `measure()`, for layout
negotiation.

Understanding the renderable protocol unlocks the ability to build your own
custom display components that integrate seamlessly with rusty-rich's
composability model -- panels that wrap your widgets, tables that contain
your data, and a `Console` that can print them all uniformly.

---

## The `Renderable` Trait

The trait lives in `crate::console` and is re-exported at the crate root:

```rust
pub trait Renderable {
    fn render(&self, options: &ConsoleOptions) -> RenderResult;

    fn measure(&self, _options: &ConsoleOptions) -> Option<Measurement> {
        None
    }
}
```

### `render(&self, options: &ConsoleOptions) -> RenderResult`

This is the only mandatory method. It receives a `ConsoleOptions` describing
the rendering context (terminal width, height, overflow behaviour, etc.) and
must return a `RenderResult`.

The `ConsoleOptions` struct carries everything a renderable might need:

```rust
pub struct ConsoleOptions {
    pub size: ConsoleDimensions,    // Terminal cell dimensions
    pub is_terminal: bool,          // True if output goes to a terminal
    pub encoding: String,           // "utf-8" in practice
    pub min_width: usize,           // Minimum allowed render width
    pub max_width: usize,           // Maximum allowed render width
    pub max_height: usize,          // Maximum allowed render height
    pub justify: Option<AlignMethod>,   // Override justification
    pub overflow: Option<OverflowMethod>,  // Override overflow handling
    pub no_wrap: bool,              // Disable wrapping
    pub ascii_only: bool,           // Use ASCII-only box characters
    pub markup: bool,               // Enable markup interpretation
    pub highlight: bool,            // Enable syntax highlighting
    pub height: Option<usize>,      // Fixed height override
    pub legacy_windows: bool,       // Legacy Windows console
}
```

Renderables should respect `options.max_width` and `options.max_height` as
hard constraints -- the output must fit within those bounds.

### `measure(&self, options: &ConsoleOptions) -> Option<Measurement>`

The optional `measure()` method returns width constraints for the renderable.
It is used by layout engines (columns, tables, panels) to determine how much
horizontal space each child needs.

```rust
pub struct Measurement {
    pub minimum: usize,  // Narrowest the renderable can be
    pub maximum: usize,  // Widest the renderable wants to be
}
```

- **minimum**: the smallest width at which the renderable is still readable
  or functional.
- **maximum**: the width the renderable would naturally take if unconstrained.

Returning `None` (the default) tells the layout engine to fall back to
rendering the content and measuring the output segments.

Concrete example from `crate::rule::Rule` --- a rule with a 25-character title
in a 60-wide terminal has a minimum of 4 (the `" ── "` around the title) and a
maximum of 60:

```rust
impl Renderable for Rule {
    fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
        let title_w = UnicodeWidthStr::width(self.title.as_str());
        let min = if title_w > 0 { title_w + 4 } else { 1 };
        Some(Measurement::new(min, options.max_width))
    }
}
```

---

## `RenderResult`

The return type of `render()`. It holds either a pre-computed list of
segment-lines, or a list of `RenderItem`s that the Console will recursively
flatten, or both.

```rust
pub struct RenderResult {
    pub lines: Vec<Vec<Segment>>,    // Flat line-oriented segments
    pub items: Vec<RenderItem>,      // Items for recursive flattening
}
```

### Construction helpers

| Method | Purpose |
|---|---|
| `RenderResult::from_text("hello")` | Wrap a plain string as a single-line result. |
| `RenderResult::from_segments(segments)` | Wrap a `Vec<Segment>` as a single line. |
| `RenderResult::from_lines(lines)` | Wrap pre-computed `Vec<Vec<Segment>>` lines. |
| `RenderResult::from_items(items)` | Wrap nested `RenderItem`s without flattening. |
| `result.push_item(item)` | Append a `RenderItem` (segment or nested renderable). |
| `result.push_renderable(r)` | Append a nested renderable for recursive rendering. |

### Flattening

When `items` is populated, `Console::render()` flattens the tree
recursively: each `RenderItem::Segment` is emitted as-is, while each
`RenderItem::Nested` is itself rendered and then flattened again. This
enables composition without eagerly producing segments:

```rust
let mut result = RenderResult::new();
result.push_renderable(inner_panel);
result.push_item(Segment::line());
// Console::render() will recursively handle inner_panel
```

---

## `RenderItem`

Represents a single element in a render pipeline --- either a final
`Segment` or a nested renderable that will be recursively resolved:

```rust
pub enum RenderItem {
    Segment(Segment),
    Nested(DynRenderable),
}

// Conversions
impl From<Segment> for RenderItem { ... }
impl From<DynRenderable> for RenderItem { ... }
```

This enum mirrors Python Rich's `RenderResult` type, which is an iterable
of `Union[Segment, RenderableType]`. By deferring nested renderables,
rusty-rich avoids producing segments for composition-only wrappers
until the final pass.

---

## `DynRenderable`

A type-erased, cloneable wrapper around `dyn Renderable + Send + Sync`. It
is used wherever a renderable must be stored by value (e.g. `Panel`'s content,
`Group`'s children, `RenderItem::Nested`).

```rust
#[derive(Clone)]
pub struct DynRenderable {
    inner: Arc<dyn Renderable + Send + Sync>,
}

impl DynRenderable {
    pub fn new(r: impl Renderable + Send + Sync + 'static) -> Self {
        Self { inner: Arc::new(r) }
    }
}

impl Renderable for DynRenderable {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        self.inner.render(options)
    }
}
```

Because `DynRenderable` implements `Renderable` itself, it can be used
anywhere a renderable reference is expected --- pass a `DynRenderable` to
a `Panel` or `Group` and it will delegate transparently.

---

## `Group`

A simple compositor that renders a sequence of children one after another:

```rust
pub struct Group {
    pub children: Vec<DynRenderable>,
}

impl Group {
    pub fn new() -> Self;
    pub fn add(&mut self, renderable: impl Renderable + Send + Sync + 'static);
}
```

When rendered, each child is rendered independently and their line-lists
are concatenated:

```rust
let mut group = Group::new();
group.add(Rule::new().title("Section A"));
group.add(Panel::new("Content for A"));
group.add(Rule::new().title("Section B"));
group.add(Panel::new("Content for B"));

console.println(&group);
```

`Group` implements `Renderable`, so it can be passed to `Console::println()`,
placed inside a `Panel`, or nested in another `Group`.

---

## `Segment` --- The Atomic Output Unit

Every renderable ultimately decomposes into `Segment`s. A segment is a
piece of styled text --- the smallest unit the Console writes to the
terminal:

```rust
pub struct Segment {
    pub text: String,              // The text content
    pub style: Option<Style>,      // Optional styling
    pub control: Option<ControlCode>,  // Optional control sequence
}
```

### Creating segments

```rust
use rusty_rich::{Segment, Style, Color};

// Plain text
let plain = Segment::new("Hello");

// Styled text
let styled = Segment::styled("Error", Style::new().bold(true).color(Color::parse("red").unwrap()));

// Newline
let nl = Segment::line();

// Control sequence
let clear = Segment::control(ControlCode::Simple(ControlType::Clear));
```

### Key methods

| Method | Description |
|---|---|
| `seg.cell_length()` | Unicode-aware display width (0 for control segments). |
| `seg.split(offset)` | Split at a cell-width position, returning two segments. |
| `seg.to_ansi()` | Produce the full ANSI escape sequence + text + reset. |
| `seg.is_empty()` | True if there is neither text nor a control code. |

### `Segments` collection

A `Segments` wrapper provides convenience operations on `Vec<Segment>`:

```rust
let mut segs = Segments::new();
segs.push(Segment::styled("bold", Style::new().bold(true)));
segs.push(Segment::line());
println!("{}", segs.to_ansi());
```

---

## Control Codes

Control codes are non-printable terminal instructions embedded in the
segment stream. They let renderables move the cursor, clear the screen,
show/hide the cursor, and more.

### `ControlType`

The enum of recognised control operations:

```rust
pub enum ControlType {
    Bell,
    CarriageReturn,
    Home,
    Clear,
    ShowCursor,
    HideCursor,
    EnableAltScreen,
    DisableAltScreen,
    CursorUp,
    CursorDown,
    CursorForward,
    CursorBackward,
    CursorMoveToColumn,
    CursorMoveTo,
    EraseInLine,
    SetWindowTitle,
}
```

### `ControlCode`

Wraps a `ControlType` with zero or more parameters:

```rust
pub enum ControlCode {
    Simple(ControlType),                     // No parameters
    WithInt(ControlType, i32),               // Single integer parameter
    WithTwoInts(ControlType, i32, i32),      // Row + column
    WithString(ControlType, String),         // Window title, etc.
}
```

### Creating control segments

```rust
use rusty_rich::segment::{ControlCode, ControlType, Segment};

// Clear screen
let seg = Segment::control(ControlCode::Simple(ControlType::Clear));

// Move cursor to row 5, column 10
let seg = Segment::control(ControlCode::WithTwoInts(ControlType::CursorMoveTo, 5, 10));

// Set window title
let seg = Segment::control(ControlCode::WithString(ControlType::SetWindowTitle, "My App".into()));
```

The `to_ansi()` method on `ControlType` converts each variant to the
appropriate ANSI escape sequence:

| Control | ANSI |
|---|---|
| `Clear` | `\x1b[2J` |
| `ShowCursor` | `\x1b[?25h` |
| `HideCursor` | `\x1b[?25l` |
| `EnableAltScreen` | `\x1b[?1049h` |
| `CursorUp(n)` | `\x1b[nA` |
| `CursorMoveTo(r,c)` | `\x1b[r;cH` |
| `SetWindowTitle(s)` | `\x1b]0;s\x07` |

Control segments have zero cell length (they contribute nothing to text
width measurement) and are passed through verbatim during rendering.

---

## How Rusty-Rich Built-Ins Implement `Renderable`

### `&str` / `String`

The simplest implementation: a plain string renders as a single line
containing one text segment.

```rust
impl Renderable for &str {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        RenderResult::from_text(self)
    }
}
```

### `Text`

Styled text with `Span`s. Its `render()` method applies the default style
plus any per-region spans, producing ANSI-wrapped lines.

### `Rule`

Draws a horizontal divider. The implementation reads `options.max_width`
and uses the `characters` field repeated to fill the row, optionally
inserting a title in the middle. It respects `options.ascii_only` by
falling back to `"-"` for non-ASCII characters.

### `Panel`

A bordered container. Its `render()` method:

1. Wraps inner content via `DynRenderable`.
2. Calculates inner width from `options.max_width` minus border +
   padding.
3. Recursively renders the content with adjusted `ConsoleOptions`.
4. Produces top border (with optional title), padding rows, content
   rows, padding rows, and bottom border (with optional subtitle).

### `Padding`

Adds whitespace around content. Renders the child with a reduced
`max_width` after subtracting left/right padding, then prepends
blank rows and appends space padding.

### `Columns` / `Layout`

These compose multiple children by measuring them first, then rendering
each with a fraction of the available width. The `measure()` hook is
critical here --- without it the layout engine cannot partition space
fairly.

---

## Implementing a Custom Renderable from Scratch

Let's build a **meter bar** --- a fixed-width visual indicator that shows
a percentage fill, colour-coded by threshold.

```rust
use std::fmt;
use unicode_width::UnicodeWidthStr;
use rusty_rich::{
    Console, ConsoleOptions, RenderResult, Renderable, RenderItem,
    Segment, Style, Color, Measurement,
};

/// A colour-coded meter bar showing a percentage.
pub struct Meter {
    percent: f64,
    width: usize,
}

impl Meter {
    /// Create a new meter with the given percentage (0.0 -- 100.0).
    pub fn new(percent: f64) -> Self {
        Self {
            percent: percent.clamp(0.0, 100.0),
            width: 20,
        }
    }

    /// Set the bar width in characters (default 20).
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Choose a colour based on the value.
    fn fill_style(&self) -> Style {
        if self.percent >= 80.0 {
            Style::new().color(Color::parse("red").unwrap())
        } else if self.percent >= 50.0 {
            Style::new().color(Color::parse("yellow").unwrap())
        } else {
            Style::new().color(Color::parse("green").unwrap())
        }
    }
}

impl Renderable for Meter {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        // Clamp bar width to the available terminal width
        let bar_width = self.width.min(options.max_width);

        // Calculate how many filled cells
        let filled = ((self.percent / 100.0) * bar_width as f64).round() as usize;
        let empty = bar_width.saturating_sub(filled);

        let fill_style = self.fill_style();
        let label = format!(" {:3.0}% ", self.percent);
        let label_w = UnicodeWidthStr::width(label.as_str());

        // Build segments
        let mut segments = Vec::new();

        // Opening bracket (always visible)
        segments.push(Segment::styled("[", Style::new()));

        // Filled portion
        if filled > 0 {
            let fill_text = "█".repeat(filled);
            segments.push(Segment::styled(fill_text, fill_style.clone()));
        }

        // Empty portion
        if empty > 0 {
            segments.push(Segment::new("░".repeat(empty)));
        }

        // Closing bracket
        segments.push(Segment::styled("]", Style::new()));

        // Percentage label
        segments.push(Segment::styled(
            label,
            Style::new().bold(true),
        ));

        // Newline
        segments.push(Segment::line());

        // Build RenderResult as a single line of segments
        RenderResult {
            lines: vec![segments],
            items: Vec::new(),
        }
    }

    fn measure(&self, _options: &ConsoleOptions) -> Option<Measurement> {
        // Minimum: bracket + 1 char + bracket + label = 2 + 1 + 2 + 6 ≈ 11
        // Maximum: our configured width + label + brackets ≈ width + 9
        let min = 11.min(self.width + 9);
        let max = self.width + 9;
        Some(Measurement::new(min, max))
    }
}
```

Now use it:

```rust
let mut console = Console::new();

let bar = Meter::new(73.5);
console.println(&bar);

// Composability: put the meter inside a panel
let panel = Panel::new(bar).title("Disk Usage");
console.println(&panel);
```

This produces terminal output similar to:

```
[██████████████░░░░░░]  74%
┌──────────────────────────────┐
│ [██████████████░░░░░░]  74%  │
│                              │
└──────────────────────────────┘
```

### Key points from this example

1. **Constraint respect**: The `render()` method clamps `bar_width` to
   `options.max_width`, ensuring no output exceeds the terminal.

2. **`measure()` hook**: Both minimum and maximum are returned. The minimum
   ensures the meter is never squeezed below displayable width; the maximum
   lets parent layout engines know its natural size.

3. **Segment construction**: Each visual element becomes a `Segment` with
   its own style. The segment stream is flat --- no `RenderItem::Nested`
   needed.

4. **Composability**: Because `Meter` implements `Renderable`, it can be
   passed to `Console::println()`, placed inside a `Panel`, added to a
   `Group`, or inserted into a `Table` cell --- no special glue required.

---

## Summary

| Concept | Role |
|---|---|
| `Renderable` trait | The single interface everything displayable must implement. |
| `render()` | Produce styled segments respecting `ConsoleOptions` constraints. |
| `measure()` | Report min/max width for layout negotiation (optional). |
| `RenderResult` | Output container: either flat segment-lines or nested items. |
| `RenderItem` | Either a final `Segment` or a recursively-rendered child. |
| `DynRenderable` | Type-erased, cloneable, `Send + Sync` trait object wrapper. |
| `Group` | Render multiple renderables sequentially. |
| `Segment` | The atomic output unit: text + style + optional control code. |
| `ControlCode` | Non-printable terminal instructions (cursor moves, clear, etc.). |
