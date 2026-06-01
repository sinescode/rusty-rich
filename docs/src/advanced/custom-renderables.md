# Custom Renderables

This guide builds on the [Renderable Protocol](../core-concepts/renderable-protocol.md) and focuses on practical patterns for implementing your own display components. You will learn how to compose children, use recursive rendering, apply style inheritance, emit control codes, and build a complete dashboard widget from scratch.

---

## Quick Recap

Every renderable implements the `Renderable` trait:

```rust
pub trait Renderable {
    fn render(&self, options: &ConsoleOptions) -> RenderResult;
    fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> { None }
}
```

- **`render()`** is required. Produce `Segment`s that represent your output.
- **`measure()`** is optional. Return min/max width hints for layout engines.

The output container `RenderResult` can hold either pre-computed lines of segments or a list of `RenderItem`s that will be recursively flattened by `Console::render()`.

---

## Patterns for Returning RenderResult

### 1. Flat segments (simplest)

Build a single line of `Segment`s and return them as a flat list:

```rust
fn render(&self, _options: &ConsoleOptions) -> RenderResult {
    let mut segments = vec![
        Segment::styled(" [ ", Style::new().dim(true)),
        Segment::styled("OK", Style::new().bold(true).color(Color::Green)),
        Segment::styled(" ] ", Style::new().dim(true)),
    ];
    segments.push(Segment::line());
    RenderResult::from_segments(segments)
}
```

### 2. Multi-line flat output

Build lines manually and return with `from_lines`:

```rust
fn render(&self, _options: &ConsoleOptions) -> RenderResult {
    let lines = vec![
        vec![Segment::styled("Header", Style::new().bold(true)), Segment::line()],
        vec![Segment::new("  content line 1"), Segment::line()],
        vec![Segment::new("  content line 2"), Segment::line()],
    ];
    RenderResult::from_lines(lines)
}
```

### 3. Deferred (nested) rendering

Push child renderables as `RenderItem::Nested` and let the console recursively flatten them. This is how container widgets like `Panel` and `Padding` avoid producing segments for their children up front:

```rust
fn render(&self, options: &ConsoleOptions) -> RenderResult {
    let mut result = RenderResult::new();
    // Emit a header segment
    result.push_item(Segment::styled("Widget:\n", Style::new().bold(true)));
    // Defer rendering of the inner content
    result.push_renderable(self.inner.clone());
    // Add a trailing separator
    result.push_item(Segment::styled("\n---", Style::new().dim(true)));
    result
}
```

When you use `push_renderable`, the wrapped child must be `Renderable + Send + Sync + 'static`. It gets converted to a `DynRenderable` internally.

### 4. Mixed approach

Combine flat segments with nested children for maximum flexibility:

```rust
fn render(&self, options: &ConsoleOptions) -> RenderResult {
    let mut result = RenderResult::new();
    // Header
    result.push_item(Segment::styled(format!("{}\n", self.title), Style::new().bold(true)));
    // Each child
    for child in &self.children {
        result.push_renderable(child.clone());
        result.push_item(Segment::line());
    }
    // Footer
    result.push_item(Segment::styled(format!("{} items total\n", self.children.len()), Style::new().dim(true)));
    result
}
```

---

## The `measure()` Hook in Practice

The `measure()` method lets layout engines (columns, tables, panels) know how much horizontal space your renderable needs **without** rendering it first. This is critical for responsive layouts.

### When to implement

- Your renderable has a known or calculable natural width.
- You want fair space allocation inside `Columns` or `Table`.
- You want to avoid the cost of a full render just for width negotiation.

### Examples from built-ins

**Rule** -- width depends on title length:

```rust
fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
    let title_w = UnicodeWidthStr::width(self.title.as_str());
    let min = if title_w > 0 { title_w + 4 } else { 1 };
    Some(Measurement::new(min, options.max_width))
}
```

**Panel** -- delegates to the inner renderable's measure and adds border + padding:

```rust
fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
    self.renderable.measure(options).map(|m| {
        let border = 2;
        let padding_h = self.padding.1 + self.padding.3;
        m.grow(border + padding_h)
    })
}
```

### Common pitfalls

- Returning `None` (the default) forces the layout engine to fall back to a full render just for measurement -- expensive for complex widgets.
- Reporting a `minimum` of 0 can cause layout collapse. Always ensure at least `1`.
- The `maximum` should never exceed `options.max_width` unless your renderable truly overflows.

---

## Working with Segments

### Creating styled segments

```rust
use rusty_rich::{Segment, Style, Color};

// Plain text
Segment::new("Hello");

// With style
Segment::styled("Warning", Style::new().bold(true).color(Color::parse("yellow").unwrap()));

// Compound style (chaining)
let style = Style::new()
    .bold(true)
    .italic(true)
    .color(Color::Rgb(255, 128, 0));
Segment::styled("Orange bold italic", style);
```

### Controlling newlines

Segments are **not** automatically wrapped. You must insert newlines explicitly:

```rust
// Correct -- separate lines
let line1 = vec![Segment::new("Line 1"), Segment::line()];
let line2 = vec![Segment::new("Line 2"), Segment::line()];
RenderResult::from_lines(vec![line1, line2]);

// Also correct -- newline inside a segment
let mut segments = vec![
    Segment::styled("Header\n", Style::new().bold(true)),
    Segment::new("Content\n"),
];
```

### Space padding for alignment

When your output must fill a fixed width, pad with spaces:

```rust
let content = "Name: Alice";
let padded = format!("{:<width$}", content, width = options.max_width);
segments.push(Segment::new(padded));
```

Or use multiple segments with fill:

```rust
let label = Segment::styled("Status:", Style::new().bold(true));
let value = Segment::styled("Running", Style::new().color(Color::Green));
let fill = options.max_width.saturating_sub(label.cell_length() + value.cell_length());
segments.push(label);
segments.push(Segment::new(" ".repeat(fill)));
segments.push(value);
```

### Splitting segments for wrapping

Use `Segment::split(offset)` to break a segment at a cell-width position. This is useful when implementing word-wrap inside a custom renderable:

```rust
fn wrap_segment(seg: &Segment, max_width: usize) -> Vec<Segment> {
    let text = &seg.text;
    let mut result = Vec::new();
    let mut remaining = text.as_str();
    while !remaining.is_empty() {
        let w = unicode_width::UnicodeWidthStr::width(remaining);
        if w <= max_width {
            result.push(Segment::styled(remaining, seg.style.clone().unwrap_or_default()));
            break;
        }
        // Find break point
        let mut n = max_width;
        while n > 0 && !remaining.is_char_boundary(n) {
            n -= 1;
        }
        result.push(Segment::styled(&remaining[..n], seg.style.clone().unwrap_or_default()));
        result.push(Segment::line());
        remaining = &remaining[n..];
    }
    result
}
```

---

## Emitting Control Codes

Control codes let you manipulate the terminal directly. They are embedded as special segments with zero display width.

### Hiding and showing the cursor

Useful for full-screen TUI widgets:

```rust
use rusty_rich::segment::{ControlCode, ControlType, Segment};

fn render(&self, _options: &ConsoleOptions) -> RenderResult {
    let mut items = Vec::new();
    // Hide cursor before drawing
    items.push(RenderItem::Segment(Segment::control(ControlCode::Simple(ControlType::HideCursor))));
    // Your content
    items.push(RenderItem::Segment(Segment::styled("Full screen content\n", Style::new())));
    // Show cursor when done
    items.push(RenderItem::Segment(Segment::control(ControlCode::Simple(ControlType::ShowCursor))));
    RenderResult::from_items(items)
}
```

### Clearing the screen

```rust
Segment::control(ControlCode::Simple(ControlType::Clear))
```

### Moving the cursor

```rust
// Move to column 10 on current row
Segment::control(ControlCode::WithInt(ControlType::CursorMoveToColumn, 10));

// Move to row 5, column 20
Segment::control(ControlCode::WithTwoInts(ControlType::CursorMoveTo, 5, 20));

// Move up 3 lines
Segment::control(ControlCode::WithInt(ControlType::CursorUp, 3));
```

### Alternative screen buffer

```rust
Segment::control(ControlCode::Simple(ControlType::EnableAltScreen));
// ... render your content ...
Segment::control(ControlCode::Simple(ControlType::DisableAltScreen));
```

### Setting the window title

```rust
Segment::control(ControlCode::WithString(ControlType::SetWindowTitle, "My Dashboard".into()));
```

### When to use control codes

- **Full-screen live displays** (cursor manipulation, alt screen).
- **Progress-like widgets** (carriage return to overwrite a line).
- **Terminal title updates** (window title).

For simple renderables that just print once, you generally do not need control codes -- plain segments with text and style are sufficient.

---

## Recursive Rendering -- How Container Widgets Work

Container widgets like `Panel`, `Padding`, and `Layout` follow a common pattern:

1. **Adjust `ConsoleOptions`** for the child (subtract border, padding, etc.).
2. **Call `child.render(&adjusted_options)`** to get the child's `RenderResult`.
3. **Wrap the child's output** with the container's own segments (borders, padding, labels).

This is the key pattern to understand when building your own containers:

```rust
impl Renderable for BorderLabel {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        // 1. Compute inner width (subtract border width)
        let inner_width = options.max_width.saturating_sub(2);
        let inner_opts = options.update_width(inner_width);

        // 2. Recursively render the child
        let child_result = self.inner.render(&inner_opts);

        // 3. Build output: border lines around child lines
        let mut lines = Vec::new();
        lines.push(vec![
            Segment::styled(format!("[{}]", self.label), Style::new().bold(true)),
            Segment::line(),
        ]);
        for child_line in &child_result.lines {
            let mut line = Vec::new();
            line.push(Segment::styled("| ", Style::new().dim(true)));
            line.extend(child_line.iter().cloned());
            line.push(Segment::styled(" |", Style::new().dim(true)));
            line.push(Segment::line());
            lines.push(line);
        }
        lines.push(vec![
            Segment::styled(format!("[{}]", "\u{2014}".repeat(self.label.chars().count())), Style::new().dim(true)),
            Segment::line(),
        ]);

        RenderResult { lines, items: Vec::new() }
    }
}
```

### Pass-through of constraints

Always pass `options.max_width` through to children. If you shrink it, use `options.update_width()` or `options.shrink_width()` so the child knows the available space. If you do not propagate the width, child renderables will use the default `ConsoleOptions` (usually 80 columns) and ignore the actual terminal width.

---

## Style Inheritance in Custom Renderables

rusty-rich does not have automatic cascading style sheets, but you can implement style inheritance patterns yourself.

### Pattern 1: Per-style field

Store a base `Style` on your struct and apply it to all child segments:

```rust
pub struct StyledBlock {
    content: String,
    style: Style,
    title_style: Style,
}

impl Renderable for StyledBlock {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        let mut segments = vec![
            Segment::styled(format!("{}\n", self.content), self.style.clone()),
        ];
        if !self.title.is_empty() {
            segments.insert(0, Segment::styled(
                format!("{}\n", self.title),
                self.title_style.clone(),
            ));
        }
        RenderResult::from_segments(segments)
    }
}
```

### Pattern 2: Merging inherited style with local overrides

Accept an optional "parent" style and merge it:

```rust
impl Renderable for MyWidget {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let base_style = options.theme.as_ref()
            .and_then(|t| t.get("mywidget"))
            .cloned()
            .unwrap_or_default();

        // Local style overrides the inherited one
        let final_style = base_style.merge(&self.local_style);
        // ... use final_style for segments
    }
}
```

### Pattern 3: Theme-aware renderables

Consult the `ConsoleOptions`'s associated theme for style names. (Currently the Console passes options with a theme; check your version's API.)

---

## DynRenderable -- Type Erasure for Storage

Whenever you need to store a renderable in a struct field (a `Vec`, a `Box`, a struct field), use `DynRenderable`:

```rust
use rusty_rich::DynRenderable;

pub struct Card {
    title: String,
    body: DynRenderable,  // any Renderable + Send + Sync + 'static
}

impl Card {
    pub fn new(title: impl Into<String>, body: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            title: title.into(),
            body: DynRenderable::new(body),
        }
    }
}

impl Renderable for Card {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut result = RenderResult::new();
        result.push_item(Segment::styled(
            format!("{}:\n", self.title),
            Style::new().bold(true),
        ));
        result.push_renderable(self.body.clone());
        result
    }
}
```

`DynRenderable` implements `Clone` (via `Arc`), `Send`, and `Sync`, making it safe to share across threads. It delegates `render()` and `measure()` to the inner value transparently.

### When to use DynRenderable

| Situation | Use |
|---|---|
| Storing one child (like `Panel`) | `DynRenderable` |
| Storing many children (like `Group`) | `Vec<DynRenderable>` |
| Cloning a renderable | `DynRenderable` (cheap Arc clone) |
| Sending across threads | `DynRenderable` is `Send + Sync` |
| Returning from a function by value | The concrete type, if known |
| Generic API (user passes any renderable) | Accept `impl Renderable + Send + Sync + 'static`, store as `DynRenderable` |

---

## Group -- Sequential Composition

`Group` renders a list of children one after another. It is useful for composing renderables without writing a container:

```rust
use rusty_rich::Group;

let mut group = Group::new();
group.add(Rule::new().title("Section 1"));
group.add(Panel::new("Content").title("Details"));
group.add(Rule::new().title("Section 2"));
group.add(Text::from("More content"));

console.println(&group);
```

### Implementing your own Group-like combinator

```rust
pub struct VStack {
    children: Vec<DynRenderable>,
    separator: Option<Segment>,
}

impl VStack {
    pub fn new() -> Self { Self { children: Vec::new(), separator: None } }

    pub fn add(&mut self, r: impl Renderable + Send + Sync + 'static) {
        self.children.push(DynRenderable::new(r));
    }

    pub fn with_separator(mut self, seg: Segment) -> Self {
        self.separator = Some(seg);
        self
    }
}

impl Renderable for VStack {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut lines = Vec::new();
        for (i, child) in self.children.iter().enumerate() {
            if i > 0 {
                if let Some(ref sep) = self.separator {
                    lines.push(vec![sep.clone()]);
                }
            }
            let result = child.render(options);
            lines.extend(result.lines);
        }
        RenderResult { lines, items: Vec::new() }
    }
}
```

---

## Best Practices

### 1. Always respect `options.max_width` and `options.max_height`

The terminal may be resized at any time. Never hardcode widths. Clamp your output to the provided constraints:

```rust
let usable = options.max_width.min(self.desired_width);
```

### 2. Provide a meaningful `measure()` implementation

If your renderable has a known size, implement `measure()` to return it. This helps parent layout engines partition screen space. Without it, the layout engine must fully render your content just to measure it, which is wasteful for complex widgets.

### 3. Prefer `RenderItem::Nested` for composition wrappers

If your renderable just wraps another renderable and adds some decorations, use `push_renderable()` to defer the child's rendering. This delays segment generation until the final pass and allows the console to optimise the full output tree.

### 4. Do not mix `lines` and `items` unless you need both

`RenderResult` can hold both, but the console processes `items` first. If you populate `items`, the `lines` field is only used as a fallback. Pick one style and stick with it.

### 5. End each line with a newline segment

Terminal output is line-oriented. Every visual line in your `lines` vector should end with a `Segment::line()` unless you are building a single-line inline element:

```rust
// Good
vec![Segment::new("Content"), Segment::line()]

// Bad -- no newline; everything glues together
vec![Segment::new("Content")]
```

### 6. Use `unicode-width` for text measurement

Rust's `len()` counts bytes, not visible characters. Always use `unicode_width::UnicodeWidthStr::width()` for layout calculations involving text. CJK characters, emoji, and combining marks all have different display widths.

### 7. Implement `Debug` for your renderables

`Debug` is required by many of rusty-rich's own types. Provide a minimal implementation that identifies the type and key fields without printing the full content:

```rust
impl std::fmt::Debug for MyWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyWidget")
            .field("title", &self.title)
            .field("width", &self.width)
            .finish()
    }
}
```

### 8. Test with different widths

Write tests that render your custom renderable at various `ConsoleOptions` widths to verify it adapts correctly:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusty_rich::{ConsoleOptions, ConsoleDimensions};

    #[test]
    fn test_at_narrow_width() {
        let widget = MyWidget::new("Hello, World!");
        let opts = ConsoleOptions {
            max_width: 10,
            ..ConsoleOptions::default()
        };
        let result = widget.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello"));
        // Verify no line exceeds 10 cells
        for line in &result.lines {
            let w: usize = line.iter().map(|s| s.cell_length()).sum();
            assert!(w <= 10, "line width {} exceeds 10", w);
        }
    }
}
```

---

## Building a Dashboard Renderable from Scratch

Let's build a complete `Dashboard` renderable that displays system metrics in a bordered grid layout. It combines multiple sub-renderables, uses recursive rendering, respects terminal width, and provides a `measure()` implementation.

```rust
use std::fmt;
use unicode_width::UnicodeWidthStr;

use rusty_rich::{
    Console, ConsoleOptions, DynRenderable, Measurement, RenderItem,
    RenderResult, Renderable, Segment, Style, Color,
};

// ---------------------------------------------------------------------------
// MetricGauge -- a single metric display
// ---------------------------------------------------------------------------

/// A single metric showing a label, a value, and a colour-coded bar.
#[derive(Clone)]
pub struct MetricGauge {
    label: String,
    value: f64,
    unit: String,
    /// 0.0 -- 100.0
    percent: f64,
    bar_width: usize,
    // Styles
    label_style: Style,
    value_style: Style,
    bar_fill_style: Style,
    bar_empty_style: Style,
}

impl MetricGauge {
    pub fn new(label: impl Into<String>, value: f64, unit: impl Into<String>, percent: f64) -> Self {
        let p = percent.clamp(0.0, 100.0);
        let fill_color = if p >= 80.0 {
            Color::parse("red").unwrap()
        } else if p >= 50.0 {
            Color::parse("yellow").unwrap()
        } else {
            Color::parse("green").unwrap()
        };

        Self {
            label: label.into(),
            value,
            unit: unit.into(),
            percent: p,
            bar_width: 15,
            label_style: Style::new().bold(true),
            value_style: Style::new().bold(true),
            bar_fill_style: Style::new().color(fill_color),
            bar_empty_style: Style::new().dim(true),
        }
    }

    /// Builder: custom bar width.
    pub fn bar_width(mut self, w: usize) -> Self {
        self.bar_width = w;
        self
    }
}

impl Renderable for MetricGauge {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        let mut segments = Vec::new();

        // Label
        segments.push(Segment::styled(
            format!("{} ", self.label),
            self.label_style.clone(),
        ));

        // Value
        let value_str = format!("{:.1}{}", self.value, self.unit);
        segments.push(Segment::styled(value_str, self.value_style.clone()));

        segments.push(Segment::new("\n"));

        // Bar: [fill][empty]
        let filled = ((self.percent / 100.0) * self.bar_width as f64).round() as usize;
        let empty = self.bar_width.saturating_sub(filled);

        segments.push(Segment::new("  "));
        segments.push(Segment::styled(
            "\u{2588}".repeat(filled),
            self.bar_fill_style.clone(),
        ));
        segments.push(Segment::styled(
            "\u{2591}".repeat(empty),
            self.bar_empty_style.clone(),
        ));

        // Percentage label
        segments.push(Segment::styled(
            format!(" {:3.0}%", self.percent),
            Style::new().dim(true),
        ));

        segments.push(Segment::line());

        RenderResult::from_segments(segments)
    }

    fn measure(&self, _options: &ConsoleOptions) -> Option<Measurement> {
        let label_w = UnicodeWidthStr::width(self.label.as_str());
        let value_w = UnicodeWidthStr::width(
            format!("{:.1}{}", self.value, self.unit).as_str(),
        );
        // Bar: prefix ("  ") + fill + empty + suffix (" 100%") ≈ bar_width + 9
        let total = label_w + 1 + value_w;
        let max = total.max(self.bar_width + 9);
        let min = 10; // label + " 0%"
        Some(Measurement::new(min, max))
    }
}

impl fmt::Debug for MetricGauge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetricGauge")
            .field("label", &self.label)
            .field("value", &self.value)
            .field("percent", &self.percent)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// DashboardPanel -- a bordered section inside the dashboard
// ---------------------------------------------------------------------------

/// A labelled box that holds a single gauge.
#[derive(Clone)]
pub struct DashboardPanel {
    title: String,
    gauge: DynRenderable,
}

impl DashboardPanel {
    pub fn new(title: impl Into<String>, gauge: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            title: title.into(),
            gauge: DynRenderable::new(gauge),
        }
    }
}

impl Renderable for DashboardPanel {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let inner_width = options.max_width.saturating_sub(4); // left/right borders + padding
        let inner_opts = options.update_width(inner_width.max(1));
        let content = self.gauge.render(&inner_opts);

        let title_w = UnicodeWidthStr::width(self.title.as_str());
        let inner_actual = content
            .lines
            .iter()
            .map(|l| l.iter().map(|s| s.cell_length()).sum::<usize>())
            .max()
            .unwrap_or(0);
        let panel_w = (inner_actual + 4).min(options.max_width);

        let mut lines = Vec::new();

        // Top border with title
        let dash = "\u{2500}";
        let inner = panel_w.saturating_sub(4);
        let title_fill = if title_w + 2 <= inner {
            let rem = inner - title_w - 2;
            let left = rem / 2;
            let right = rem - left;
            format!(
                "{}{} {} {}{}",
                "\u{250C}",
                dash.repeat(left),
                self.title,
                dash.repeat(right),
                "\u{2510}",
            )
        } else {
            format!("\u{250C}{}\u{2510}", dash.repeat(inner))
        };
        lines.push(vec![Segment::new(title_fill), Segment::line()]);

        // Content lines with side borders
        for content_line in &content.lines {
            let mut line = Vec::new();
            line.push(Segment::new(format!("\u{2502} ")));
            line.extend(content_line.iter().cloned());
            let content_w: usize = content_line.iter().map(|s| s.cell_length()).sum();
            let fill = inner_actual.saturating_sub(content_w);
            if fill > 0 {
                line.push(Segment::new(" ".repeat(fill)));
            }
            line.push(Segment::new(" \u{2502}"));
            line.push(Segment::line());
            lines.push(line);
        }

        // Bottom border
        let bottom = format!("\u{2514}{}\u{2518}", dash.repeat(inner));
        lines.push(vec![Segment::new(bottom), Segment::line()]);

        RenderResult { lines, items: Vec::new() }
    }

    fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
        self.gauge.measure(options).map(|m| {
            let border_pad = 4;
            m.grow(border_pad)
        })
    }
}

impl fmt::Debug for DashboardPanel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DashboardPanel")
            .field("title", &self.title)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Dashboard -- full dashboard combining multiple panels
// ---------------------------------------------------------------------------

/// A full-screen dashboard that arranges MetricGauges in a responsive grid.
#[derive(Clone)]
pub struct Dashboard {
    title: String,
    panels: Vec<DynRenderable>,
    columns: usize,
}

impl Dashboard {
    /// Create a new dashboard with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            panels: Vec::new(),
            columns: 2,
        }
    }

    /// Add a panel.
    pub fn add(&mut self, panel: impl Renderable + Send + Sync + 'static) {
        self.panels.push(DynRenderable::new(panel));
    }

    /// Set the number of columns (default 2).
    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }
}

impl Renderable for Dashboard {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut result = RenderResult::new();

        // Title bar
        let title_text = format!(" {} ", self.title);
        let title_w = UnicodeWidthStr::width(title_text.as_str());
        let title_line = if title_w + 2 <= options.max_width {
            let rem = options.max_width.saturating_sub(title_w + 2);
            let left = rem / 2;
            let right = rem - left;
            format!(
                "{}{}{}{}",
                "\u{2554}",
                "\u{2550}".repeat(left),
                title_text,
                "\u{2550}".repeat(right),
            )
        } else {
            format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(options.max_width.saturating_sub(2)))
        };
        result.push_item(Segment::styled(title_line, Style::new().bold(true)));
        result.push_item(Segment::line());

        // Measure how wide each panel wants to be
        let col_width = options.max_width / self.columns;
        let mut row_panels: Vec<Vec<&DynRenderable>> = Vec::new();
        let mut current_row = Vec::new();

        for panel in &self.panels {
            current_row.push(panel);
            if current_row.len() >= self.columns {
                row_panels.push(std::mem::take(&mut current_row));
            }
        }
        if !current_row.is_empty() {
            row_panels.push(current_row);
        }

        // Render each row
        for row in &row_panels {
            let col_opts = options.update_width(col_width.saturating_sub(1));
            // Render each panel in the row and collect their line-vectors
            let mut rendered_panels: Vec<RenderResult> = row
                .iter()
                .map(|p| p.render(&col_opts))
                .collect();

            // Determine max line count across panels in this row
            let max_lines = rendered_panels
                .iter()
                .map(|r| r.lines.len())
                .max()
                .unwrap_or(0);

            // Emit lines side-by-side
            for line_idx in 0..max_lines {
                for (p_idx, r) in rendered_panels.iter().enumerate() {
                    if p_idx > 0 {
                        result.push_item(Segment::styled(" \u{2502} ", Style::new().dim(true)));
                    }
                    if let Some(segments) = r.lines.get(line_idx) {
                        for seg in segments {
                            result.push_item(seg.clone());
                        }
                        // Pad to column width
                        let w: usize = segments.iter().map(|s| s.cell_length()).sum();
                        let pad = col_width.saturating_sub(w);
                        if pad > 0 {
                            result.push_item(Segment::new(" ".repeat(pad)));
                        }
                    } else {
                        // Empty line -- fill with spaces
                        result.push_item(Segment::new(" ".repeat(col_width)));
                    }
                }
                result.push_item(Segment::line());
            }
        }

        // Bottom border
        let bottom = format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(options.max_width.saturating_sub(2)));
        result.push_item(Segment::styled(bottom, Style::new().bold(true)));
        result.push_item(Segment::line());

        result
    }

    fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
        let col_width = options.max_width / self.columns;
        let col_opts = options.update_width(col_width);
        let mut min = 0usize;
        let mut max = 0usize;
        for panel in &self.panels {
            if let Some(m) = panel.measure(&col_opts) {
                min = min.max(m.minimum);
                max = max.max(m.maximum);
            }
        }
        // Account for separators between columns
        let separators = self.columns.saturating_sub(1) * 3; // " | "
        Some(Measurement::new(min, (max * self.columns + separators).min(options.max_width)))
    }
}

impl fmt::Debug for Dashboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dashboard")
            .field("title", &self.title)
            .field("panels", &self.panels.len())
            .field("columns", &self.columns)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Usage example
// ---------------------------------------------------------------------------

fn main() -> rusty_rich::Result<()> {
    let mut console = Console::new();

    // Create metric gauges
    let cpu = MetricGauge::new("CPU", 3.2, "%", 32.0);
    let mem = MetricGauge::new("Memory", 6.8, "GB", 68.0);
    let disk = MetricGauge::new("Disk", 245.0, "GB", 73.0);
    let net = MetricGauge::new("Network", 1.4, "Mbps", 14.0);

    // Wrap in dashboard panels
    let mut dashboard = Dashboard::new("System Monitor").columns(2);
    dashboard.add(DashboardPanel::new("Processor", cpu));
    dashboard.add(DashboardPanel::new("Memory", mem));
    dashboard.add(DashboardPanel::new("Storage", disk));
    dashboard.add(DashboardPanel::new("Network I/O", net));

    console.println(&dashboard);

    Ok(())
}
```

### What this example demonstrates

| Concept | How it appears |
|---|---|
| **`Renderable` trait** | `MetricGauge`, `DashboardPanel`, and `Dashboard` all implement `Renderable`. |
| **`RenderResult`** | Flat segments for `MetricGauge`; lines for `DashboardPanel`; deferred items for `Dashboard`. |
| **`measure()`** | Each component reports its min/max width, enabling responsive column sizing. |
| **`DynRenderable`** | Used for `DashboardPanel.gauge`, `Dashboard.panels` storage. |
| **Recursive rendering** | `DashboardPanel` renders its child gauge with adjusted `ConsoleOptions`; `Dashboard` renders panels and arranges their output side by side. |
| **Segment construction** | Block-drawing characters for bars, Unicode box-drawing for borders, styled text for labels. |
| **Constraint respect** | Every level clamps to `options.max_width`. |
| **Composability** | `MetricGauge` can be used standalone, inside `DashboardPanel`, or inside `Panel` -- it implements `Renderable` uniformly. |

---

## Summary

| Technique | When to use |
|---|---|
| **Flat segments** | Simple self-contained renderables with no children. |
| **Segments + newlines** | Multi-line output that you build manually. |
| **`push_renderable()`** | Container widgets that wrap children. |
| **`measure()`** | Whenever your renderable has a known or calculable width. |
| **`DynRenderable`** | Storing renderables in struct fields, `Vec`s, or cloning. |
| **`Group`** | Sequential composition without writing a container. |
| **Control codes** | Full-screen TUIs, cursor manipulation, progress displays. |
| **Recursive rendering** | Container widgets that adjust options and wrap child output. |
| **Style inheritance** | Accept a base style and merge with local overrides. |
