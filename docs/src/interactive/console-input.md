# Console Input

The `Console` provides a straightforward method for reading user input from stdin:
[`Console::input()`](../api/console.md#input). It writes a prompt string to the
console output and then reads a line of input, optionally masking characters for
password entry.

---

## `Console::input(prompt, password)`

### Signature

```rust
pub fn input(&mut self, prompt: &str, password: bool) -> String
```

- **`prompt`** -- A string written verbatim to the console before reading input.
  The prompt is not styled automatically; use ANSI escape codes or markup via
  `print_str()` before calling `input()` if you need a styled prompt.
- **`password`** -- When `true`, input is masked with `*` characters and raw
  terminal mode is enabled via `crossterm` to prevent echo.
- **Returns** -- The user's input as a trimmed `String`.

### Basic Usage

```rust
use rusty_rich::Console;

let mut console = Console::new();

let name = console.input("Enter your name: ", false);
console.print_str(&format!("[green]Hello, {}![/green]\n", name));
```

The prompt string is written directly to the console's output file (usually
stdout). The method then blocks on `std::io::stdin().read_line()`, trims the
result, and returns it.

---

## Styled Prompts

Since `input()` writes the prompt verbatim, you can apply styling by writing a
styled string beforehand, or by embedding ANSI escape sequences in the prompt
itself.

### Using `print_str()` with Markup

```rust
use rusty_rich::Console;

let mut console = Console::new();

// Write a styled prompt label, then read input
console.print_str("[bold cyan]Username:[/bold cyan] ");
let username = console.input("", false);

// Or use input() for the prompt text with markup disabled
console.options.markup = false;
let path = console.input("\x1b[32mPath:\x1b[0m ", false);
console.options.markup = true;
```

### Using `Prompt` for Styled Prompts

The `prompt` module provides [`Prompt`](prompts.md) which applies theme-based
styling automatically:

```rust
use rusty_rich::{Console, Prompt};

let mut console = Console::new();

let p = Prompt::new("Enter your email")
    .console(console);  // Transfer ownership to the prompt

match p.ask() {
    Ok(email) => println!("You entered: {}", email),
    Err(e) => eprintln!("Prompt error: {}", e),
}
```

The `Prompt` type uses the `prompt` and `prompt.choices` styles from the current
theme to colour the prompt text, giving a consistent look across your
application.

---

## Password Mode

When `password` is `true`, `input()` switches to raw terminal mode and masks
every typed character with `*`.

### How It Works

1. `crossterm::terminal::enable_raw_mode()` is called to disable line buffering
   and echo.
2. Each keystroke is read one byte at a time from stdin.
3. Printable characters are appended to the password buffer and a `*` is written
   to the console.
4. **Backspace** (0x7f or 0x08) removes the last character from the buffer and
   erases one `*` from the display.
5. **Enter** (0x0d or 0x0a) completes input; a newline is written and the
   password string is returned.
6. **Ctrl+C** (0x03) breaks out and returns whatever has been typed so far.
7. Raw mode is disabled before returning.

### Example

```rust
use rusty_rich::Console;

let mut console = Console::new();

console.print_str("[bold]Password setup[/bold]\n");
let pw = console.input("Create a password: ", true);
console.print_str("[green]Password accepted.[/green]\n");
```

### Raw Mode Fallback

If `enable_raw_mode()` fails (for example, when stdout is not a terminal), the
method falls back to plain `read_line()` without masking. The input is still
returned, but characters will be visible as the user types.

```rust
let mut console = Console::new();

// On a non-TTY or when raw mode is unavailable,
// password input falls back to unmasked reading
let secret = console.input("Token: ", true);
```

> **Note:** For production password entry, consider using the `rpassword` crate
> for cross-platform masked input. The built-in masking works on Unix-like
> terminals where `crossterm` raw mode is supported.

---

## Reading from Non-TTY

When stdin is not a terminal (piped input, file redirection, or programmatic
calls), `input()` still works the same way -- it writes the prompt to the
console output and reads from `stdin::read_line()`. There is no TTY detection
inside `input()` itself; it always attempts to write the prompt and read a line.

### Detecting TTY Stdin

You can check whether stdin is a terminal using `atty`:

```rust
use rusty_rich::Console;

let mut console = Console::new();

if atty::is(atty::Stream::Stdin) {
    console.print_str("[green]Interactive mode[/green]\n");
    let name = console.input("Name: ", false);
    console.print_str(&format!("Hello, {}\n", name));
} else {
    // Read from piped input without a prompt
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    console.print_str(&format!("Received: {}\n", input.trim()));
}
```

### Piped Input Example

```rust
use rusty_rich::Console;

// When called as: echo "Alice" | ./app
let mut console = Console::new();

// The prompt is still written to stderr-like output,
// but will appear before the piped input is consumed
let name = console.input("", false);
console.print_str(&format!("Hello, {}\n", name));
```

When input is piped, it is good practice to either suppress the prompt or write
it to stderr so it does not mix with the piped data stream:

```rust
use rusty_rich::Console;
use std::io::Write;

let mut console = Console::new();

if !atty::is(atty::Stream::Stdin) {
    // Write prompt to stderr instead of stdout
    let _ = write!(std::io::stderr(), "Name: ");
    let _ = std::io::stderr().flush();
}

let name = console.input("", false);
console.print_str(&format!("Hello, {}\n", name));
```

---

## Silent / Quiet Prompts

When `console.quiet` is `true`, the `input()` method still reads input -- it
only suppresses the prompt write. This means you can combine quiet mode with
input to create scripts that run silently when `--quiet` is passed:

```rust
use rusty_rich::Console;

let mut console = Console::new();

// When quiet, the prompt is not displayed but input is still read
console.quiet = true;

let answer = console.input("You will not see this prompt: ", false);
// answer is still populated from stdin
```

---

## Complete Example: Interactive CLI with Styled Prompts

The following example demonstrates a small interactive CLI that uses styled
prompts, password masking, and a confirmation loop.

```rust
use rusty_rich::{Console, Style, Color};
use std::io::Write;

fn main() {
    let mut console = Console::new();

    // Title
    console.print_str("[bold underline cyan]User Registration[/bold underline cyan]\n");
    console.print_str("[dim]Press Ctrl+C at any prompt to cancel[/dim]\n\n");

    // --- Styled text input ---
    console.print_str("[bold]Username:[/bold] ");
    let _ = std::io::stdout().flush();  // flush if using write! directly
    let username = console.input("", false);

    if username.is_empty() {
        console.print_str("[red]Username cannot be empty.[/red]\n");
        return;
    }

    // --- Styled password input ---
    console.print_str("[bold]Password:[/bold] ");
    let password = console.input("", true);

    if password.len() < 6 {
        console.print_str("[yellow]Warning:[/yellow] Password is too short.\n");
    }

    // --- Confirmation prompt ---
    console.print_str("[bold]Email:[/bold] ");
    let email = console.input("", false);

    console.print_str("\n[bold cyan]Summary[/bold cyan]\n");
    console.print_str(&format!("  Username: [green]{}[/green]\n", username));
    console.print_str(&format!("  Password: [dim]{}[/dim] characters\n",
        "*".repeat(password.len())));
    console.print_str(&format!("  Email:    [green]{}[/green]\n", email));

    // --- Confirm ---
    loop {
        console.print_str("\n[bold]Save this entry?[/bold] [y/N] ");
        let _ = std::io::stdout().flush();
        let answer = console.input("", false).to_lowercase();

        match answer.as_str() {
            "y" | "yes" => {
                console.print_str("[green]Entry saved successfully![/green]\n");
                break;
            }
            "n" | "no" | "" => {
                console.print_str("[yellow]Entry discarded.[/yellow]\n");
                break;
            }
            _ => {
                console.print_str("[red]Please answer y or n.[/red]\n");
            }
        }
    }
}
```

When run, this produces:

```
User Registration                        <-- bold underline cyan
Press Ctrl+C at any prompt to cancel     <-- dim

Username: alice                          <-- bold prompt
Password: *******                        <-- masked input
Email: alice@example.com                 <-- bold prompt

Summary                                  <-- bold cyan
  Username: alice                        <-- green value
  Password: ******* characters           <-- dim
  Email:    alice@example.com            <-- green value

Save this entry? [y/N] y                 <-- styled confirmation
Entry saved successfully!                <-- green confirmation
```

### Running the Example

Save the code to `examples/interactive_cli.rs` and run:

```bash
cargo run --example interactive_cli
```

---

## Comparison: `Console::input()` vs `Prompt`

| Feature                | `Console::input()`                   | `Prompt` / `Confirm` / `IntPrompt`   |
|------------------------|--------------------------------------|--------------------------------------|
| Styled prompt          | Manual (write ANSI first)            | Automatic (theme-based)              |
| Password masking       | Built-in (`password: true`)          | Built-in (`.password(true)`)         |
| Default values         | Not supported                        | `Confirm::default`                   |
| Choice validation      | Not supported                        | `Prompt::choices()`                  |
| Type coercion          | Returns `String`                     | `IntPrompt` -> `i64`, `FloatPrompt` -> `f64`, `Confirm` -> `bool` |
| Error handling         | Returns `String` (always succeeds)   | Returns `Result<T, PromptError>`     |
| Retry on invalid       | Not supported                        | Loops until valid input (IntPrompt, FloatPrompt, Select) |

Use `Console::input()` when you need a quick, unvalidated string from the user
and want to control the prompt style yourself. Use the `Prompt` family when you
need validation, type coercion, or consistent theme-driven styling.

---

## Related

- [Prompt API](prompts.md) -- The `Prompt`, `Confirm`, `IntPrompt`, `FloatPrompt`,
  and `Select` types for structured user input.
- [Console API](../api/console.md) -- Full `Console` method reference.
- [Style & Color](../core-concepts/style-and-color.md) -- How styles and colours
  work in rusty-rich.
- [Text & Markup](../core-concepts/text-and-markup.md) -- Using markup for
  inline styling of prompt text.
