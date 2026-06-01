# JSON

`JsonRender` pretty-prints JSON values with syntax highlighting. It uses `serde_json` for formatting and supports configurable indentation and theme overrides.

```rust
use rusty_rich::{render_json, JsonRender, Console};
use serde_json::json;

let mut console = Console::new();

let data = json!({
    "name": "rusty-rich",
    "version": "0.1.0",
    "active": true,
    "contributors": null,
});

let rendered = render_json(&data);
console.println(&rendered);
```

---

## render_json(value)

```rust
pub fn render_json(value: &Value) -> JsonRender
```

The entry point. Takes a `serde_json::Value` and returns a `JsonRender` builder with default settings:

- Indent: 2 spaces
- Theme: the default theme colors (see [Theme Styles](#theme-styles))
- Syntax highlighting: enabled

```rust
use serde_json::json;

let data = json!({"hello": "world"});
let rendered = render_json(&data);
console.println(&rendered);
```

---

## JsonRender

The builder struct returned by `render_json()`. Configures rendering options before display.

### indent()

Set the number of spaces per indentation level.

```rust
let rendered = render_json(&data)
    .indent(4);
```

The default is 2. A larger indent pushes nested values further to the right. A value of 0 collapses the output to minimal whitespace (every value on its own line, flush left).

### theme()

Override the theme used for syntax highlighting styles.

```rust
use rusty_rich::{Theme, Style, Color};

let mut theme = Theme::new();
theme.set(
    "json.key",
    Style::new().color(Color::parse("magenta").unwrap()),
);
theme.set(
    "json.str",
    Style::new().color(Color::parse("green").unwrap()),
);

let rendered = render_json(&data)
    .theme(theme)
    .indent(4);
```

When no theme is provided, the console's default theme is used (or the library-wide default if `JsonRender` is rendered directly without a console context).

### highlight (disable via render_json)

Syntax highlighting is enabled by default. To disable it and produce plain ANSI-free output, you can construct a `JsonRender` directly:

```rust
use rusty_rich::json::JsonRender;

let plain = JsonRender {
    value: data.clone(),
    indent: 2,
    highlight: false,
    ..JsonRender::new(&data)
};
```

When highlighting is disabled, the output is equivalent to `serde_json::to_string_pretty(value)` with no ANSI escape codes.

---

## Syntax Highlighting

Each JSON value type is styled with a distinct theme key. The default theme styles are:

| Style Key      | Default Style                         | Applies To           |
|----------------|---------------------------------------|----------------------|
| `json.key`     | cyan                                  | Object keys          |
| `json.str`     | green                                 | String values        |
| `json.number`  | bright blue + bold                    | Number values        |
| `json.bool`    | bright yellow + bold                  | `true` / `false`     |
| `json.null`    | bright red + dim                      | `null`               |
| `json.brace`   | bright black                          | `{` `}` `[` `]`      |

### Customizing styles

Override any style via the `Theme` system:

```rust
use rusty_rich::{Theme, Style, Color};
use rusty_rich::theme::names;

let mut theme = Theme::new();

theme.set(
    names::JSON_KEY,
    Style::new().bold(true).color(Color::parse("yellow").unwrap()),
);
theme.set(
    names::JSON_NUMBER,
    Style::new().color(Color::parse("cyan").unwrap()),
);

let rendered = render_json(&data).theme(theme);
console.println(&rendered);
```

The theme uses ANSI escape codes applied inline per token, so even when `JsonRender` is composed inside a `Panel`, `Columns`, or `Layout`, the syntax highlighting is preserved.

---

## serde_json Integration

`JsonRender` works with any `serde_json::Value`. Construct values using the `serde_json::json!` macro or parse from a string:

```rust
use serde_json::{json, Value};

// Using the json! macro
let data = json!({
    "name": "rusty-rich",
    "version": "0.1.0",
    "features": ["tables", "trees", "markdown"]
});

// Parsing from a string
let text = r#"{"name": "rusty-rich", "version": "0.1.0"}"#;
let parsed: Value = serde_json::from_str(text).unwrap();

let rendered = render_json(&parsed);
console.println(&rendered);
```

---

## Console Convenience

The `Console` type provides a direct `print_json()` method:

```rust
console.print_json(&data);
```

This is shorthand for:

```rust
let rendered = render_json(&data);
console.println(&rendered);
```

There is also a free function `print_json_val()` that uses the global console:

```rust
use rusty_rich::print_json;

print_json(&data);
```

### Import paths

```rust
use rusty_rich::render_json;          // Builder function
use rusty_rich::json::JsonRender;     // The builder struct (if direct construction is needed)
use rusty_rich::Theme;                // For customising JSON styles
use rusty_rich::Style;                // For constructing style overrides
use rusty_rich::Color;                // For color values in custom styles
use rusty_rich::theme::names;         // For theme key constants (JSON_KEY, JSON_STR, etc.)
use rusty_rich::print_json;           // Convenience: print using global console
```

---

## Example: Pretty-Printing an API Response

The following example fetches JSON from a public API and renders it with syntax highlighting:

```rust
use rusty_rich::{render_json, Panel, Console};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();

    // Fetch a public API (replace with any JSON endpoint)
    let response = ureq::get("https://api.github.com/repos/rust-lang/rust")
        .call()?
        .into_string()?;

    // Parse JSON
    let data: Value = serde_json::from_str(&response)?;

    // Render with 4-space indentation inside a bordered panel
    let json_render = render_json(&data).indent(4);

    let panel = Panel::new(json_render)
        .title(" GitHub API Response ")
        .border_style(
            Style::new().color(Color::parse("bright_black").unwrap()),
        )
        .padding(1, 2, 1, 2);

    console.println(&panel);

    Ok(())
}
```

The output shows the API response with colored keys, strings, numbers, booleans, and nulls, wrapped in a panel:

```
┌─ GitHub API Response ─────────────────┐
│ {                                     │
│     "id": 23323456,                   │
│     "node_id": "MDEwOlJlcG9...",     │
│     "name": "rust",                    │
│     "full_name": "rust-lang/rust",     │
│     "private": false,                  │
│     "fork": false,                     │
│     "size": 314159,                    │
│     "language": "Rust",                │
│     "archived": false,                 │
│     "disabled": false,                 │
│     "license": {                       │
│         "key": "MIT",                  │
│         "name": "MIT License",         │
│         "spdx_id": "MIT",              │
│         "url": "https://api..."        │
│     },                                 │
│     "topics": [                        │
│         "rust",                        │
│         "compiler",                    │
│         "language"                     │
│     ],                                 │
│     "open_issues_count": 832,          │
│     "default_branch": "master",        │
│     "score": null                      │
│ }                                      │
└────────────────────────────────────────┘
```

Keys appear in cyan, strings in green, numbers in bright blue and bold, booleans in bright yellow and bold, and `null` in dim bright red.

---

## Composing with other renderables

Because `JsonRender` implements the `Renderable` trait, it can be nested inside other components:

```rust
use rusty_rich::{render_json, Panel, Columns, Console};
use serde_json::json;

let mut console = Console::new();

let left = json!({"status": "ok", "code": 200});
let right = json!({"error": null, "message": "success"});

let mut cols = Columns::new();
cols.add(Panel::new(render_json(&left)).title(" Response A "));
cols.add(Panel::new(render_json(&right)).title(" Response B "));

console.println(&cols);
```

The two JSON responses are rendered side-by-side, each inside its own panel, with full syntax highlighting preserved.
