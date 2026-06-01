# Rule

`Rule` renders a horizontal rule (divider) across the terminal width, optionally with a title embedded in the line. It is useful for separating sections in terminal output, creating visual headings, or dividing log output.

```rust
use rusty_rich::Rule;

let rule = Rule::new();
console.println(&rule);
```

```
────────────────────────────────────────
```

---

## new()

Creates a plain horizontal rule with no title, using the default character (`─`).

```rust
use rusty_rich::Rule;

let rule = Rule::new();
console.println(&rule);
```

Default configuration:

| Field      | Type         | Default            |
|------------|--------------|--------------------|
| `title`    | `String`     | `""` (empty)       |
| `characters` | `String`  | `"─"` (U+2500)     |
| `style`    | `Style`      | plain (no style)   |
| `end`      | `String`     | `"\n"`             |
| `align`    | `AlignMethod` | `Center`          |

---

## title()

Sets an optional title displayed in the middle of the rule line. The title is surrounded by spaces and the rule character extends on both sides.

```rust
use rusty_rich::Rule;

let rule = Rule::new().title("Section 1");
console.println(&rule);
```

```
─────────────────── Section 1 ───────────────────
```

A plain (untitled) rule can be created by omitting `title()` or passing an empty string:

```rust
use rusty_rich::Rule;

// These two are equivalent:
let plain1 = Rule::new();
let plain2 = Rule::new().title("");
console.println(&plain1);
```

```
────────────────────────────────────────
```

When the title is too long to fit within the available width (including the required spacing), the rule falls back to rendering a plain line with no title text.

---

## characters()

Changes the character (or string) used to draw the rule line. The default is `"─"` (U+2500, light horizontal line). Any string can be used, including multi-character sequences.

```rust
use rusty_rich::Rule;

// Dashes
let rule = Rule::new()
    .title("Dashed Rule")
    .characters("-");
console.println(&rule);
```

```
────────────────── Dashed Rule ──────────────────
```

```rust
// Equals signs
let rule = Rule::new()
    .title("Important")
    .characters("=");
console.println(&rule);
```

```
═══════════════════ Important ═══════════════════
```

```rust
// Multi-character pattern
let rule = Rule::new().characters("~=");
console.println(&rule);
```

```
~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=~=
```

```rust
// Stars
let rule = Rule::new()
    .title("Chapter 1")
    .characters("★");
console.println(&rule);
```

```
★★★★★★★★★★★★★★★ Chapter 1 ★★★★★★★★★★★★★★★★★
```

### ASCII fallback

When the terminal's `ascii_only` mode is active and the `characters` string contains non-ASCII glyphs, the rule automatically falls back to `"-"` (ASCII hyphen). This ensures compatibility with terminals that do not support Unicode.

```rust
// Falls back to "-" in ascii_only mode
let rule = Rule::new().characters("─");
```

---

## style()

Applies a `Style` to the rule line and the title text. When styling is applied, ANSI escape codes are emitted around the rendered line.

```rust
use rusty_rich::{Rule, Style, Color};

let rule = Rule::new()
    .title("Styled Rule")
    .style(Style::new().color(Color::parse("cyan").unwrap()));
console.println(&rule);
```

```
─────────────────── Styled Rule ───────────────────
```
(The line above renders in cyan on supported terminals.)

```rust
use rusty_rich::{Rule, Style, Color};

let rule = Rule::new()
    .title("Warning")
    .characters("=")
    .style(Style::new().color(Color::parse("yellow").unwrap()).bold(true));
console.println(&rule);
```

```
═══════════════════ Warning ═══════════════════════
```
(Renders in bold yellow on supported terminals.)

---

## align()

Controls where the title is placed within the rule line. Accepts an `AlignMethod` value.

| Variant   | Behavior                                      |
|-----------|-----------------------------------------------|
| `Center`  | Title is centered on the line (default)       |
| `Left`    | Title is flush against the left side          |
| `Right`   | Title is flush against the right side         |
| `Full`    | No title is shown; the rule fills the full width (same as a plain rule) |

```rust
use rusty_rich::{Rule, AlignMethod};

// Center-aligned (default)
let center = Rule::new().title("Center").align(AlignMethod::Center);

// Left-aligned
let left = Rule::new().title("Left").align(AlignMethod::Left);

// Right-aligned
let right = Rule::new().title("Right").align(AlignMethod::Right);

console.println(&center);
console.println(&left);
console.println(&right);
```

```
─────────────────── Center ───────────────────
Left ─────────────────────────────────────────
───────────────────────────────────────── Right
```

Note that alignment only affects title placement. A `Full` alignment renders a plain rule line, ignoring any title text:

```rust
use rusty_rich::{Rule, AlignMethod};

let full = Rule::new()
    .title("Ignored Title")
    .align(AlignMethod::Full);
console.println(&full);
```

```
────────────────────────────────────────
```

---

## end

The `end` field is a public string appended after the rendered rule. The default is `"\n"`, which ensures the rule is followed by a newline. Set it to an empty string to suppress the trailing newline, or to any custom suffix.

```rust
use rusty_rich::Rule;

let mut rule = Rule::new();
rule.end = "";  // no trailing newline
console.print(&rule);
```

Unlike builder methods, `end` is a public field that you assign directly. There is no builder method for it.

---

## Full examples

### Plain horizontal rule

```rust
use rusty_rich::Rule;

let rule = Rule::new();
console.println(&rule);
```

```
────────────────────────────────────────
```

### Rule with centered title

```rust
use rusty_rich::Rule;

let rule = Rule::new().title("Overview");
console.println(&rule);
```

```
─────────────────── Overview ───────────────────
```

### Rule with left-aligned title and custom characters

```rust
use rusty_rich::{Rule, AlignMethod};

let rule = Rule::new()
    .title("Details")
    .characters("·")
    .align(AlignMethod::Left);
console.println(&rule);
```

```
Details ·········································
```

### Rule with right-aligned title, styled

```rust
use rusty_rich::{Rule, Style, Color, AlignMethod};

let rule = Rule::new()
    .title("END")
    .characters("─")
    .style(Style::new().color(Color::parse("bright_black").unwrap()))
    .align(AlignMethod::Right);
console.println(&rule);
```

```
──────────────────────────────────────────── END
```

### Multiple rules as section dividers

```rust
use rusty_rich::{Rule, Style, Color, AlignMethod};

// Section header
console.println(&Rule::new()
    .title("Installation")
    .characters("=")
    .style(Style::new().bold(true))
);

// ... installation steps ...

// Another section
console.println(&Rule::new()
    .title("Configuration")
    .characters("=")
    .style(Style::new().bold(true))
);

// End marker
console.println(&Rule::new()
    .title("Done")
    .characters("~")
    .align(AlignMethod::Right)
);
```

```
═════════════════ Installation ═════════════════

═════════════════ Configuration ════════════════

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Done
```

### ASCII-safe rule

```rust
use rusty_rich::Rule;

// Falls back to ASCII "-" in ascii_only mode
let rule = Rule::new()
    .title("Note")
    .characters("-");
console.println(&rule);
```

```
──────────────────── Note ──────────────────────
```

---

## Builder Method Reference

| Method             | Description                                              |
|--------------------|----------------------------------------------------------|
| `.title("...")`    | Sets the title text displayed in the rule line           |
| `.characters("...")` | Sets the character(s) used to draw the line (default `"─"`) |
| `.style(Style)`    | Applies a style to the line and title text               |
| `.align(AlignMethod)` | Sets title alignment: `Left`, `Center` (default), `Right`, `Full` |

**Public field (not a builder method):**

| Field  | Type     | Default | Description                                          |
|--------|----------|---------|------------------------------------------------------|
| `end`  | `String` | `"\n"`  | Text appended after the rendered rule line            |

---

## Import Paths

```rust
use rusty_rich::Rule;            // The Rule type
use rusty_rich::AlignMethod;     // Title alignment
use rusty_rich::Style;           // Styling the rule
use rusty_rich::Color;           // Color values for styles
```
