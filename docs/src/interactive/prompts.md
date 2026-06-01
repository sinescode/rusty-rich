# Prompts

Interactive prompts let you ask the user for input with styled prompt text, choice
validation, and password masking. They are inspired by the Python Rich `rich.prompt`
module.

## Overview

rusty-rich provides five prompt types:

| Type         | Return value | Description                           |
|--------------|--------------|---------------------------------------|
| `Prompt`     | `String`     | Free-form string input                |
| `IntPrompt`  | `i64`        | Integer input (loops until valid)     |
| `FloatPrompt`| `f64`        | Floating-point input (loops until valid) |
| `Confirm`    | `bool`       | Yes/no answer with a default          |
| `Select<T>`  | `T`          | Numbered selection from a list        |

All prompts share a common base configuration via `PromptBase` and support:
- Styled prompt text using theme styles (`prompt`, `prompt.choices`, `prompt.default`)
- Optional `Console` for output (falls back to raw stdout)
- Password mode (characters masked with `*`)
- Choice validation with optional case sensitivity
- Display of default values and choices

---

## PromptBase

`PromptBase` holds the common configuration for every prompt type. You rarely
use it directly -- each concrete prompt wraps its own `PromptBase` instance and
exposes builder methods that delegate to it.

### Fields

```rust
pub struct PromptBase {
    pub prompt: String,
    pub console: Option<Console>,
    pub password: bool,
    pub choices: Option<Vec<String>>,
    pub case_sensitive: bool,
    pub show_default: bool,
    pub show_choices: bool,
}
```

### Builder Methods

All builder methods consume and return `self`, enabling a fluent chain:

| Method | Signature | Description |
|--------|-----------|-------------|
| `new(prompt)` | `(impl Into<String>) -> Self` | Create with the given prompt text |
| `console(console)` | `(Console) -> Self` | Attach a `Console` for styled output |
| `password(yes)` | `(bool) -> Self` | Enable or disable password masking |
| `choices(choices)` | `(Vec<String>) -> Self` | Set valid response choices |
| `case_sensitive(yes)` | `(bool) -> Self` | Require exact case matching for choices |
| `show_default(yes)` | `(bool) -> Self` | Show or hide the default value in the prompt |
| `show_choices(yes)` | `(bool) -> Self` | Show or hide the choices list in the prompt |

### Helper Methods

- **`render_default(&self, default: &str) -> String`** -- Formats the default
  value for display (e.g. `" (default: Alice)"`). Returns an empty string if
  `show_default` is `false` or the default is empty.

- **`make_prompt(&self) -> String`** -- Builds the full prompt string including
  choices and trailing `": "`. Example output:
  `"Enter choice [a/b/c]: "` (styled with theme styles).

- **`check_choice(&self, value: &str) -> bool`** -- Validates `value` against
  the configured choices. When `choices` is `None`, every value is accepted.
  When `case_sensitive` is `false` (the default), comparison is
  case-insensitive.

---

## Prompt (string input)

`Prompt` reads a free-form string from the user.

```rust
pub struct Prompt {
    base: PromptBase,
}
```

### Builder Methods

`Prompt` exposes all `PromptBase` builder methods directly:
`new`, `console`, `password`, `choices`, `case_sensitive`, `show_default`,
`show_choices`.

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `render()` | `() -> String` | Returns the styled prompt string |
| `ask()` | `() -> Result<String, PromptError>` | Show prompt, read input, validate, return trimmed string |
| `ask_with(prompt)` | `(impl Into<String>) -> Result<String, PromptError>` | Convenience: `Prompt::new(prompt).ask()` |

### Behaviour

1. The prompt string (styled via `make_prompt()`) is written to stdout or the
   attached `Console`.
2. A line is read from stdin. In normal mode, this uses `io::stdin().lock()`.
3. If `password` is `true`, input is read character-by-character in raw mode
   with `*` masking.
4. If choices are configured, the response is validated. On mismatch,
   `Err(PromptError::InvalidResponse(...))` is returned.
5. The trimmed response is returned as `String`.

---

## IntPrompt (integer input)

`IntPrompt` reads an integer (`i64`) from the user. Unlike `Prompt`, it
**loops** on invalid input, printing an error message and re-prompting until a
valid integer (or choice match) is entered.

```rust
pub struct IntPrompt {
    base: PromptBase,
}
```

### Builder Methods

`new`, `console`, `password`, `choices`, `case_sensitive`.

Note: `IntPrompt` does not expose `show_default` or `show_choices` builders.

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `ask()` | `() -> Result<i64, PromptError>` | Show prompt, loop until valid integer |
| `ask_with(prompt)` | `(impl Into<String>) -> Result<i64, PromptError>` | Convenience: `IntPrompt::new(prompt).ask()` |

### Behaviour

1. Displays the prompt and reads input.
2. Empty lines are silently retried.
3. If choices are configured, the response is validated first; invalid choices
   print `"Invalid choice: '...'. Please try again.\n"` and loop.
4. Parses the input as `i64`. On parse failure, prints `"Please enter a valid integer.\n"`
   and loops.
5. Returns `Ok(i64)` on success, or `Err(PromptError::Cancelled)` on EOF/Ctrl+C.

---

## FloatPrompt (float input)

`FloatPrompt` reads a floating-point number (`f64`) from the user. Like
`IntPrompt`, it loops on invalid input.

```rust
pub struct FloatPrompt {
    base: PromptBase,
}
```

### Builder Methods

`new`, `console`, `password`, `choices`, `case_sensitive`.

Note: `FloatPrompt` does not expose `show_default` or `show_choices` builders.

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `ask()` | `() -> Result<f64, PromptError>` | Show prompt, loop until valid float |
| `ask_with(prompt)` | `(impl Into<String>) -> Result<f64, PromptError>` | Convenience: `FloatPrompt::new(prompt).ask()` |

### Behaviour

Same loop semantics as `IntPrompt` but parses as `f64` and prints
`"Please enter a valid number.\n"` on parse failure.

---

## Confirm (yes/no)

`Confirm` asks for a yes/no answer and returns `bool`. It carries a `default`
value used when the user presses Enter without typing.

```rust
pub struct Confirm {
    base: PromptBase,
    pub default: bool,
}
```

### Builder Methods

`new(prompt, default)`, `console`.

`Confirm` does not expose general `PromptBase` builders (`password`, `choices`,
`case_sensitive`, `show_default`, `show_choices`).

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `ask()` | `() -> Result<bool, PromptError>` | Show `[y/N]` or `[Y/n]` prompt, return bool |
| `ask_with(prompt, default)` | `(impl Into<String>, bool) -> Result<bool, PromptError>` | Convenience: `Confirm::new(prompt, default).ask()` |

### Accepted Inputs

| Input | Interpretation |
|-------|---------------|
| (empty) | Returns the configured default |
| `y`, `yes`, `true`, `1` | `true` (affirmative) |
| `n`, `no`, `false`, `0` | `false` (negative) |

On unrecognised input, `"Please answer y or n.\n"` is printed and the prompt
repeats.

### Prompt Display

The confirmation prompt shows `[Y/n]` when the default is `true` and `[y/N]`
when the default is `false` (the capital letter indicates the default).

---

## Select\<T\> (numbered menu)

`Select<T>` presents a numbered list of choices and returns the value
associated with the chosen entry.

```rust
pub struct Select<T> {
    base: PromptBase,
    choices: Vec<(String, T)>,
}
```

### Builder Methods

`new(prompt)`, `console`, `choice(label, value)`.

The `choice` method adds a `(label, T)` pair to the internal list. Each label
is displayed as a numbered item.

### Methods

| Method | Requirements | Description |
|--------|-------------|-------------|
| `render()` | `T: Display` | Render the numbered list + prompt as a `String` |
| `ask()` | `T: Display + Clone` | Show menu, read numbered choice, return the selected value |

### Behaviour

1. If no choices have been added, `ask()` returns
   `Err(PromptError::InvalidResponse("no choices available"))`.
2. The prompt is rendered as a multi-line string:
   ```text
   Pick a color:
     1) Red
     2) Green
     3) Blue
   Enter number [1-3]:
   ```
3. Input is parsed as a `usize`. Numbers outside the valid range (or
   non-numeric input) print `"Please enter a number between 1 and N.\n"` and
   loop.
4. On valid input, the `T` value at `choices[n - 1]` is cloned and returned.

---

## Password Masking

When `password(true)` is set on `Prompt`, `IntPrompt`, or `FloatPrompt`, input
is read with echoing disabled via crossterm raw mode. Each typed character
is echoed as `*`. Backspace erases the last character and removes one `*` from
the display. Escape or Delete cancels the prompt (`PromptError::Cancelled`).
Enter accepts the input and writes a newline.

The password reader is implemented directly in `read_password()` and does not
depend on the `rpassword` crate.

Note: `Confirm` and `Select<T>` do not support password mode.

---

## Choice Validation

When `choices` is set on `Prompt`, `IntPrompt`, or `FloatPrompt`, the user's
response is validated against the list.

- With `case_sensitive(false)` (the default), comparison is case-insensitive:
  `"YES"` matches `"yes"`.
- With `case_sensitive(true)`, comparison is exact, including case.
- When no choices are configured, every value is accepted.

For `Prompt`, an invalid choice returns `Err(PromptError::InvalidResponse(...))`.
For `IntPrompt` and `FloatPrompt`, an invalid choice prints an error message and
re-prompts.

---

## PromptError

Errors during prompting are represented by the `PromptError` enum.

```rust
pub enum PromptError {
    InvalidResponse(String),   // User input failed validation
    IOError(io::Error),        // Underlying I/O failure
    Cancelled,                 // EOF, Ctrl+C, or Ctrl+D
}
```

`PromptError` implements `std::error::Error` (with `source()` returning the
inner `io::Error` for the `IOError` variant), `Display`, `From<io::Error>`,
and `Debug`.

---

## Error Handling Patterns

Handle errors at the prompt site to distinguish cancellation from validation
failures:

```rust
use rusty_rich::{Prompt, PromptError};

fn get_username() -> Option<String> {
    match Prompt::ask_with("Enter username") {
        Ok(name) => Some(name),
        Err(PromptError::Cancelled) => {
            eprintln!("Input cancelled.");
            None
        }
        Err(PromptError::InvalidResponse(msg)) => {
            eprintln!("Invalid: {msg}");
            None
        }
        Err(PromptError::IOError(e)) => {
            eprintln!("I/O error: {e}");
            None
        }
    }
}
```

---

## Examples

### Login form (string prompt with password)

```rust
use rusty_rich::{Prompt, Confirm};

fn login() -> Result<(), Box<dyn std::error::Error>> {
    let username = Prompt::new("Username").ask()?;
    let password = Prompt::new("Password").password(true).ask()?;

    println!("Logged in as: {username}");

    if Confirm::ask_with("Save credentials?", false)? {
        println!("Credentials saved.");
    }

    Ok(())
}
```

This example demonstrates:
- `Prompt` for free-form string input, chained with `.password(true)` for
  masked input.
- `Confirm` with a default of `false` (shown as `[y/N]`).

### Numbered menu selection

```rust
use rusty_rich::Select;

#[derive(Debug, Clone)]
enum Action {
    View,
    Edit,
    Delete,
    Quit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let action = Select::new("Choose an action")
        .choice("View entry", Action::View)
        .choice("Edit entry", Action::Edit)
        .choice("Delete entry", Action::Delete)
        .choice("Quit", Action::Quit)
        .ask()?;

    println!("Selected: {:?}", action);
    Ok(())
}
```

This example shows `Select<T>` with a custom `enum` as the value type. Each
`choice()` call adds a labelled entry; the selected entry's value is returned
by `ask()`.

### Confirmation prompt

```rust
use rusty_rich::Confirm;

fn destructive_action() -> Result<(), Box<dyn std::error::Error>> {
    if Confirm::ask_with("Delete all data?", false)? {
        println!("Deleting all data...");
        // ...
    } else {
        println!("Cancelled.");
    }
    Ok(())
}
```

### Integer and float prompts with choices

```rust
use rusty_rich::{IntPrompt, FloatPrompt};

fn get_rating() -> Result<i64, Box<dyn std::error::Error>> {
    let rating = IntPrompt::new("Rating (1-5)")
        .choices(vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()])
        .case_sensitive(true)
        .ask()?;
    Ok(rating)
}

fn get_temperature() -> Result<f64, Box<dyn std::error::Error>> {
    let temp = FloatPrompt::ask_with("Enter temperature in Celsius")?;
    Ok(temp)
}
```

### Full prompt with styled console

```rust
use rusty_rich::{Console, Prompt};

fn styled_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let console = Console::new();
    let answer = Prompt::new("Enter your name")
        .console(console)
        .show_default(false)
        .ask()?;
    println!("Hello, {answer}!");
    Ok(())
}
```

---

## Builder Pattern Summary

All prompt types follow the same fluent builder pattern:

```rust
let result = Type::new("prompt text")
    .console(some_console)   // optional
    .password(true)          // optional
    .choices(vec![...])      // optional
    .case_sensitive(true)    // optional
    .show_default(false)     // optional (Prompt only)
    .show_choices(false)     // optional (Prompt only)
    .ask()?;                 // or .ask_with(...) for convenience
```

| Builder | Prompt | IntPrompt | FloatPrompt | Confirm | Select\<T\> |
|---------|--------|-----------|-------------|---------|-------------|
| `console` | Yes | Yes | Yes | Yes | Yes |
| `password` | Yes | Yes | Yes | No | No |
| `choices` | Yes | Yes | Yes | No | Via `choice()` |
| `case_sensitive` | Yes | Yes | Yes | No | No |
| `show_default` | Yes | No | No | No | No |
| `show_choices` | Yes | No | No | No | No |
