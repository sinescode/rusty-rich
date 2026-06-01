# Style and Color

The style system is the foundation of all visual output in rust-rich. It combines
`Color` (foreground/background color values) with `Style` (color plus text
attributes and hyperlinks) to describe how text should appear in the terminal.

---

## Color

`Color` is the atomic unit of terminal color. It can represent one of four
kinds of value, distinguished by `ColorType`:

| Variant | Description |
|---|---|
| `Default` | Inherit from the surrounding context (no explicit color). |
| `Standard` | One of the 16 ANSI named colors (black, red, green, ..., bright_white). |
| `EightBit` | An index (0-255) into the 256-color palette. |
| `TrueColor` | A 24-bit RGB triple `(r, g, b)`. |

### Color parsing

`Color::parse(s)` accepts a flexible string format and returns
`Result<Color, ColorParseError>`.

```rust
use rusty_rich::Color;

// Named ANSI colors (case-insensitive)
let red   = Color::parse("red").unwrap();
let green = Color::parse("bright_green").unwrap();

// Hex notation — with or without leading #
let hex1 = Color::parse("#ff6600").unwrap();
let hex2 = Color::parse("ff6600").unwrap();

// 8-bit palette index
let c200 = Color::parse("color200").unwrap();

// Default / inherit
let def = Color::parse("default").unwrap();
assert!(def.is_default());

let empty = Color::parse("").unwrap();
assert!(empty.is_default());

// Unknown names produce an error
let err = Color::parse("burlywood");
assert!(err.is_err());
```

### Named ANSI colors

The 16 standard ANSI colors are available by name:

```rust
// Standard set (indices 0-15)
Color::parse("black");       // index 0
Color::parse("red");         // index 1
Color::parse("green");       // index 2
Color::parse("yellow");      // index 3
Color::parse("blue");        // index 4
Color::parse("magenta");     // index 5
Color::parse("cyan");        // index 6
Color::parse("white");       // index 7
Color::parse("bright_black");    // index 8
Color::parse("bright_red");      // index 9
Color::parse("bright_green");    // index 10
Color::parse("bright_yellow");   // index 11
Color::parse("bright_blue");     // index 12
Color::parse("bright_magenta");  // index 13
Color::parse("bright_cyan");     // index 14
Color::parse("bright_white");    // index 15
```

A much larger set of extended color names is also recognised — covering the
full X11 / web color namespace mapped to 8-bit palette entries. Examples:

```rust
Color::parse("navy_blue");
Color::parse("dark_green");
Color::parse("cornflower_blue");
Color::parse("orange4");
Color::parse("light_grey");
```

Use `Color::name_to_index(name)` to look up the palette index for any
recognised name.

### Constructors

```rust
use rusty_rich::Color;

// Default (inherit)
let c1 = Color::default();

// From hex
let c2 = Color::from_hex("#ff0000").unwrap();
let c3 = Color::from_rgb(255, 0, 0);

// From 8-bit palette index
let c4 = Color::from_8bit(196);   // bright red in 256 palette

// From named ANSI value
let c5 = Color::from_ansi_name("bright_green").unwrap();
```

### ColorTriplet

The `ColorTriplet` struct represents a raw RGB triple and is used internally
for blending and palette operations.

```rust
use rusty_rich::color::ColorTriplet;

let t = ColorTriplet::new(255, 128, 64);
assert_eq!(t.to_string(), "#ff8040");
```

---

## ColorSystem

`ColorSystem` is an enum that describes what the terminal supports. It
determines how colors are encoded in ANSI escape sequences.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorSystem {
    Standard = 1,   // 3-bit / 4-bit ANSI (8/16 colors)
    EightBit = 2,   // 8-bit (256 colors)
    TrueColor = 3,  // 24-bit true color
}
```

The three variants have an implied ordering: `Standard < EightBit < TrueColor`.

```rust
assert!(ColorSystem::Standard < ColorSystem::TrueColor);
```

### Display representation

```rust
assert_eq!(ColorSystem::Standard.to_string(),  "standard");
assert_eq!(ColorSystem::EightBit.to_string(),  "256");
assert_eq!(ColorSystem::TrueColor.to_string(), "truecolor");
```

### Color downgrade

When the terminal does not support the full color depth of a `Color`, the
`downgrade()` method converts it to the nearest available representation.

#### TrueColor -> EightBit

Downloads from 24-bit RGB to the nearest entry in the 256-color palette using
`rgb_to_8bit()`. This function finds the closest colour in a 6x6x6 colour cube
plus a 24-step grey ramp.

```rust
use rusty_rich::Color;
use rusty_rich::color::ColorSystem;

let tc = Color::from_rgb(255, 0, 0);          // TrueColor red
let eight = tc.downgrade(ColorSystem::EightBit);
// eight is now ColorType::EightBit at the nearest 256-palette index
```

#### TrueColor -> Standard

Downloads to one of the 16 ANSI colours using Euclidean distance in RGB space
via `rgb_to_standard()`.

```rust
let tc = Color::from_rgb(128, 0, 0);          // dark red
let std = tc.downgrade(ColorSystem::Standard);
// std is now ColorType::Standard, index 1 (red)
```

#### EightBit -> Standard

8-bit indices 16-255 are collapsed into the closest standard colour
(`index % 16`).

```rust
let c = Color::from_8bit(196);
let std = c.downgrade(ColorSystem::Standard);
```

### RGB utilities

The `color` module provides several utility functions for colour arithmetic.

**`rgb_to_8bit(r, g, b) -> u8`**

Finds the closest 8-bit palette entry for an RGB colour. Greyscale values
(matching R=G=B) map to the neutral ramp (indices 232-255). Other values
map to the 6x6x6 colour cube (indices 16-231).

**`rgb_to_standard(r, g, b) -> u8`**

Finds the closest standard (0-15) colour by Euclidean distance in RGB space.

**`blend_rgb(color1, color2, cross_fade) -> (u8, u8, u8)`**

Linearly interpolates between two RGB tuples. A `cross_fade` of 0.0 returns
`color1`; 1.0 returns `color2`.

**`blend_colors(color1, color2, cross_fade, theme) -> Color`**

Blends two `Color` values by resolving them to RGB via `get_truecolor()` and
then interpolating. Returns a TrueColor result.

```rust
use rusty_rich::color::{blend_rgb, blend_colors, TerminalTheme};
use rusty_rich::Color;

let a = (255, 0, 0);
let b = (0, 0, 255);
let mid = blend_rgb(a, b, 0.5);
assert_eq!(mid, (128, 0, 128));  // purple

let theme = TerminalTheme::default();
let red = Color::parse("red").unwrap();
let blue = Color::parse("blue").unwrap();
let purple = blend_colors(&red, &blue, 0.5, &theme);
```

### TerminalTheme

`TerminalTheme` describes the terminal's default foreground and background
colours, used when resolving default/inherit colours to true colour.

```rust
use rusty_rich::color::TerminalTheme;

let theme = TerminalTheme::default();
assert_eq!(theme.foreground_color, (255, 255, 255));  // white
assert_eq!(theme.background_color, (0, 0, 0));        // black
```

### Palette data

The module exposes two static palettes:

- `STANDARD_PALETTE` — the RGB values of the 16 ANSI colours (index 0-15).
- `EIGHT_BIT_PALETTE` — the full 256-colour table, lazily built from the 16
  standard colours, a 6x6x6 colour cube (indices 16-231), and a 24-step grey
  ramp (indices 232-255).

---

## Style

A `Style` combines a foreground colour, background colour, a set of boolean
text attributes, and an optional hyperlink. It uses a **three-state** system
for attributes: a given attribute bit may be set to `true`, set to `false`, or
**not set** (inherited from a parent or base style).

```rust
pub struct Style {
    pub(crate) color: Option<Color>,
    pub(crate) bgcolor: Option<Color>,
    pub(crate) attributes: Attributes,
    pub(crate) set_attributes: u32,    // mask of explicitly-set bits
    pub(crate) link: Option<String>,
    pub(crate) is_null: bool,
    pub(crate) meta: Option<Vec<u8>>,
}
```

### Null style

A null style (`Style::null()`) is an identity value — combining any style with
null returns the other style unchanged. Closing tags (e.g. `[/]`) produce null
styles in the markup parser.

```rust
let null = Style::null();
assert!(null.is_null());

let s = Style::new().bold(true);
let combined = s.combine(&Style::null());
assert_eq!(combined.get_bold(), Some(true));  // null doesn't override
```

### The builder pattern

`Style` uses a method-chaining builder. Every attribute setter consumes and
returns `self`.

#### Foreground and background colour

```rust
use rusty_rich::{Style, Color};

let s = Style::new()
    .color(Color::parse("red").unwrap())
    .bgcolor(Color::parse("bright_black").unwrap());
```

The colour setters accept anything implementing `Into<Option<Color>>`:

```rust
Style::new().color(Color::parse("blue").unwrap());
Style::new().color(None);       // clear the foreground colour
```

#### Text attributes

```rust
let s = Style::new()
    .bold(true)
    .dim(false)
    .italic(true)
    .underline(true)
    .blink(true)
    .reverse(true)
    .strike(true);
```

Each setter marks the attribute as **explicitly set** (it will not be
inherited). Setting `false` means "explicitly turn this off" — different from
"not set."

| Method | ANSI on | ANSI off | Notes |
|---|---|---|---|
| `.bold(bool)` | `1` | `22` | Bolder text (also turns off dim) |
| `.dim(bool)` | `2` | `22` | Faint / dim text |
| `.italic(bool)` | `3` | `23` | Italic (not widely supported) |
| `.underline(bool)` | `4` | `24` | Single underline |
| `.blink(bool)` | `5` | `25` | Slow blink (less than 150 per minute) |
| `.blink2(bool)` | `6` | `25` | Rapid blink |
| `.reverse(bool)` | `7` | `27` | Swap foreground and background |
| `.conceal(bool)` | `8` | `28` | Conceal (hiding text) |
| `.strike(bool)` | `9` | `29` | Strikethrough |
| `.underline2(bool)` | `21` | `24` | Double underline |
| `.frame(bool)` | `51` | `54` | Framed text |
| `.encircle(bool)` | `52` | `54` | Encircled text |
| `.overline(bool)` | `53` | `55` | Overline |

Note that `bold(false)` and `dim(false)` both emit ANSI code `22` (normal
intensity), and multiple underline/overline attributes share the same reset
codes.

#### Chaining everything

```rust
let s = Style::new()
    .color(Color::parse("bright_green").unwrap())
    .bgcolor(Color::parse("black").unwrap())
    .bold(true)
    .italic(true);
```

### Parsing style from a string

`Style::from_str(definition)` parses a space-separated style string, mirroring
the BBCode-like syntax used in Rich markup.

```rust
let s = Style::from_str("bold red on bright_black");
// Equivalent to:
Style::new()
    .bold(true)
    .color(Color::parse("red").unwrap())
    .bgcolor(Color::parse("bright_black").unwrap());
```

Supported tokens:

| Token | Effect |
|---|---|
| `bold` / `b` | Enable bold |
| `dim` / `d` | Enable dim |
| `italic` / `i` | Enable italic |
| `underline` / `u` | Enable underline |
| `blink` | Enable blink |
| `reverse` / `r` | Enable reverse |
| `strike` / `s` | Enable strikethrough |
| `not bold` / `!bold` / `nobold` | Explicitly disable bold |
| `not italic` / `!italic` / `noitalic` | Explicitly disable italic |
| `not underline` / `!underline` / `nounderline` | Explicitly disable underline |
| `on <color>` | Set background colour |
| `link=<url>` | Set hyperlink |
| `none` / `default` | No effect (resets nothing) |
| Any colour name | Set foreground colour |

The parser processes tokens left to right. The word `on` is special: the next
token sets the background colour. Any unrecognised token is tried as a colour
name.

```rust
// Foreground + background + attribute
let s = Style::from_str("bright_yellow on blue bold");

// With hyperlink
let s = Style::from_str("underline blue link=https://example.com");

// Disable attributes
let s = Style::from_str("not bold !italic");
```

### Composing styles with combine()

Two styles are combined with `base.combine(&other)`. The **other** style's
explicitly-set attributes override those of the base. Unset attributes on
`other` are inherited from `base`.

```rust
let base = Style::from_str("red");
let bold = Style::from_str("bold");

let combined = base.combine(&bold);
// Result: colour=red, bold=true, italic=not-set, ...
```

This follows a three-state cascade:

| Base attribute | Other attribute | Result |
|---|---|---|
| not set | not set | not set |
| not set | set true | true |
| not set | set false | false |
| set true | not set | true (inherited) |
| set true | set false | false |

Colours and hyperlinks from `other` always override the base if they are
`Some(...)`.

```rust
let default = Style::new();
let green_bold = Style::from_str("bold green");
let result = default.combine(&green_bold);
// result has color=green, bold=true
```

The null style is the identity element:

```rust
let a = Style::from_str("red");
assert_eq!(a.combine(&Style::null()), a);
assert_eq!(Style::null().combine(&a), a);
```

### Querying attributes

Each attribute has a getter that returns `Option<bool>`:

- `None` — the attribute is **not set** (inherited).
- `Some(true)` — the attribute is explicitly enabled.
- `Some(false)` — the attribute is explicitly disabled.

```rust
let s = Style::new().bold(true);
assert_eq!(s.get_bold(), Some(true));

let s = Style::new();
assert_eq!(s.get_bold(), None);   // not set

let s = Style::new().bold(false);
assert_eq!(s.get_bold(), Some(false));  // explicitly off
```

### Checking style state

```rust
let null = Style::null();
assert!(null.is_null());

let plain = Style::new();
assert!(plain.is_plain());   // no colours, no attributes, no link

let s = Style::new().bold(true);
assert!(!s.is_plain());
```

### without_color()

Returns a copy of the style with both foreground and background colours
stripped. Attributes and hyperlinks are preserved.

```rust
let s = Style::from_str("bold red on blue");
let no_color = s.without_color();
assert!(no_color.color.is_none());
assert!(no_color.bgcolor.is_none());
assert_eq!(no_color.get_bold(), Some(true));
```

### background_style()

Returns a new style whose background colour is set to the original style's
foreground colour. This is useful for rendering background-only fills.

```rust
let fg = Style::from_str("red");
let bg = fg.background_style();
// bg has no foreground colour; its background is red
```

### Display representation

```rust
let s = Style::from_str("bold red on blue");
assert_eq!(s.to_string(), "red on blue bold");

let s = Style::new();
assert_eq!(s.to_string(), "none");
```

---

## Hyperlinks

OSC-8 hyperlinks are supported via the `link()` builder method.

```rust
let linked = Style::new()
    .underline(true)
    .color(Color::parse("bright_blue").unwrap())
    .link("https://example.com");
```

When rendered to ANSI, hyperlinks are encoded using the standard OSC-8 escape
sequence (`\x1b]8;;<url>\x1b\\ ... \x1b]8;;\x1b\\`). Each link gets a unique
`link_id` for terminal-internal tracking.

The link can also be set via `Style::from_str`:

```rust
let s = Style::from_str("underline link=https://docs.rs/rusty_rich");
```

---

## ANSI rendering

The `to_ansi()` method on `Style` produces the SGR (Select Graphic Rendition)
escape sequence for the style's colour and attributes. The `reset_ansi()` method
returns `\x1b[0m` to reset all attributes.

```rust
let s = Style::new()
    .color(Color::parse("red").unwrap())
    .bold(true);
assert_eq!(s.to_ansi(), "\x1b[31;1m");
```

The method builds a semicolon-separated list of ANSI parameter codes:

- Foreground Standard (0-7): `30-37`
- Foreground Standard (8-15): `90-97`
- Foreground EightBit: `38;5;N`
- Foreground TrueColor: `38;2;R;G;B`
- Background Standard (0-7): `40-47`
- Background Standard (8-15): `100-107`
- Background EightBit: `48;5;N`
- Background TrueColor: `48;2;R;G;B`
- Default foreground: `39`
- Default background: `49`

---

## StyleStack

`StyleStack` is used internally when rendering nested markup. It maintains a
stack of `Style` values and provides the combined view of all active styles.

```rust
use rusty_rich::{Style, StyleStack};

let mut stack = StyleStack::new(Style::new());   // base style

// Enter a [bold] region
stack.push(Style::from_str("bold"));
assert_eq!(stack.len(), 1);

let current = stack.current();
assert_eq!(current.get_bold(), Some(true));

// Enter a nested [red] region
stack.push(Style::from_str("red"));
let current = stack.current();
// Now combines: base + bold + red

// Exit back to [bold] only
stack.pop();

// Exit to base
stack.pop();
assert!(stack.is_empty());
```

The `current()` method walks the stack from bottom to top, combining each
style in order. The base style is always applied first:

```rust
let mut stack = StyleStack::new(Style::new());   // base
stack.push(Style::from_str("bold"));
stack.push(Style::from_str("red on blue"));

let current = stack.current();
// Equivalent to: Style::new().combine(&bold).combine(&red_on_blue)
```

---

## Theme

A `Theme` is a mapping from style names (like `"repr.number"` or
`"table.header"`) to `Style` values. It provides a central place to customise
the appearance of different renderable types.

```rust
use rusty_rich::{Theme, Style, Color};

let mut theme = Theme::new();
theme.set("repr.number",
    Style::new().color(Color::parse("cyan").unwrap()).bold(true));
theme.set("repr.str",
    Style::new().color(Color::parse("green").unwrap()));
theme.set("table.header",
    Style::new().bold(true).color(Color::parse("white").unwrap()));
```

### Inheritance

A theme can inherit from a parent theme. Lookups fall back to the parent if
the name is not found in the child.

```rust
let parent = Theme::new();
parent.set("repr.number", Style::from_str("cyan"));

let child = Theme::with_inherit(parent);
// inherits "repr.number" from parent
assert!(child.get("repr.number").is_some());
// child can override specific styles
child.set("repr.number", Style::from_str("bold cyan"));
```

### Merging

`theme.merge(&other)` inserts all styles from `other` that are not already
present in `self`.

```rust
let mut a = Theme::new();
a.set("a", Style::from_str("red"));

let b = Theme::new();
b.set("b", Style::from_str("blue"));

a.merge(&b);
assert!(a.get("a").is_some());
assert!(a.get("b").is_some());
```

### ThemeStack

`ThemeStack` is a stack of themes. When looking up a style name, themes are
searched from top (most recently pushed) to bottom.

```rust
use rusty_rich::theme::ThemeStack;

let mut stack = ThemeStack::new();

let mut dark = Theme::new();
dark.set("text", Style::from_str("white"));
stack.push(dark);

let mut accent = Theme::new();
accent.set("text", Style::from_str("bright_green"));
stack.push(accent);

// Topmost theme wins
assert_eq!(
    stack.get("text").unwrap().to_string(),
    "bright_green"
);
```

### Default theme style names

The built-in `theme::names` module defines constants for every style name used
across the library. Here are the categories:

**repr / pretty printing:**
`repr.number`, `repr.str`, `repr.bool_true`, `repr.bool_false`,
`repr.none`, `repr.url`, `repr.path`, `repr.ipv4`, `repr.ipv6`,
`repr.ellipsis`, `repr.attrib_name`, `repr.attrib_value`,
`repr.tag_name`, `repr.tag_contents`, `repr.tag_punctuation`

**Table:**
`table.header`, `table.footer`, `table.title`, `table.caption`,
`table.border`

**Logging:**
`logging.keyword`, `logging.level.debug`, `logging.level.info`,
`logging.level.warning`, `logging.level.error`, `logging.level.critical`

**Traceback:**
`traceback.border`, `traceback.title`, `traceback.error`,
`traceback.error_mark`, `traceback.filename`, `traceback.line_no`,
`traceback.locals_header`

**Rule:**
`rule.line`, `rule.text`

**Tree:**
`tree`, `tree.line`

**Progress bars:**
`bar.complete`, `bar.finished`, `bar.pulse`,
`progress.description`, `progress.percentage`, `progress.remaining`,
`progress.elapsed`, `progress.data`

**Markdown:**
`markdown.h1`, `markdown.h2`, `markdown.code`, `markdown.link`,
`markdown.item`, `markdown.blockquote`

**JSON:**
`json.key`, `json.str`, `json.number`, `json.bool`, `json.null`,
`json.brace`

**Syntax highlighting:**
`syntax.comment`, `syntax.keyword`, `syntax.string`, `syntax.number`,
`syntax.function`, `syntax.type`

### Default theme

Calling `default_theme()` returns a theme pre-populated with a standard set
of styles that mirror Python Rich's defaults.

```rust
use rusty_rich::theme::default_theme;

let theme = default_theme();
// Numbers appear cyan and bold
let number_style = theme.get("repr.number").unwrap();
// Strings appear green
let str_style = theme.get("repr.str").unwrap();
```

---

## Style in the markup system

The markup parser (`rusty_rich::markup`) converts BBCode-like tags into
`Style` values via a pattern-matching function `tag_to_style`. This enables
inline styling in string content.

```rust
use rusty_rich::markup;

// Tags map directly to style attributes:
// [bold]  -> Style::new().bold(true)
// [red]   -> Style::new().color(Color::parse("red").unwrap())
// [on blue] -> Style::new().bgcolor(Color::parse("blue").unwrap())
// [bold red on black] -> combined style

let text = markup::render("[bold green]Hello[/bold green]!");
// Returns a Text with a "Hello" span styled bold + green
```

The recognised markup tags are:

| Tag | Style builder call |
|---|---|
| `[bold]` / `[b]` | `.bold(true)` |
| `[dim]` / `[d]` | `.dim(true)` |
| `[italic]` / `[i]` | `.italic(true)` |
| `[underline]` / `[u]` | `.underline(true)` |
| `[blink]` | `.blink(true)` |
| `[reverse]` / `[r]` | `.reverse(true)` |
| `[strike]` / `[s]` | `.strike(true)` |
| `[<color>]` | `.color(Color::parse(name).unwrap())` |
| `[on <color>]` | `.bgcolor(Color::parse(name).unwrap())` |
| `[<fg> on <bg>]` | `.color(fg).bgcolor(bg)` |
| `[<tag>=<value>]` | `.color(Color::parse(value).unwrap())` |
| `[/]` | null style (closes all) |
| `[/bold]` | null style (pops one level) |

Literal `[` is written as `[[`.

---

## Style in Segments

Every `Segment` carries an optional `Style`. When rendered to ANSI, the
segment wraps its text with the style's escape sequence followed by a reset.

```rust
use rusty_rich::{Segment, Style};

let s = Style::new().bold(true).color(Color::parse("red").unwrap());
let seg = Segment::styled("Danger!", s);
// Renders as: \x1b[31;1mDanger!\x1b[0m
```

The `Segments` collection groups multiple styled segments together for
efficient rendering.

---

## Style in Text spans

The `Text` type stores styled regions as `Span` values, each with a `start`
byte offset, `end` byte offset, and a `Style`. The `style_at(position)` method
resolves the effective style at a given position by combining the default text
style with every active span.

```rust
use rusty_rich::{Text, Style};

let mut t = Text::new("Hello ");
t.append_styled("World", Style::new().bold(true));

// style_at(0) returns the default style (no bold)
// style_at(6) returns a style with bold=true
```

The `stylize` method applies a style over a subrange:

```rust
t.stylize(Style::from_str("red"), 0, Some(5));  // style "Hello" in red
```

---

## Putting it all together

A complete example that exercises the full style and colour system:

```rust
use rusty_rich::{
    Color, ColorSystem, Style, StyleStack, Theme,
    Segment, Segments, Text,
};

// 1. Create colours via different methods
let fg = Color::parse("bright_green").unwrap();
let bg = Color::parse("color235").unwrap();   // near-black in 256-palette
let accent = Color::from_hex("#ff6600").unwrap();

// 2. Build a style with builder
let heading = Style::new()
    .color(accent)
    .bold(true)
    .underline(true)
    .link("https://example.com");

// 3. Parse a style from a string
let subtitle = Style::from_str("italic bright_white on color235");

// 4. Combine two styles
let combined = heading.combine(&subtitle);

// 5. Verify the style renders to ANSI
println!("Heading ANSI: {:?}", heading.to_ansi());

// 6. Colour downgrade for older terminals
let downgraded = fg.downgrade(ColorSystem::EightBit);

// 7. Blend two colours
let blended = Color::from_rgb(255, 0, 0)
    .downgrade(ColorSystem::EightBit);
println!("Blended to 8-bit index: {:?}", blended.number);

// 8. Use StyleStack for nested regions
let mut stack = StyleStack::new(Style::new());
stack.push(Style::from_str("bold"));
stack.push(Style::from_str("red"));
let current = stack.current();
println!("Nested style: {}", current);

// 9. Create a theme
let mut theme = Theme::new();
theme.set("custom.heading", heading.clone());
theme.set("custom.body", Style::from_str("white"));

// 10. Build segments
let segments = Segments::from(vec![
    Segment::styled("Styled text\n", heading),
    Segment::styled("Normal text\n", Style::new()),
]);

// 11. Build a Text with spans
let mut text = Text::new("This is ");
text.append_styled("styled", Style::new().bold(true));
text.append_styled(" text.", Style::new().italic(true));

// 12. Render to ANSI string
let output = text.render();
println!("{}", output);
```
