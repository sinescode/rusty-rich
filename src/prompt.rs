//! Interactive prompts — equivalent to Rich's `rich/prompt.py`.
//!
//! This module provides types for prompting the user for input:
//!
//! - `Prompt` — string input
//! - `IntPrompt` — integer input
//! - `FloatPrompt` — float input
//! - `Confirm` — yes / no input
//! - `Select<T>` — pick from a list of named choices
//!
//! All prompts support:
//! - Optional `Console` for styled output (falls back to raw stdout)
//! - Password mode (hidden input, masked with `*`)
//! - Choice validation with optional case sensitivity
//! - Display of default values and choices

use std::fmt;
use std::io::{self, BufRead, Write};

use crate::console::Console;
use crate::style::Style;

// ---------------------------------------------------------------------------
// PromptError
// ---------------------------------------------------------------------------

/// Errors that can occur during prompting.
#[derive(Debug)]
pub enum PromptError {
    /// The user provided an invalid response.
    InvalidResponse(String),
    /// An underlying I/O error occurred.
    IOError(io::Error),
    /// The user cancelled the prompt (e.g. Ctrl+C / Ctrl+D).
    Cancelled,
}

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResponse(msg) => write!(f, "{}", msg),
            Self::IOError(e) => write!(f, "I/O error: {}", e),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for PromptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IOError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for PromptError {
    fn from(e: io::Error) -> Self {
        PromptError::IOError(e)
    }
}

// ---------------------------------------------------------------------------
// Password reader (crossterm raw mode, no rpassword dependency)
// ---------------------------------------------------------------------------

/// Read a line of input with echoing disabled; show `*` for each character.
/// Handles backspace for erasing the last character.
fn read_password() -> Result<String, PromptError> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    enable_raw_mode().map_err(PromptError::IOError)?;

    let mut result = String::new();

    let cleanup = || {
        let _ = disable_raw_mode();
    };

    loop {
        match event::read() {
            Ok(Event::Key(key))
                if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat =>
            {
                match key.code {
                    KeyCode::Enter => {
                        let _ = io::stdout().write(b"\n");
                        let _ = io::stdout().flush();
                        break;
                    }
                    KeyCode::Char(c) => {
                        result.push(c);
                        let _ = io::stdout().write(b"*");
                        let _ = io::stdout().flush();
                    }
                    KeyCode::Backspace => {
                        if result.pop().is_some() {
                            let _ = io::stdout().write(b"\x08 \x08");
                            let _ = io::stdout().flush();
                        }
                    }
                    KeyCode::Esc | KeyCode::Delete => {
                        cleanup();
                        return Err(PromptError::Cancelled);
                    }
                    _ => {}
                }
            }
            Ok(Event::Key(key)) if key.code == KeyCode::Enter => {
                let _ = io::stdout().write(b"\n");
                let _ = io::stdout().flush();
                break;
            }
            Ok(Event::Key(key)) if key.code == KeyCode::Esc => {
                cleanup();
                return Err(PromptError::Cancelled);
            }
            Ok(_) => {}
            Err(e) => {
                cleanup();
                return Err(PromptError::IOError(e));
            }
        }
    }

    cleanup();
    Ok(result)
}

// ---------------------------------------------------------------------------
// PromptBase
// ---------------------------------------------------------------------------

/// Base configuration for all prompt types.
///
/// Holds common fields like the prompt text, optional console, password mode,
/// choices list, case sensitivity, and display flags.
pub struct PromptBase {
    /// The prompt text to display.
    pub prompt: String,
    /// An optional `Console` for styled output. If `None`, writes directly to
    /// `std::io::stdout()`.
    pub console: Option<Console>,
    /// When `true`, input characters are masked with `*`.
    pub password: bool,
    /// Optional list of valid choices. When set, the user's response is
    /// validated against this list.
    pub choices: Option<Vec<String>>,
    /// Whether choice matching is case-sensitive (default `false`).
    pub case_sensitive: bool,
    /// Whether to show the default value in the prompt string.
    pub show_default: bool,
    /// Whether to show the list of choices in the prompt string.
    pub show_choices: bool,
}

impl PromptBase {
    /// Create a new `PromptBase` with the given prompt text.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            console: None,
            password: false,
            choices: None,
            case_sensitive: false,
            show_default: true,
            show_choices: true,
        }
    }

    /// Builder: set the console.
    pub fn console(mut self, console: Console) -> Self {
        self.console = Some(console);
        self
    }

    /// Builder: enable or disable password mode.
    pub fn password(mut self, yes: bool) -> Self {
        self.password = yes;
        self
    }

    /// Builder: set the valid choices.
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.choices = Some(choices);
        self
    }

    /// Builder: set case sensitivity for choice validation.
    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.case_sensitive = yes;
        self
    }

    /// Builder: show or hide the default value.
    pub fn show_default(mut self, yes: bool) -> Self {
        self.show_default = yes;
        self
    }

    /// Builder: show or hide the choices list.
    pub fn show_choices(mut self, yes: bool) -> Self {
        self.show_choices = yes;
        self
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Format the default value for display.
    ///
    /// Returns `" (default: value)"` wrapped in the `prompt.default` style, or
    /// an empty string if `show_default` is `false`.
    pub fn render_default(&self, default: &str) -> String {
        if !self.show_default || default.is_empty() {
            return String::new();
        }
        let styled = apply_style(default, "prompt.default");
        format!(" ({})", styled)
    }

    /// Build the full prompt string including choices and default.
    ///
    /// Returns a string like:
    /// `"Enter choice [a/b/c] (default: x): "`
    pub fn make_prompt(&self) -> String {
        let mut parts = Vec::new();

        // Choices display
        if self.show_choices {
            if let Some(choices) = &self.choices {
                let display_choices: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();
                let styled = apply_style(&display_choices.join("/"), "prompt.choices");
                parts.push(format!("[{}]", styled));
            }
        }

        let suffix = if parts.is_empty() {
            String::new()
        } else {
            format!(" {} ", parts.join(" "))
        };

        let styled_prompt = apply_style(&self.prompt, "prompt");
        format!("{}{}: ", styled_prompt, suffix)
    }

    /// Check whether `value` is a valid choice.
    ///
    /// If `choices` is `None`, returns `true`.
    /// Otherwise returns `true` only if `value` (optionally case-insensitive)
    /// matches one of the allowed choices.
    pub fn check_choice(&self, value: &str) -> bool {
        match &self.choices {
            None => true,
            Some(choices) => {
                if self.case_sensitive {
                    choices.iter().any(|c| c == value)
                } else {
                    let lower = value.to_lowercase();
                    choices.iter().any(|c| c.to_lowercase() == lower)
                }
            }
        }
    }

    /// Read a line from stdin.
    fn read_line(&self) -> Result<String, PromptError> {
        if self.password {
            read_password()
        } else {
            let mut input = String::new();
            io::stdin()
                .lock()
                .read_line(&mut input)
                .map_err(PromptError::IOError)?;
            if input.is_empty() {
                return Err(PromptError::Cancelled);
            }
            Ok(input
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string())
        }
    }

    /// Write a string to stdout.
    fn write_output(&self, text: &str) -> Result<(), PromptError> {
        let mut out = io::stdout();
        out.write_all(text.as_bytes())?;
        out.flush()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Prompt (string)
// ---------------------------------------------------------------------------

/// Prompt the user for a string.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::Prompt;
///
/// let name = Prompt::ask_with("Enter name").unwrap();
/// ```
pub struct Prompt {
    base: PromptBase,
}

impl Prompt {
    /// Create a new string prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            base: PromptBase::new(prompt),
        }
    }

    /// Builder: set the console.
    pub fn console(mut self, console: Console) -> Self {
        self.base.console = Some(console);
        self
    }

    /// Builder: enable password mode.
    pub fn password(mut self, yes: bool) -> Self {
        self.base.password = yes;
        self
    }

    /// Builder: set valid choices.
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.base.choices = Some(choices);
        self
    }

    /// Builder: set case sensitivity.
    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.base.case_sensitive = yes;
        self
    }

    /// Builder: show or hide choices.
    pub fn show_choices(mut self, yes: bool) -> Self {
        self.base.show_choices = yes;
        self
    }

    /// Builder: show or hide default.
    pub fn show_default(mut self, yes: bool) -> Self {
        self.base.show_default = yes;
        self
    }

    /// Render the prompt string with styling applied.
    ///
    /// Returns a styled string like `"Enter name: "` where the prompt text and
    /// choices are colored using the theme's `prompt` and `prompt.choices`
    /// styles.
    pub fn render(&self) -> String {
        self.base.make_prompt()
    }

    /// Ask the user for string input.
    ///
    /// Displays the prompt, reads a line from stdin, validates it against
    /// any configured choices, and returns the trimmed string.
    ///
    /// # Errors
    ///
    /// Returns `PromptError::Cancelled` on EOF or Ctrl+C,
    /// `PromptError::InvalidResponse` when the input does not match choices,
    /// and `PromptError::IOError` on I/O failures.
    pub fn ask(&self) -> Result<String, PromptError> {
        let prompt_str = self.base.make_prompt();
        self.base.write_output(&prompt_str)?;
        let value = self.base.read_line()?;
        if !self.base.check_choice(&value) {
            return Err(PromptError::InvalidResponse(format!(
                "invalid choice: '{}'",
                value
            )));
        }
        Ok(value)
    }

    /// Convenience: create a prompt, ask, and return the result.
    ///
    /// Equivalent to `Prompt::new(prompt).ask()`.
    pub fn ask_with(prompt: impl Into<String>) -> Result<String, PromptError> {
        Prompt::new(prompt).ask()
    }
}

// ---------------------------------------------------------------------------
// IntPrompt
// ---------------------------------------------------------------------------

/// Prompt the user for an integer.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::IntPrompt;
///
/// let age = IntPrompt::ask_with("Enter age").unwrap();
/// ```
pub struct IntPrompt {
    base: PromptBase,
}

impl IntPrompt {
    /// Create a new integer prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            base: PromptBase::new(prompt),
        }
    }

    /// Builder: set the console.
    pub fn console(mut self, console: Console) -> Self {
        self.base.console = Some(console);
        self
    }

    /// Builder: enable password mode.
    pub fn password(mut self, yes: bool) -> Self {
        self.base.password = yes;
        self
    }

    /// Builder: set valid choices.
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.base.choices = Some(choices);
        self
    }

    /// Builder: set case sensitivity.
    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.base.case_sensitive = yes;
        self
    }

    /// Ask the user for an integer.
    ///
    /// Reads input and attempts to parse it as `i64`. Loops until a valid
    /// integer is provided.
    ///
    /// # Errors
    ///
    /// Returns `PromptError::Cancelled` on EOF or Ctrl+C.
    /// Returns `PromptError::IOError` on I/O failures.
    pub fn ask(&self) -> Result<i64, PromptError> {
        loop {
            let prompt_str = self.base.make_prompt();
            self.base.write_output(&prompt_str)?;
            let value = self.base.read_line()?;
            if value.is_empty() {
                continue;
            }
            if !self.base.check_choice(&value) {
                let _ = self
                    .base
                    .write_output(&format!("Invalid choice: '{}'. Please try again.\n", value));
                continue;
            }
            match value.parse::<i64>() {
                Ok(n) => return Ok(n),
                Err(_) => {
                    let _ = self.base.write_output("Please enter a valid integer.\n");
                }
            }
        }
    }

    /// Convenience: create an integer prompt, ask, and return the result.
    pub fn ask_with(prompt: impl Into<String>) -> Result<i64, PromptError> {
        IntPrompt::new(prompt).ask()
    }
}

// ---------------------------------------------------------------------------
// FloatPrompt
// ---------------------------------------------------------------------------

/// Prompt the user for a floating-point number.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::FloatPrompt;
///
/// let height = FloatPrompt::ask_with("Enter height").unwrap();
/// ```
pub struct FloatPrompt {
    base: PromptBase,
}

impl FloatPrompt {
    /// Create a new float prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            base: PromptBase::new(prompt),
        }
    }

    /// Builder: set the console.
    pub fn console(mut self, console: Console) -> Self {
        self.base.console = Some(console);
        self
    }

    /// Builder: enable password mode.
    pub fn password(mut self, yes: bool) -> Self {
        self.base.password = yes;
        self
    }

    /// Builder: set valid choices.
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.base.choices = Some(choices);
        self
    }

    /// Builder: set case sensitivity.
    pub fn case_sensitive(mut self, yes: bool) -> Self {
        self.base.case_sensitive = yes;
        self
    }

    /// Ask the user for a float.
    ///
    /// Reads input and attempts to parse it as `f64`. Loops until a valid
    /// float is provided.
    ///
    /// # Errors
    ///
    /// Returns `PromptError::Cancelled` on EOF or Ctrl+C.
    /// Returns `PromptError::IOError` on I/O failures.
    pub fn ask(&self) -> Result<f64, PromptError> {
        loop {
            let prompt_str = self.base.make_prompt();
            self.base.write_output(&prompt_str)?;
            let value = self.base.read_line()?;
            if value.is_empty() {
                continue;
            }
            if !self.base.check_choice(&value) {
                let _ = self
                    .base
                    .write_output(&format!("Invalid choice: '{}'. Please try again.\n", value));
                continue;
            }
            match value.parse::<f64>() {
                Ok(n) => return Ok(n),
                Err(_) => {
                    let _ = self.base.write_output("Please enter a valid number.\n");
                }
            }
        }
    }

    /// Convenience: create a float prompt, ask, and return the result.
    pub fn ask_with(prompt: impl Into<String>) -> Result<f64, PromptError> {
        FloatPrompt::new(prompt).ask()
    }
}

// ---------------------------------------------------------------------------
// Confirm
// ---------------------------------------------------------------------------

/// Prompt the user for a yes/no answer.
///
/// Returns `bool` where `true` means yes / affirmative.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::Confirm;
///
/// let ok = Confirm::ask_with("Continue?", true).unwrap();
/// ```
pub struct Confirm {
    base: PromptBase,
    /// Default answer if the user presses Enter without typing.
    pub default: bool,
}

impl Confirm {
    /// Create a new confirmation prompt with a default answer.
    pub fn new(prompt: impl Into<String>, default: bool) -> Self {
        Self {
            base: PromptBase::new(prompt),
            default,
        }
    }

    /// Builder: set the console.
    pub fn console(mut self, console: Console) -> Self {
        self.base.console = Some(console);
        self
    }

    /// Build the confirmation prompt string.
    ///
    /// Displays `[y/N]` or `[Y/n]` depending on the default, followed by `: `.
    fn make_confirm_prompt(&self) -> String {
        let (yes, no) = if self.default { ("Y", "n") } else { ("y", "N") };
        let styled_prompt = apply_style(&self.base.prompt, "prompt");
        let styled_choices = apply_style(&format!("[{}/{}]", yes, no), "prompt.choices");
        format!("{} {}: ", styled_prompt, styled_choices)
    }

    /// Ask the user for a yes/no answer.
    ///
    /// Recognises `y`, `yes`, `true`, `1` as affirmative;
    /// `n`, `no`, `false`, `0` as negative.
    /// An empty input returns the default.
    ///
    /// # Errors
    ///
    /// Returns `PromptError::Cancelled` on EOF or Ctrl+C.
    pub fn ask(&self) -> Result<bool, PromptError> {
        loop {
            let prompt_str = self.make_confirm_prompt();
            self.base.write_output(&prompt_str)?;
            let value = self.base.read_line()?;
            match value.to_lowercase().as_str() {
                "" => return Ok(self.default),
                "y" | "yes" | "true" | "1" => return Ok(true),
                "n" | "no" | "false" | "0" => return Ok(false),
                _ => {
                    let _ = self.base.write_output("Please answer y or n.\n");
                }
            }
        }
    }

    /// Convenience: create a confirmation prompt with the given default,
    /// ask, and return the result.
    pub fn ask_with(prompt: impl Into<String>, default: bool) -> Result<bool, PromptError> {
        Confirm::new(prompt, default).ask()
    }
}

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

/// Prompt the user to select from a list of named choices.
///
/// Each choice is a `(label, value)` pair. The user selects by number.
///
/// # Example
///
/// ```rust,no_run
/// use rusty_rich::Select;
///
/// let choice = Select::new("Pick a color")
///     .choice("Red", "red")
///     .choice("Green", "green")
///     .choice("Blue", "blue")
///     .ask()
///     .unwrap();
/// ```
pub struct Select<T> {
    base: PromptBase,
    choices: Vec<(String, T)>,
}

impl<T> Select<T> {
    /// Create a new select prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            base: PromptBase::new(prompt),
            choices: Vec::new(),
        }
    }

    /// Builder: set the console.
    pub fn console(mut self, console: Console) -> Self {
        self.base.console = Some(console);
        self
    }

    /// Builder: add a choice with the given label and value.
    pub fn choice(mut self, label: impl Into<String>, value: T) -> Self {
        self.choices.push((label.into(), value));
        self
    }
}

impl<T: fmt::Display> Select<T> {
    /// Render the select prompt as a numbered list.
    ///
    /// Returns a multi-line string like:
    /// ```text
    /// Pick a color:
    ///   1) Red
    ///   2) Green
    ///   3) Blue
    /// Enter number [1-3]:
    /// ```
    pub fn render(&self) -> String {
        let mut output = String::new();
        let styled_prompt = apply_style(&self.base.prompt, "prompt");
        output.push_str(&styled_prompt);
        output.push('\n');

        for (i, (label, _)) in self.choices.iter().enumerate() {
            output.push_str(&format!("  {}) {}\n", i + 1, label));
        }

        let styled_choices = apply_style(
            &format!("Enter number [1-{}]", self.choices.len()),
            "prompt.choices",
        );
        output.push_str(&format!("{}: ", styled_choices));
        output
    }
}

impl<T: fmt::Display + Clone> Select<T> {
    /// Ask the user to select from the choices.
    ///
    /// Displays a numbered list, then prompts for a number.
    /// Loops until a valid number is entered.
    ///
    /// # Errors
    ///
    /// Returns `PromptError::Cancelled` on EOF or Ctrl+C.
    /// Returns `PromptError::InvalidResponse` if there are no choices.
    pub fn ask(&self) -> Result<T, PromptError> {
        if self.choices.is_empty() {
            return Err(PromptError::InvalidResponse("no choices available".into()));
        }

        let prompt_str = self.render();
        self.base.write_output(&prompt_str)?;

        loop {
            let value = self.base.read_line()?;
            if value.is_empty() {
                continue;
            }
            match value.trim().parse::<usize>() {
                Ok(n) if n >= 1 && n <= self.choices.len() => {
                    return Ok(self.choices[n - 1].1.clone());
                }
                _ => {
                    let _ = self.base.write_output(&format!(
                        "Please enter a number between 1 and {}.\n",
                        self.choices.len()
                    ));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: apply a theme style name to text via ANSI escapes
// ---------------------------------------------------------------------------

/// Apply the ANSI style for the given theme key to `text`.
///
/// Falls back to plain text if no style is configured.
fn apply_style(text: &str, style_name: &str) -> String {
    let theme = crate::theme::default_theme();
    if let Some(style) = theme.get(style_name) {
        let ansi = style.to_ansi();
        if ansi.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{}m{}\x1b[0m", ansi, text)
        }
    } else {
        text.to_string()
    }
}

/// Apply a raw `Style` to text via ANSI escapes.
#[allow(dead_code)]
fn apply_raw_style(text: &str, style: &Style) -> String {
    let ansi = style.to_ansi();
    if ansi.is_empty() {
        text.to_string()
    } else {
        format!("\x1b[{}m{}\x1b[0m", ansi, text)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- PromptBase tests ---------------------------------------------------

    #[test]
    fn test_make_prompt_no_choices() {
        let pb = PromptBase::new("Enter name");
        let result = pb.make_prompt();
        assert!(result.contains("Enter name"));
        assert!(result.ends_with(": "));
    }

    #[test]
    fn test_make_prompt_with_choices() {
        let pb = PromptBase::new("Choose").choices(vec!["a".into(), "b".into()]);
        let result = pb.make_prompt();
        assert!(result.contains("Choose"));
        assert!(result.contains("["));
        assert!(result.contains("a/b"));
        assert!(result.contains("]"));
    }

    #[test]
    fn test_render_default() {
        let pb = PromptBase::new("test");
        let rendered = pb.render_default("hello");
        assert!(rendered.contains("hello"));

        let pb_hidden = PromptBase::new("test").show_default(false);
        let rendered_hidden = pb_hidden.render_default("hello");
        assert_eq!(rendered_hidden, "");
    }

    #[test]
    fn test_check_choice_no_choices() {
        let pb = PromptBase::new("test");
        assert!(pb.check_choice("anything"));
    }

    #[test]
    fn test_check_choice_case_insensitive() {
        let pb = PromptBase::new("test")
            .choices(vec!["yes".into(), "no".into()])
            .case_sensitive(false);
        assert!(pb.check_choice("YES"));
        assert!(pb.check_choice("yes"));
        assert!(pb.check_choice("No"));
        assert!(!pb.check_choice("maybe"));
    }

    #[test]
    fn test_check_choice_case_sensitive() {
        let pb = PromptBase::new("test")
            .choices(vec!["Yes".into(), "No".into()])
            .case_sensitive(true);
        assert!(pb.check_choice("Yes"));
        assert!(!pb.check_choice("yes"));
    }

    // -- PromptError tests --------------------------------------------------

    #[test]
    fn test_prompt_error_display() {
        let err = PromptError::InvalidResponse("bad input".into());
        assert_eq!(format!("{}", err), "bad input");

        let err = PromptError::Cancelled;
        assert_eq!(format!("{}", err), "cancelled");

        let io_err = io::Error::new(io::ErrorKind::Other, "oh no");
        let err = PromptError::IOError(io_err);
        let msg = format!("{}", err);
        assert!(msg.contains("I/O error"));
    }

    #[test]
    fn test_prompt_error_source() {
        use std::error::Error;

        let err = PromptError::InvalidResponse("bad".into());
        assert!(err.source().is_none());

        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let err = PromptError::IOError(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "oh no");
        let err: PromptError = io_err.into();
        match err {
            PromptError::IOError(_) => {}
            _ => panic!("expected IOError"),
        }
    }

    // -- Confirm tests ------------------------------------------------------

    #[test]
    fn test_confirm_prompt_text_default_true() {
        let c = Confirm::new("Continue?", true);
        let prompt = c.make_confirm_prompt();
        assert!(prompt.contains("Continue?"));
        assert!(prompt.contains("[Y/n]"));
    }

    #[test]
    fn test_confirm_prompt_text_default_false() {
        let c = Confirm::new("Continue?", false);
        let prompt = c.make_confirm_prompt();
        assert!(prompt.contains("Continue?"));
        assert!(prompt.contains("[y/N]"));
    }

    // -- Select tests -------------------------------------------------------

    #[test]
    fn test_select_render() {
        let s: Select<&str> = Select::new("Pick")
            .choice("Option A", "a")
            .choice("Option B", "b");
        let rendered = s.render();
        assert!(rendered.contains("Pick"));
        assert!(rendered.contains("1) Option A"));
        assert!(rendered.contains("2) Option B"));
        assert!(rendered.contains("Enter number [1-2]"));
    }

    #[test]
    fn test_select_no_choices_error() {
        let s: Select<String> = Select::new("empty");
        let result = s.ask();
        match result {
            Err(PromptError::InvalidResponse(msg)) => {
                assert!(msg.contains("no choices"));
            }
            _ => panic!("expected InvalidResponse for no choices"),
        }
    }

    // -- Prompt builder tests -----------------------------------------------

    #[test]
    fn test_prompt_builder() {
        let p = Prompt::new("Enter value")
            .password(false)
            .show_choices(true);
        let rendered = p.render();
        assert!(rendered.contains("Enter value"));
    }

    #[test]
    fn test_prompt_render_default() {
        let pb = PromptBase::new("Name").show_default(true);
        assert!(pb.render_default("Alice").contains("Alice"));
    }

    // -- Style helper tests -------------------------------------------------

    #[test]
    fn test_apply_style_plain() {
        let result = apply_style("hello", "nonexistent.style");
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_apply_style_with_theme() {
        let result = apply_style("hello", "prompt");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_apply_raw_style_empty() {
        let s = Style::new();
        let result = apply_raw_style("test", &s);
        assert_eq!(result, "test");
    }
}
