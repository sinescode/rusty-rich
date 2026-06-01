# Themes

A `Theme` is a mapping from named style identifiers (like `"repr.number"` or
`"table.header"`) to concrete `Style` values. Themes let you change the colour
scheme of every renderable at once -- tables, trees, progress bars, markdown,
JSON output, tracebacks, and more -- without touching individual component
configuration.

---

## The Theme struct

```rust
#[derive(Debug, Clone)]
pub struct Theme {
    pub styles: HashMap<String, Style>,
    pub inherit: Option<Box<Theme>>,
}
```

The `styles` field holds the direct mapping. The optional `inherit` field
points to a parent theme that is consulted when a style name is not found in
this theme.

### Creating a theme

```rust
use rusty_rich::{Theme, Style};

let mut theme = Theme::new();
theme.set("repr.number", Style::from_str("bold cyan"));
theme.set("repr.str",    Style::from_str("green"));
theme.set("table.header", Style::from_str("bold white"));
```

### Looking up a style

```rust
let style = theme.get("repr.number");
// Returns Some(&Style) if found in this theme or a parent, None otherwise
```

### Merging themes

`theme.merge(&other)` inserts every style from `other` that does not already
exist in `self`.

```rust
let mut a = Theme::new();
a.set("heading", Style::from_str("bold white"));

let b = Theme::new();
b.set("heading", Style::from_str("bold cyan")); // will NOT override
b.set("body",    Style::from_str("white"));

a.merge(&b);
// a still has "heading" = "bold white" (not overwritten)
// a now has "body" = "white"
```

---

## Default theme

`default_theme()` returns a pre-populated `Theme` that mirrors Python Rich's
default colour scheme. It is the theme used by new `Console` instances.

```rust
use rusty_rich::theme::default_theme;

let theme = default_theme();
let style = theme.get("repr.number").unwrap();
// bold cyan
```

The default theme covers all built-in style name categories: repr, table,
rule, tree, progress, markdown, JSON, traceback, and syntax highlighting.

---

## Style name constants

The `theme::names` module defines string constants for every recognised style
name. These are the keys you use with `Theme::set()` and `Theme::get()`.

### repr / pretty-printing

| Constant | Value | Default style |
|---|---|---|
| `REPR_NUMBER` | `"repr.number"` | bold cyan |
| `REPR_STR` | `"repr.str"` | green |
| `REPR_BOOL_TRUE` | `"repr.bool_true"` | bold bright_green |
| `REPR_BOOL_FALSE` | `"repr.bool_false"` | bold bright_red |
| `REPR_NONE` | `"repr.none"` | dim bright_yellow |
| `REPR_URL` | `"repr.url"` | underline bright_blue |
| `REPR_PATH` | `"repr.path"` | magenta |
| `REPR_IPV4` | `"repr.ipv4"` | (inherited) |
| `REPR_IPV6` | `"repr.ipv6"` | (inherited) |
| `REPR_ELLIPSIS` | `"repr.ellipsis"` | (inherited) |
| `REPR_ATTRIB_NAME` | `"repr.attrib_name"` | bright_cyan |
| `REPR_ATTRIB_VALUE` | `"repr.attrib_value"` | white |
| `REPR_TAG_NAME` | `"repr.tag_name"` | (inherited) |
| `REPR_TAG_CONTENTS` | `"repr.tag_contents"` | (inherited) |
| `REPR_TAG_PUNCTUATION` | `"repr.tag_punctuation"` | (inherited) |

### Table

| Constant | Value | Default style |
|---|---|---|
| `TABLE_HEADER` | `"table.header"` | bold white |
| `TABLE_FOOTER` | `"table.footer"` | bold |
| `TABLE_TITLE` | `"table.title"` | bold |
| `TABLE_CAPTION` | `"table.caption"` | dim |
| `TABLE_BORDER` | `"table.border"` | bright_black |

### Rule

| Constant | Value | Default style |
|---|---|---|
| `RULE_LINE` | `"rule.line"` | bright_black |
| `RULE_TEXT` | `"rule.text"` | bold |

### Tree

| Constant | Value | Default style |
|---|---|---|
| `TREE` | `"tree"` | white |
| `TREE_LINE` | `"tree.line"` | bright_black |

### Progress bars

| Constant | Value | Default style |
|---|---|---|
| `BAR_COMPLETE` | `"bar.complete"` | green |
| `BAR_FINISHED` | `"bar.finished"` | bright_green |
| `BAR_PULSE` | `"bar.pulse"` | bright_cyan |
| `PROGRESS_DESCRIPTION` | `"progress.description"` | white |
| `PROGRESS_PERCENTAGE` | `"progress.percentage"` | cyan |
| `PROGRESS_REMAINING` | `"progress.remaining"` | (inherited) |
| `PROGRESS_ELAPSED` | `"progress.elapsed"` | (inherited) |
| `PROGRESS_DATA` | `"progress.data"` | (inherited) |

### Markdown

| Constant | Value | Default style |
|---|---|---|
| `MARKDOWN_H1` | `"markdown.h1"` | bold bright_cyan |
| `MARKDOWN_H2` | `"markdown.h2"` | bold cyan |
| `MARKDOWN_CODE` | `"markdown.code"` | bright_yellow on black |
| `MARKDOWN_LINK` | `"markdown.link"` | underline bright_blue |
| `MARKDOWN_ITEM` | `"markdown.item"` | (inherited) |
| `MARKDOWN_BLOCKQUOTE` | `"markdown.blockquote"` | (inherited) |

### JSON

| Constant | Value | Default style |
|---|---|---|
| `JSON_KEY` | `"json.key"` | cyan |
| `JSON_STR` | `"json.str"` | green |
| `JSON_NUMBER` | `"json.number"` | bold bright_blue |
| `JSON_BOOL` | `"json.bool"` | bold bright_yellow |
| `JSON_NULL` | `"json.null"` | dim bright_red |
| `JSON_BRACE` | `"json.brace"` | bright_black |

### Traceback

| Constant | Value | Default style |
|---|---|---|
| `TRACEBACK_BORDER` | `"traceback.border"` | red |
| `TRACEBACK_TITLE` | `"traceback.title"` | bold |
| `TRACEBACK_ERROR` | `"traceback.error"` | bold bright_red |
| `TRACEBACK_ERROR_MARK` | `"traceback.error_mark"` | bold bright_red |
| `TRACEBACK_FILENAME` | `"traceback.filename"` | cyan |
| `TRACEBACK_LINE_NO` | `"traceback.line_no"` | bright_black |
| `TRACEBACK_LOCALS_HEADER` | `"traceback.locals_header"` | bold |

### Syntax highlighting

| Constant | Value | Default style |
|---|---|---|
| `SYNTAX_COMMENT` | `"syntax.comment"` | (inherited) |
| `SYNTAX_KEYWORD` | `"syntax.keyword"` | (inherited) |
| `SYNTAX_STRING` | `"syntax.string"` | (inherited) |
| `SYNTAX_NUMBER` | `"syntax.number"` | (inherited) |
| `SYNTAX_FUNCTION` | `"syntax.function"` | (inherited) |
| `SYNTAX_TYPE` | `"syntax.type"` | (inherited) |

### Logging

| Constant | Value | Default style |
|---|---|---|
| `LOGGING_KEYWORD` | `"logging.keyword"` | (inherited) |
| `LOGGING_LEVEL_DEBUG` | `"logging.level.debug"` | (inherited) |
| `LOGGING_LEVEL_INFO` | `"logging.level.info"` | (inherited) |
| `LOGGING_LEVEL_WARNING` | `"logging.level.warning"` | (inherited) |
| `LOGGING_LEVEL_ERROR` | `"logging.level.error"` | (inherited) |
| `LOGGING_LEVEL_CRITICAL` | `"logging.level.critical"` | (inherited) |

Using the constants rather than raw strings protects against typos:

```rust
use rusty_rich::theme::names;

theme.set(names::REPR_NUMBER, Style::from_str("bold cyan"));
// instead of: theme.set("repr.number", ...)
```

---

## Loading a theme from an INI file

`Theme::read()` loads a theme from a simple INI-format file. The file
contains a `[theme]` section with `key = style_definition` pairs:

```ini
[theme]
repr.number = bold cyan
repr.str = green
table.header = bold white
table.border = bright_black
tree = white on color235
markdown.h1 = bold bright_cyan
markdown.code = bright_yellow on black
json.key = cyan
```

```rust
use rusty_rich::Theme;

let theme = Theme::read("my_theme.ini")
    .expect("failed to load theme");
// theme now contains all styles defined in the file
```

The style definition on the right-hand side uses the same syntax as
`Style::from_str()`: foreground colour, optional `on <bgcolour>`, and any
combination of attribute tokens (`bold`, `italic`, `underline`, `dim`,
`reverse`, `strike`, `blink`).

An `inherit` key can point to another theme file:

```ini
[theme]
inherit = base_theme.ini
repr.number = bold yellow
```

When `inherit` is specified, the parent theme is loaded first and its styles
are used as fallbacks for any name not defined in the child.

---

## ThemeStack

`ThemeStack` is a stack of themes. When looking up a style name, themes are
searched from the top (most recently pushed) downward. This lets you layer
a temporary "accent" theme on top of a base theme.

```rust
use rusty_rich::theme::{ThemeStack, Theme, default_theme};

let mut stack = ThemeStack::new();

// Push the default theme as the base
stack.push(default_theme());

// Push an accent theme on top
let mut accent = Theme::new();
accent.set("table.header", Style::from_str("bold bright_yellow"));
stack.push(accent);

// Lookup checks accent first, then falls back to default
let header_style = stack.get("table.header");
// => bold bright_yellow (from accent)

let number_style = stack.get("repr.number");
// => bold cyan (from default_theme, inherited)

stack.pop(); // removes accent
stack.pop(); // removes default
```

### Methods

```rust
let mut stack = ThemeStack::new();

stack.push(theme);               // Push a theme onto the stack
if let Some(t) = stack.pop() {   // Pop and return the top theme
    // ...
}
let style = stack.get("name");   // Look up a style (top-down)
```

---

## Console theme management

Every `Console` holds a single active `Theme` in its `theme` field. You can
set it directly or use the stack-based helpers for temporary changes.

### Setting the theme directly

```rust
use rusty_rich::{Console, Theme, Style};

let mut console = Console::new();

console.theme = Theme::new();  // reset to empty

// Or set a pre-built theme
let mut dark = Theme::new();
dark.set("repr.number", Style::from_str("bold cyan"));
dark.set("repr.str",    Style::from_str("green"));
console.theme = dark;
```

### Temporary theme with push_theme / pop_theme

`push_theme(theme)` wraps the current theme as the `inherit` parent of the new
theme, making the new theme take precedence. `pop_theme()` restores the
previous theme.

```rust
use rusty_rich::{Console, Theme, Style};
use rusty_rich::theme::names;

let mut console = Console::new();

// Temporarily override table styles
let mut accent = Theme::new();
accent.set(names::TABLE_HEADER, Style::from_str("bold bright_yellow"));
accent.set(names::TABLE_BORDER, Style::from_str("bright_yellow"));
accent.set(names::TABLE_TITLE,  Style::from_str("bold bright_yellow"));

// All table output from here uses the accent colours
console.push_theme(accent);

console.print_str("[bold]A table with yellow accents[/bold]");
// ... render tables ...

// Restore the previous theme
console.pop_theme();
```

Because `push_theme` chains via inheritance, you can nest multiple temporary
themes:

```rust
console.push_theme(red_theme);   // outer override
console.push_theme(blue_theme);  // blue takes precedence over red
// ... output with blue-accented styles ...
console.pop_theme();  // back to red
console.pop_theme();  // back to original
```

### get_style

`Console::get_style(name, default)` looks up a style by name from the
console's current theme. If the name is not found and `default` is non-empty,
it parses `default` as a style definition.

```rust
let style = console.get_style("custom.heading", "bold white");
// Returns the theme's "custom.heading" style if defined,
// otherwise "bold white"
```

---

## Inheritance

A theme can inherit from a parent. When `get()` is called, the child's own
`styles` map is checked first. If the name is not found, the lookup
recurses into `inherit`.

```rust
use rusty_rich::{Theme, Style};

let mut base = Theme::new();
base.set("repr.number", Style::from_str("cyan"));
base.set("repr.str",    Style::from_str("green"));

// Create a "dark mode" variant that only overrides a few styles
let mut dark = Theme::with_inherit(base);
dark.set("repr.str", Style::from_str("bright_green"));

assert_eq!(
    dark.get("repr.number").unwrap().to_string(),
    "cyan"          // inherited from base
);
assert_eq!(
    dark.get("repr.str").unwrap().to_string(),
    "bright_green"  // overridden in dark
);
```

You can also set the inherit field directly:

```rust
let mut child = Theme::new();
child.inherit = Some(Box::new(base));
```

---

## Creating custom themes

### Programmatic theme

```rust
use rusty_rich::{Theme, Style, Color};
use rusty_rich::theme::{default_theme, names};

// Start from the default theme and override specific styles
let mut my_theme = default_theme();

my_theme.set(names::REPR_NUMBER, Style::from_str("bold yellow"));
my_theme.set(names::TABLE_HEADER, Style::from_str("bold cyan on blue"));
my_theme.set(names::JSON_KEY, Style::from_str("yellow"));
my_theme.set(names::MARKDOWN_H1, Style::from_str("bold underline yellow"));
my_theme.set(names::RULE_LINE, Style::from_str("blue"));
```

### Theme from an INI file

Create a file `solarized.ini`:

```ini
[theme]
repr.number = bold cyan
repr.str = green
repr.bool_true = bold bright_green
repr.bool_false = bold bright_red
repr.none = dim bright_yellow
repr.url = underline bright_blue
repr.path = magenta

table.header = bold white
table.border = bright_black
table.title = bold

tree = white
tree.line = bright_black

bar.complete = green
bar.finished = bright_green
bar.pulse = bright_cyan

markdown.h1 = bold bright_cyan
markdown.h2 = bold cyan
markdown.code = bright_yellow on black
markdown.link = underline bright_blue

json.key = cyan
json.str = green
json.number = bold bright_blue
json.bool = bold bright_yellow
json.null = dim bright_red
json.brace = bright_black

traceback.border = red
traceback.title = bold
traceback.error = bold bright_red
traceback.filename = cyan
```

Load it:

```rust
use rusty_rich::Theme;
use rusty_rich::theme::default_theme;

// Load from file (uses the file's style definitions)
let file_theme = Theme::read("solarized.ini").unwrap();

// Merge with defaults so uncovered names still have sensible styles
let mut theme = default_theme();
theme.merge(&file_theme);

// Apply to console
console.theme = theme;
```

### Theme with inheritance from a base file

```ini
; dark_solarized.ini
[theme]
inherit = solarized.ini
repr.number = bold bright_yellow
markdown.h1 = bold underline bright_yellow
```

```rust
let theme = Theme::read("dark_solarized.ini").unwrap();
// All styles from solarized.ini are inherited;
// repr.number and markdown.h1 are overridden.
```

---

## Complete example: temporary theme with Console

The following example applies a "high contrast" theme temporarily for a
section of output, then restores the original.

```rust
use rusty_rich::{Console, Theme, Style};
use rusty_rich::theme::{default_theme, names};

fn main() {
    let mut console = Console::new();

    // Start with the default theme
    console.theme = default_theme();

    // Print some default-themed output
    console.print_str("Default theme:\n");
    console.print_str("[repr.number]42[/repr.number] and ");
    console.print_str("[repr.str]hello[/repr.str]\n");

    // Create a high-contrast override theme
    let mut high_contrast = Theme::new();
    high_contrast.set(names::REPR_NUMBER, Style::from_str("bold bright_yellow on blue"));
    high_contrast.set(names::REPR_STR,    Style::from_str("bold white on blue"));
    high_contrast.set(names::TABLE_HEADER, Style::from_str("bold bright_yellow on blue"));
    high_contrast.set(names::TABLE_BORDER, Style::from_str("bright_yellow"));

    // Apply the temporary theme
    console.push_theme(high_contrast);

    console.print_str("High contrast theme:\n");
    console.print_str("[repr.number]42[/repr.number] and ");
    console.print_str("[repr.str]hello[/repr.str]\n");
    // ... render tables, JSON, etc. ...

    // Restore the previous theme
    console.pop_theme();

    console.print_str("Back to default theme:\n");
    console.print_str("[repr.number]42[/repr.number]\n");
}
```

---

## Summary

| Concept | Description |
|---|---|
| `Theme` | A mapping of style names to `Style` values, with optional parent inheritance. |
| `Theme::new()` | Creates an empty theme. |
| `Theme::with_inherit(parent)` | Creates a theme that falls back to `parent`. |
| `Theme::read(path)` | Loads a theme from an INI file. |
| `Theme::get(name)` | Looks up a style by name (recurses into `inherit`). |
| `Theme::set(name, style)` | Adds or overrides a named style. |
| `Theme::merge(&other)` | Merges styles from another theme without overriding. |
| `default_theme()` | Returns the built-in Rich-compatible default theme. |
| `theme::names::*` | String constants for all built-in style names. |
| `ThemeStack` | A LIFO stack of themes for layered style resolution. |
| `Console::push_theme()` | Temporarily replaces the console's theme via inheritance. |
| `Console::pop_theme()` | Restores the previous theme. |
| `Console::get_style(name, default)` | Looks up a style with a fallback definition. |
