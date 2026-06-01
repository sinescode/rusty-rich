# Text and Markup

The `Text` type is rusty-rich's structured representation of styled text. It
pairs a plain string with a collection of `Span` values that mark byte ranges
with specific `Style` attributes. Console markup, on the other hand, is a
BBCode-inspired mini-language for expressing those same styles in a human-
readable string format.

Together they form the foundation for every renderable in rusty-rich: tables,
panels, trees, rules, and logs all use `Text` internally for their content.

---

## Text

`Text` is a renderable piece of text with optional style spans.

```rust
pub struct Text {
    pub plain: String,
    pub spans: Vec<Span>,
    pub style: Style,
    pub justify: JustifyMethod,
    pub end: String,
    pub overflow: OverflowMethod,
    pub no_wrap: bool,
}
```

The `plain` field holds the raw character data. The `spans` vector describes
which byte ranges should be rendered with which style. The top-level `style`
field is the default style for the whole piece of text -- individual spans
override it.

### Constructors

#### Text::new() -- Plain text

The simplest constructor. Creates a `Text` with no spans, left-justified, fold-
mode overflow, and a trailing newline.

```rust
use rusty_rich::Text;

let t = Text::new("Hello, world!");
assert_eq!(t.plain, "Hello, world!");
assert!(t.spans.is_empty());
assert_eq!(t.justify, text::JustifyMethod::Left);
assert_eq!(t.end, "\n");
```

Both `&str` and `String` can be used thanks to `From` impls:

```rust
let t1 = Text::from("from a &str");
let t2 = Text::from(String::from("from a String"));
```

#### Text::styled() -- Text with a default style

Creates a `Text` whose entire content carries a given default `Style`. No
individual spans are created -- the style is stored in the top-level `style`
field and applied uniformly.

```rust
use rusty_rich::{Text, Style, Color};

let t = Text::styled(
    "Important message",
    Style::new()
        .color(Color::parse("red").unwrap())
        .bold(true),
);
```

#### Content from console markup

To build a `Text` from a markup string, use `markup::render()`:

```rust
use rusty_rich::markup;

let t = markup::render("[bold green]Success![/bold green]");
assert_eq!(t.plain, "Success!");
```

This is the closest analogue to Python Rich's `Text.from_markup()`. There is
no inherent method on `Text` itself -- the free function is the canonical API.

### Builder methods

Every mutating method on `Text` also has a builder-style counterpart that
consumes and returns `self`, enabling method chaining.

#### style()

Overrides the default style.

```rust
use rusty_rich::{Text, Style, Color};

let t = Text::new("greetings")
    .style(Style::new().color(Color::parse("green").unwrap()).italic(true));
```

#### justify()

Sets text justification. Accepts any `AlignMethod` variant.

```rust
use rusty_rich::{Text, AlignMethod};

let t = Text::new("centered").justify(AlignMethod::Center);
let t = Text::new("right-aligned").justify(AlignMethod::Right);
```

| JustifyMethod    | Behavior                                      |
|------------------|-----------------------------------------------|
| `Left`           | Left-aligned (default).                       |
| `Right`          | Right-aligned.                                |
| `Center`         | Centred within the available width.           |
| `Full`           | Justified to full width (not yet implemented).|

#### end()

Sets the string appended after the text during rendering.

```rust
use rusty_rich::Text;

// No trailing newline
let t = Text::new("inline").end("");

// Custom separator
let t = Text::new("item").end(" --- ");
```

#### overflow()

Controls what happens when the text exceeds the available width.

```rust
use rusty_rich::{Text, OverflowMethod};

let t = Text::new("A very long string that may exceed the available width")
    .overflow(OverflowMethod::Ellipsis);
```

See the [OverflowMethod section](#overflow-method) for details on each variant.

#### no_wrap (field)

Disabling wrapping is done by setting the `no_wrap` field directly:

```rust
use rusty_rich::Text;

let mut t = Text::new("This line will not wrap");
t.no_wrap = true;
```

### Appending content

#### append() -- Append a Text or string

`append` takes anything that implements `Into<Text>` and splices its content and
spans into `self`. An optional style can be applied to the appended block.

```rust
use rusty_rich::{Text, Style};

let mut t = Text::new("Hello");
t.append(" World", None);
assert_eq!(t.plain, "Hello World");

// Append with a style that spans the appended region
t.append("!", Some(Style::new().bold(true)));
```

When a `&str` or `String` is passed, it is first converted to a plain `Text`
(losing any previous styling). When an existing `Text` is passed, its spans
are re-indexed to account for the new position.

#### append_styled() -- Append a plain string with a style

A convenience that takes a plain string and a single `Style`, pushing one new
`Span` for the appended region.

```rust
use rusty_rich::{Text, Style, Color};

let mut t = Text::new("The status is ");
t.append_styled(
    "OK",
    Style::new()
        .color(Color::parse("green").unwrap())
        .bold(true),
);

assert_eq!(t.plain, "The status is OK");
assert_eq!(t.spans.len(), 1);
assert_eq!(t.spans[0].start, 14);  // byte offset of "OK"
assert_eq!(t.spans[0].end,   16);
```

#### stylize() -- Apply a style to a subrange

Equivalent to Python Rich's `Text.stylize()`. Adds a span over the given byte
range.

```rust
use rusty_rich::{Text, Style, Color};

let mut t = Text::new("Hello World");
t.stylize(
    Style::new().color(Color::parse("red").unwrap()),
    0,           // start byte
    Some(5),     // end byte (exclusive)
);

// style_at(2) now returns a style that includes red
```

When `end` is `None`, the style is applied from `start` to the end of the
string.

```rust
t.stylize(Style::new().bold(true), 6, None);  // style "World" as bold
```

### Spans

Each styled region is represented by a `Span`:

```rust
pub struct Span {
    pub start: usize,   // byte offset, inclusive
    pub end: usize,     // byte offset, exclusive
    pub style: Style,
}
```

```rust
use rusty_rich::{Span, Style};

let span = Span::new(0, 5, Style::new().bold(true));
assert!(!span.is_empty());
```

Span helper methods:

| Method             | Description                                      |
|--------------------|--------------------------------------------------|
| `is_empty()`       | Returns `true` if `end <= start`.                |
| `split(offset)`    | Splits at a byte offset (if inside the range).   |
| `move_by(offset)`  | Shifts both `start` and `end` by a signed delta. |
| `right_crop(max)`  | Clamps `end` to a maximum value.                 |

`split` is the workhorse of word-wrap and line-breaking logic:

```rust
use rusty_rich::{Span, Style};

let span = Span::new(0, 10, Style::new().bold(true));
let (a, b) = span.split(4);

assert_eq!(a.start, 0);
assert_eq!(a.end,   4);
assert_eq!(b.as_ref().unwrap().start, 4);
assert_eq!(b.as_ref().unwrap().end,   10);
```

#### style_at() -- Query the style at a position

Combines the default `Text::style` with every span that covers the given byte
position.

```rust
use rusty_rich::{Text, Style, Color};

let mut t = Text::styled(
    "base",
    Style::new().color(Color::parse("blue").unwrap()),
);
t.append_styled("bold", Style::new().bold(true));

// "base" is blue only
let s0 = t.style_at(0);
assert!(s0.get_bold().is_none());
// The colour comes from the default style
assert!(s0.color.is_some());

// "b" at offset 4 is bold + blue (bold span + default style)
let s4 = t.style_at(4);
assert_eq!(s4.get_bold(), Some(true));
```

### Overflow method

The `OverflowMethod` enum controls what happens when text is wider than the
available space.

| Variant    | Behavior                                            |
|------------|-----------------------------------------------------|
| `Fold`     | Wrap text onto the next line (default).             |
| `Crop`     | Chop the text at the width boundary.                |
| `Ellipsis` | Chop and append a single-character ellipsis (`...`).|
| `Ignore`   | Let the text run past the boundary without clipping.|

```rust
use rusty_rich::{Text, OverflowMethod};

let mut t = Text::new("Hello World");
t.truncate(8, OverflowMethod::Ellipsis);
// plain contains "Hello W..." (or similar, depending on char widths)
```

### truncate()

Truncates the text to a given maximum cell width (measured in Unicode display
cells, not bytes).

```rust
use rusty_rich::{Text, OverflowMethod};

// Ellipsis mode appends the ellipsis character within the width limit
let mut t = Text::new("Hello World");
t.truncate(6, OverflowMethod::Ellipsis);
assert!(t.plain.len() <= 8);  // still fits within max_width

// Crop mode simply cuts
let mut t = Text::new("Hello World");
t.truncate(5, OverflowMethod::Crop);
assert_eq!(t.plain, "Hello");

// Fold and Ignore modes are no-ops for truncate
let mut t = Text::new("Hello World");
t.truncate(5, OverflowMethod::Fold);
assert_eq!(t.plain, "Hello World");  // unchanged
```

Truncation is cell-width aware: wide characters (CJK, emoji) that count as two
cells are handled correctly.

```rust
let mut t = Text::new("Hello \u{1f600}");  // hello + grinning face
t.truncate(6, OverflowMethod::Crop);
// The emoji (2 cells wide) is either kept whole or dropped
```

### Pad, pad_left, pad_right

Adds padding characters around the text. All padding operations shift existing
spans to stay aligned with the content.

```rust
use rusty_rich::Text;

let mut t = Text::new("centered");
t.pad(2, ' ');
assert_eq!(t.plain, "  centered  ");

let mut t = Text::new("right");
t.pad_left(10, ' ');
assert_eq!(t.plain, "          right");

let mut t = Text::new("left");
t.pad_right(10, ' ');
assert_eq!(t.plain, "left          ");
```

The `pad` family works with any character, not just spaces:

```rust
let mut t = Text::new("Chapter 1");
t.pad(1, '=');
assert_eq!(t.plain, "=Chapter 1=");
```

### align()

Aligns text within a given width using the specified method. Internally it
calls `pad_left`, `pad_right`, or both depending on the method.

```rust
use rusty_rich::{Text, AlignMethod};

let mut t = Text::new("Title");
t.align(AlignMethod::Center, 20);
assert_eq!(t.plain, "       Title        ");

let mut t = Text::new("Name");
t.align(AlignMethod::Right, 10);
assert_eq!(t.plain, "      Name");

// No change if text is already equal to or wider than the target width
let mut t = Text::new("A long title that exceeds the width");
t.align(AlignMethod::Left, 5);
assert_eq!(t.plain, "A long title that exceeds the width");  // unchanged
```

### split_lines()

Splits the text on newline boundaries, producing a `Vec<Text>` where each
element holds one line.

```rust
use rusty_rich::Text;

let t = Text::new("line one\nline two\nline three");
let lines: Vec<Text> = t.split_lines();
assert_eq!(lines.len(), 3);
assert_eq!(lines[0].plain, "line one");
assert_eq!(lines[1].plain, "line two");
```

Note that spans are not currently preserved across the split -- each output
`Text` is a fresh plain-text copy.

### Join (combining Text values)

There is no dedicated `join()` method on `Text`, but you can build multi-part
text by chaining `append()` calls:

```rust
use rusty_rich::{Text, Style, Color};

let items = ["alpha", "beta", "gamma"];
let mut t = Text::new("");
for (i, item) in items.iter().enumerate() {
    if i > 0 {
        t.append(", ", None);
    }
    t.append_styled(item, Style::new().bold(true));
}
assert_eq!(t.plain, "alpha, beta, gamma");
```

### highlight_regex()

Finds all non-overlapping matches of a regular expression in the plain text and
adds a `Span` for each one with the given style. Returns the number of matches.

```rust
use rusty_rich::{Text, Style, Color};

let mut t = Text::new("Error code: 42, warning count: 7");
let count = t.highlight_regex(
    r"\d+",
    Style::new().color(Color::parse("cyan").unwrap()).bold(true),
);
assert_eq!(count, 2);

// The matches ("42" and "7") now have a span with cyan + bold.
// style_at(12) returns a style that includes cyan + bold;
// style_at(1) returns only the default style.
```

If the regex pattern is invalid, `highlight_regex` returns 0 and leaves the
text unchanged:

```rust
let mut t = Text::new("hello");
let count = t.highlight_regex(r"[invalid", Style::new().bold(true));
assert_eq!(count, 0);
```

Under the hood this uses the `regex` crate, so all standard regex syntax is
supported (character classes, alternation, groups, etc.).

### render()

Produces a plain ANSI string by emitting the default style, then walking each
character and applying/removing span styles as boundaries are crossed.

```rust
use rusty_rich::{Text, Style, Color};

let mut t = Text::new("Hello ");
t.append_styled("World", Style::new().bold(true));

let ansi = t.render();
// Contains ANSI escape sequences around "World":
// "Hello \x1b[1mWorld\x1b[0m"
```

`Text` also implements `fmt::Display` via `render()`:

```rust
let output = format!("{}", t);
// Same as t.render()
```

When there are no spans and the default style is plain, `render()` returns the
plain text verbatim with no escape sequences:

```rust
let t = Text::new("plain text");
assert_eq!(t.render(), "plain text");
```

#### Style combination during render

When multiple spans overlap at a character boundary, their styles are combined
using `Style::combine()`. The active span styles are layered on top of the
default text style in insertion order.

```rust
let mut t = Text::styled(
    "base",
    Style::new().color(Color::parse("blue").unwrap()),
);
t.append_styled("A", Style::new().bold(true));
t.append_styled("B", Style::new().italic(true));

// "A" is blue + bold
// "B" is blue + italic
```

### wrap()

Splits the text into multiple lines, each not exceeding a given cell width.
Uses simple whitespace-based word wrapping.

```rust
use rusty_rich::Text;

let t = Text::new("This is a long sentence that will wrap");
let lines = t.wrap(15);

assert_eq!(lines.len(), 3);
assert_eq!(lines[0].plain, "This is a long");
assert_eq!(lines[1].plain, "sentence that");
assert_eq!(lines[2].plain, "will wrap");
```

Words longer than the width are placed on their own line (they will overflow
unless the downstream renderer applies additional truncation).

### expand_tabs()

Replaces tab characters with spaces using the standard 8-column tab stops.

```rust
use rusty_rich::Text;

let mut t = Text::new("col1\tcol2\tcol3");
t.expand_tabs();
// "col1    col2    col3"
```

### cell_len()

Returns the display width of the plain text measured in Unicode cells. This is
not the same as `plain.len()` because CJK characters and emoji occupy two cells
while combining marks occupy zero.

```rust
use rusty_rich::Text;

let t = Text::new("Hello");
assert_eq!(t.cell_len(), 5);

let t = Text::new("\u{1f600}");  // grinning face emoji
assert_eq!(t.cell_len(), 2);     // wide character
```

---

## Console Markup

Console markup is rusty-rich's inline styling language, closely modelled on
Python Rich's markup syntax. It uses BBCode-style tags embedded directly in
strings.

### Using markup::render()

The function `markup::render(markup: &str) -> Text` parses a markup string and
returns a fully styled `Text` ready for rendering.

```rust
use rusty_rich::markup;

let text: rusty_rich::Text = markup::render("[bold]Hello[/bold]");
assert_eq!(text.plain, "Hello");
assert!(!text.spans.is_empty());
```

This is the idiomatic way to build a `Text` from markup in rusty-rich. It
replaces Python Rich's `Text.from_markup()` class method.

### Tag syntax

#### Opening tags

```
[bold]          -> apply bold
[red]           -> set foreground colour to red
[on blue]       -> set background colour to blue
[bold red]      -> bold + red foreground
[bold red on blue]  -> bold + red foreground + blue background
[#ff6600]       -> set foreground to hex colour
[color(red)]    -> via parameter syntax
[color=red]     -> via key=value syntax
```

#### Closing tags

```
[/]             -> close all open tags (reset to default)
[/bold]         -> close the most recently opened bold tag
[/red]          -> close the most recently opened red tag
```

The parser tracks a style stack. An opening tag pushes a new style onto the
stack; `[/]` pops everything; `[/name]` pops one level regardless of the name.

#### Supported style tags

| Tag                  | Effect                                                 |
|----------------------|--------------------------------------------------------|
| `[bold]` / `[b]`    | Bold text.                                             |
| `[dim]` / `[d]`     | Dim / faint text.                                      |
| `[italic]` / `[i]`  | Italic text.                                           |
| `[underline]` / `[u]`| Underlined text.                                      |
| `[blink]`            | Blinking text.                                         |
| `[reverse]` / `[r]` | Reverse video (swap fg/bg).                            |
| `[strike]` / `[s]`  | Strikethrough text.                                    |

#### Colour tags

Any recognised colour name can be used as a tag. The parser tries the following
in order:

1. **Attribute tag** -- bold, italic, dim, underline, blink, reverse, strike.
2. **Background colour** -- if the tag starts with `on ` (e.g. `[on red]`).
3. **Combined fg + bg** -- if the tag contains ` on ` (e.g. `[red on blue]`).
4. **Colour name** -- bare name like `[red]`, `[bright_green]`, `[#ff6600]`.
5. **Parameterised colour** -- `[color=red]`, `[color(red)]`, `[color(#ff6600)]`.

```rust
use rusty_rich::markup;

// Background colour
markup::render("[on blue]background[/on blue]");

// Combined foreground and background
markup::render("[bright_yellow on blue]warning[/]");

// Hex colour
markup::render("[#ff6600]orange text[/]");

// Parameterised
markup::render("[color=red]via key=value[/]");
markup::render("[color(red)]via parentheses[/]");
```

### Nesting

Tags can be nested. Each tag pushes a new style onto the stack, and the
effective style at any point is the combination of all active styles.

```rust
use rusty_rich::markup;

let t = markup::render("[bold]bold [red]bold red[/red] just bold[/bold]");
// "bold "            -> bold only
// "bold red"         -> bold + red
// " just bold"       -> bold only
```

### Literal brackets

To include a literal `[` character in output, use `[[`. This is the escape
mechanism.

```rust
use rusty_rich::markup;

let t = markup::render("[[ERROR]]: File not found");
assert_eq!(t.plain, "[ERROR]: File not found");
```

### Unclosed tags

If an opening `[` is found but no matching `]` exists before the end of the
string, the `[` is treated as a literal character:

```rust
use rusty_rich::markup;

let t = markup::render("This is [not a tag");
assert_eq!(t.plain, "This is [not a tag");
```

### Using markup with Console

The `Console::print_str()` method applies markup rendering internally before
outputting, so you rarely need to call `markup::render()` directly when simply
printing.

```rust
use rusty_rich::Console;

let mut console = Console::new();
console.print_str("[bold green]Success![/bold green] Operation completed.\n");
```

When markup is disabled on the console (`options.markup = false`), strings are
printed verbatim and `[[` is not needed:

```rust
let mut console = Console::new();
console.options.markup = false;
console.print_str("[bold]This shows as plain text[/bold]\n");
```

---

## escape()

The `markup::escape()` function pre-processes a string so that it will not be
interpreted as markup. It replaces every `[` with `[[`.

```rust
use rusty_rich::markup;

let safe = markup::escape("Use [bold] for bold text");
assert_eq!(safe, "Use [[bold]] for bold text");

// When rendered, the text appears literally:
let t = markup::render(&safe);
assert_eq!(t.plain, "Use [bold] for bold text");
```

This is essential when displaying user-provided input that might contain
bracket characters:

```rust
use rusty_rich::{Console, markup};

fn display_filename(console: &mut Console, filename: &str) {
    let safe = markup::escape(filename);
    console.print_str(&format!("File: [bold]{safe}[/bold]\n"));
}

display_filename(&mut console, "report[final].md");
// Output: File: report[final].md
```

---

## TextType

`TextType` is a convenience enum that accepts either a plain `String` or a
pre-styled `Text`. It is used in APIs where the caller may have already done
the styling work or just wants to pass a plain string.

```rust
pub enum TextType {
    Plain(String),
    Rich(Text),
}
```

```rust
use rusty_rich::{Text, text::TextType};

// From a plain string
let tt: TextType = "hello".into();
assert_eq!(tt.render(), "hello");

// From a styled Text
let styled = Text::styled("world", rusty_rich::Style::new().bold(true));
let tt: TextType = styled.into();
// render() produces ANSI-wrapped output
```

Conversions are provided from `&str`, `String`, and `Text`.

---

## Integration with Segments

The rendering pipeline converts `Text` into `Segment` values. Each segment is a
plain string plus an optional style. The `Console::render()` method ultimately
flattens `Text` objects into a flat sequence of segments.

```rust
use rusty_rich::{Console, Segment, Style};

let mut buffer = Vec::new();
let mut console = Console::with_file(Box::new(&mut buffer));

let text = rusty_rich::Text::styled(
    "Hello",
    Style::new().bold(true),
);

let segments = console.render(&text, &console.options);
// segments is Vec<Segment> -- the text's ANSI is split into segments
```

---

## Text and Renderable

`Text` implements the `Renderable` trait, so it can be passed directly to any
method that accepts a renderable:

```rust
use rusty_rich::{Console, Text, Style, Color};

let mut console = Console::new();

let t = Text::styled(
    "Greetings from a custom Text renderable\n",
    Style::new().color(Color::parse("cyan").unwrap()),
);
console.println(&t);
```

The same is true for `&str` and `String`, which are also `Renderable` -- they
are converted to plain `Segment` values during rendering.

---

## Putting it all together

A complete example that builds a multi-style message with markup, regex
highlighting, word-wrapping, and padding:

```rust
use rusty_rich::{
    Console, Text, Style, Color, OverflowMethod, AlignMethod,
    markup,
};

let mut console = Console::new();

// 1. Build from markup
let mut msg = markup::render("[bold]Results:[/bold] ");

// 2. Append styled text
msg.append_styled(
    "42 passed",
    Style::new().color(Color::parse("green").unwrap()).bold(true),
);
msg.append(" | ", None);
msg.append_styled(
    "3 failed",
    Style::new().color(Color::parse("red").unwrap()),
);

// 3. Highlight numbers with regex
let errors = "Errors in: main.rs:7, lib.rs:42, util.rs:15";
let mut err_text = Text::new(errors);
err_text.highlight_regex(
    r"\d+",
    Style::new().color(Color::parse("cyan").unwrap()).bold(true),
);
err_text.highlight_regex(
    r"\w+\.rs",
    Style::new().color(Color::parse("magenta").unwrap()),
);

// 4. Pad for indentation
msg.pad_left(4, ' ');
err_text.pad_left(4, ' ');

// 5. Print both
console.println(&msg);
console.println(&err_text);

// 6. Escape user input
let user_input = "Config value: [debug] enabled";
let safe = markup::escape(user_input);
console.print_str(&format!("  [dim]{safe}[/dim]\n"));
```

---

## Summary

| Concept              | Primary API                           |
|----------------------|---------------------------------------|
| Plain text           | `Text::new()`, `Text::from()`         |
| Text with style      | `Text::styled()`, `.style()`          |
| Append text          | `.append()`, `.append_styled()`       |
| Style subrange       | `.stylize()`                          |
| Regex highlighting   | `.highlight_regex()`                  |
| Truncation           | `.truncate()` + `OverflowMethod`      |
| Padding              | `.pad()`, `.pad_left()`, `.pad_right()`|
| Alignment            | `.align()`                            |
| Word wrap            | `.wrap()`                             |
| ANSI output          | `.render()`, `format!("{}", t)`       |
| Markup parse         | `markup::render()`                    |
| Escape markup        | `markup::escape()`                    |
| Style lookup at pos  | `.style_at()`                         |
| Display width        | `.cell_len()`                         |
