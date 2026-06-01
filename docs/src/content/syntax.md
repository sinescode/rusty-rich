# Syntax Highlighting

`Syntax` renders source code with full syntax highlighting powered by [`syntect`](https://github.com/trishume/syntect) (a Rust syntax-highlighting library with hundreds of grammars — the Rust equivalent of Pygments).

```rust
use rusty_rich::Syntax;

let highlighted = Syntax::new("fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}", "rust");
console.println(&highlighted);
```

---

## Syntax::new(code, lexer_name)

Creates a new `Syntax` renderable for the given source code and language.

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `impl Into<String>` | The source code to highlight (may contain newlines). |
| `lexer_name` | `impl Into<String>` | Language / lexer name (e.g. `"rust"`, `"python"`, `"json"`). Also accepts file extensions such as `"rs"`, `"py"`, `"js"`. |

```rust
use rusty_rich::Syntax;

// By language name
let rust = Syntax::new("fn main() {}", "rust");
let python = Syntax::new("print('hello')", "python");

// By file extension
let js = Syntax::new("console.log('hi');", "js");
let toml = Syntax::new("[package]\nname = \"app\"", "toml");
```

If the language is not recognised (or is an empty string), the code is rendered as plain text without highlighting.

---

## Default configuration

When constructed with `Syntax::new(code, language)`, the defaults are:

| Property | Default |
|----------|---------|
| `theme` | `"base16-ocean.dark"` |
| `line_numbers` | `false` |
| `start_line` | `1` |
| `highlight` | `true` |
| `background_color` | `None` |
| `tab_size` | `4` |

---

## Supported languages

Because the library delegates to `syntect`, hundreds of languages and grammars are available out of the box. Some of the most commonly used:

| Language    | Name string     | Extensions        |
|-------------|-----------------|-------------------|
| Rust        | `"rust"`        | `rs`              |
| Python      | `"python"`      | `py`              |
| JavaScript  | `"javascript"`  | `js`, `mjs`       |
| TypeScript  | `"typescript"`  | `ts`, `tsx`       |
| JSON        | `"json"`        | `json`            |
| YAML        | `"yaml"`        | `yaml`, `yml`     |
| TOML        | `"toml"`        | `toml`            |
| Markdown    | `"markdown"`    | `md`              |
| HTML        | `"html"`        | `html`, `htm`     |
| CSS         | `"css"`         | `css`             |
| Shell       | `"shell"`       | `sh`, `bash`, `zsh` |
| Go          | `"go"`          | `go`              |
| Java        | `"java"`        | `java`            |
| C           | `"c"`           | `c`, `h`          |
| C++         | `"cpp"`         | `cpp`, `hpp`, `cc` |
| SQL         | `"sql"`         | `sql`             |
| Ruby        | `"ruby"`        | `rb`              |
| Elixir      | `"elixir"`      | `ex`, `exs`       |
| Diff        | `"diff"`        | `diff`, `patch`   |

When a string is passed as the language, `Syntax` first tries to match it as a language name, then as a file extension. If neither matches, plain text is used.

You can list all available grammars at runtime via `syntect::parsing::SyntaxSet::load_defaults_newlines()`.

---

## Theme selection

### theme()

Sets the syntect theme used for token colours and styles.

```rust
use rusty_rich::Syntax;

let code = Syntax::new("console.log('hello');", "javascript")
    .theme("base16-ocean.dark");
```

The default theme is `"base16-ocean.dark"`. Other built-in themes include:

- `"base16-ocean.dark"` (default)
- `"base16-ocean.light"`
- `"base16-eighties.dark"`
- `"base16-mocha.dark"`
- `"base16-oceanicnext.dark"`
- `"Solarized (dark)"`
- `"Solarized (light)"`
- `"InspiredGitHub"`
- `"base16-tomorrow.dark"`
- `"base16-tomorrow.light"`

Themes are loaded from `syntect::highlighting::ThemeSet::load_defaults()`, which bundles the `base16` and `sublime` theme collections. You may also load custom `.tmTheme` or `.sublime-color-scheme` files by constructing a custom `ThemeSet`.

```rust
use rusty_rich::Syntax;
use syntect::highlighting::ThemeSet;

let mut custom_themes = ThemeSet::load_from_folder("/path/to/themes/").unwrap();
// Usage would require a custom Syntax implementation or direct syntect integration.
```

---

## Line numbers

### line_numbers()

Enables line number rendering in the left gutter. Line numbers are right-aligned and separated from the code by a vertical bar.

```rust
use rusty_rich::Syntax;

let code = Syntax::new("fn main() {\n    println!(\"hi\");\n}", "rust")
    .line_numbers();
```

Output (conceptually):

```
1 │ fn main() {
2 │     println!("hi");
3 │ }
```

Line numbers are formatted with enough width to accommodate the highest line count, ensuring alignment.

---

## Start line

### start_line(n)

Sets the line number for the first line. Only meaningful when `line_numbers` is enabled.

```rust
use rusty_rich::Syntax;

let code = Syntax::new(
    "import sys\n\ndef main():\n    return 0\n",
    "python",
)
    .line_numbers()
    .start_line(10);
```

Renders with line numbering starting at 10:

```
10 │ import sys
11 │
12 │ def main():
13 │     return 0
```

The default is `1`.

---

## Highlight lines

### highlight_lines(line_numbers)

Highlights specific lines by applying a background tint or emphasis style, drawing attention to relevant parts of the code.

```rust
use rusty_rich::Syntax;

let code = Syntax::new(
    "def fib(n):\n    if n <= 1:\n        return n\n    return fib(n - 1) + fib(n - 2)",
    "python",
)
    .line_numbers()
    .highlight_lines(&[2, 4]);
```

The lines specified receive a distinct background colour to make them stand out from the rest of the code block.

---

## Background colour

### background(color)

Sets a background colour for the entire code block.

```rust
use rusty_rich::Syntax;
use rusty_rich::Color;

let code = Syntax::new("const x: i32 = 42;", "rust")
    .background(Color::parse("#1e1e2e").unwrap());
```

When set, every character cell in the rendered output receives the specified background colour. If `None` (the default), the background is inherited from the console or terminal.

---

## Tab size

### tab_size

Controls how many spaces a tab character expands to. The default is `4`.

```rust
use rusty_rich::Syntax;

let code = Syntax::new(
    "def foo():\n\treturn 42",
    "python",
);
// tab_size defaults to 4

// Change to 2
code.tab_size = 2;
```

The `tab_size` field is a public `usize` field on the struct and can be set directly after construction.

---

## Word wrap

### word_wrap

Controls whether long lines are wrapped at the available terminal width. When enabled, lines that exceed the console width are broken at word boundaries onto subsequent lines. Line numbers are repeated for continuation lines so they remain scannable.

```rust
use rusty_rich::Syntax;

let code = Syntax::new(long_snippet, "rust");
code.word_wrap = true;
```

When `word_wrap` is `false` (the default), long lines are truncated or cropped according to the console's overflow behaviour (typically `OverflowMethod::Ellipsis`).

---

## Padding

### padding

Controls the space (in characters) between the code block edges and the surrounding container. Specified as a 4-tuple `(top, right, bottom, left)`.

```rust
use rusty_rich::Syntax;

let code = Syntax::new("print('hello')", "python");
code.padding = (0, 2, 0, 2);  // 2 characters left and right
```

The default padding is `(0, 0, 0, 0)`. Padding is particularly useful when the `Syntax` block is placed inside a `Panel` or when a `background_color` is set, to prevent text from touching the edges.

---

## Fields reference

The `Syntax` struct exposes all configuration as public fields, so you can inspect or modify them directly:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `code` | `String` | (required) | The source code to highlight. |
| `language` | `String` | (required) | Language/lexer name or file extension. |
| `theme` | `String` | `"base16-ocean.dark"` | Syntect theme name. |
| `start_line` | `usize` | `1` | First line number (when `line_numbers` is true). |
| `line_numbers` | `bool` | `false` | Show line numbers in the gutter. |
| `highlight` | `bool` | `true` | Enable syntax highlighting (set to `false` for plain text rendering). |
| `background_color` | `Option<Color>` | `None` | Background colour for the entire block. |
| `tab_size` | `usize` | `4` | Tab expansion width. |
| `word_wrap` | `bool` | `false` | Wrap long lines at word boundaries. |
| `padding` | `(usize, usize, usize, usize)` | `(0, 0, 0, 0)` | Padding `(top, right, bottom, left)`. |
| `highlight_lines` | `Option<Vec<usize>>` | `None` | 1-based line numbers to emphasise. |

---

## Builder methods

| Method | Modifies | Description |
|--------|----------|-------------|
| `.theme(s)` | `theme` | Select a syntect theme. |
| `.line_numbers()` | `line_numbers` | Enable line numbers. |
| `.start_line(n)` | `start_line` | Set the starting line number. |
| `.background(c)` | `background_color` | Set a background colour. |

Fields that do not have a dedicated builder method can be set directly on the struct:

```rust
let mut code = Syntax::new("let x = 1;", "rust");
code.tab_size = 2;
code.word_wrap = true;
code.highlight_lines = Some(vec![1, 3]);
code.padding = (1, 2, 1, 2);
```

---

## Plain text mode

Set `highlight` to `false` (or pass an empty / unrecognised language) to render the code without any syntax colouring. Line numbers and background colour are still applied.

```rust
use rusty_rich::Syntax;

let mut plain = Syntax::new("raw log output\nline 2", "");
plain.highlight = false;
plain.line_numbers = true;
```

---

## Complete examples

### Python

```rust
use rusty_rich::Syntax;
use rusty_rich::Color;

let code = Syntax::new(
    "\"\"\"Calculate fibonacci numbers.\"\"\"\n\
     \n\
     def fib(n: int) -> int:\n\
         \"\"\"Return the nth fibonacci number.\"\"\"\n\
         if n <= 1:\n\
             return n\n\
         return fib(n - 1) + fib(n - 2)\n\
     \n\
     \n\
     def main() -> None:\n\
         result = fib(10)\n\
         print(f\"fib(10) = {result}\")\n\
     \n\
     \n\
     if __name__ == \"__main__\":\n\
         main()",
    "python",
)
    .theme("base16-eighties.dark")
    .line_numbers()
    .start_line(1)
    .background(Color::parse("#2d2d2d").unwrap());
```

This renders the Python code with `base16-eighties.dark` theming, line numbers starting at 1, and a dark background.

### Rust

```rust
use rusty_rich::Syntax;
use rusty_rich::Color;

let code = Syntax::new(
    "use std::collections::HashMap;\n\
     \n\
     /// A simple key-value store.\n\
     struct Store {\n\
         inner: HashMap<String, String>,\n\
     }\n\
     \n\
     impl Store {\n\
         fn new() -> Self {\n\
             Self {\n\
                 inner: HashMap::new(),\n\
             }\n\
         }\n\
     \n\
         fn get(&self, key: &str) -> Option<&str> {\n\
             self.inner.get(key).map(|s| s.as_str())\n\
         }\n\
     \n\
         fn set(&mut self, key: String, value: String) {\n\
             self.inner.insert(key, value);\n\
         }\n\
     }\n\
     \n\
     fn main() {\n\
         let mut store = Store::new();\n\
         store.set(\"host\".into(), \"localhost\".into());\n\
         println!(\"{}\", store.get(\"host\").unwrap());\n\
     }",
    "rust",
)
    .theme("base16-ocean.dark")
    .line_numbers()
    .start_line(1);
```

### JSON

```rust
use rusty_rich::Syntax;
use rusty_rich::Color;

let code = Syntax::new(
    "{\n\
     \"name\": \"rusty-rich\",\n\
     \"version\": \"0.1.0\",\n\
     \"description\": \"Rich text and beautiful formatting in the terminal\",\n\
     \"license\": \"MIT\",\n\
     \"keywords\": [\n\
         \"terminal\",\n\
         \"tui\",\n\
         \"formatting\"\n\
     ],\n\
     \"dependencies\": {\n\
         \"syntect\": \"5.1\",\n\
         \"serde\": { \"features\": [\"derive\"] }\n\
     }\n\
 }",
    "json",
)
    .theme("Solarized (dark)")
    .line_numbers()
    .start_line(1)
    .background(Color::parse("#073642").unwrap());
```

### Minimal inline usage

```rust
use rusty_rich::Syntax;

// Inline one-liner — no line numbers, default theme
let snippet = Syntax::new("git status", "shell");
console.println(&snippet);
```

### With line highlighting

```rust
use rusty_rich::Syntax;

let code = Syntax::new(
    "fn divide(a: f64, b: f64) -> f64 {\n\
     if b == 0.0 {\n\
     panic!(\"division by zero\");\n\
     }\n\
     a / b\n\
     }",
    "rust",
)
    .line_numbers()
    .highlight_lines(&[2, 3]);
```

The second and third lines are visually emphasised to draw the reader's attention to the error-handling branch.
