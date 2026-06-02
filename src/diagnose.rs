//! Error diagnostics — rich-formatted error reporting.

use crate::console::Console;
use crate::panel::Panel;
use crate::style::Style;
use crate::text::Text;

/// Generate and print a diagnostic report for an error.
pub fn report(error: &dyn std::error::Error, console: &mut Console) {
    let panel = Panel::new(format_error_text(error))
        .title("Error Report")
        .border_style(Style::new().color(crate::color::Color::parse("red").unwrap()));
    console.println(&panel);
}

/// Create a diagnostic Text from an error chain.
pub fn diagnose(error: &dyn std::error::Error) -> Text {
    let mut text = Text::new(format!("Error: {}\n", error));
    if let Some(source) = error.source() {
        text.append_styled(
            format!("Caused by: {}\n", source),
            Style::new().dim(true),
        );
    }
    text
}

fn format_error_text(error: &dyn std::error::Error) -> Text {
    let mut text = diagnose(error);
    // Add help text based on error type
    text.append_styled("\n───", Style::new().dim(true));
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_diagnose() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let text = diagnose(&err);
        assert!(text.plain.contains("file not found"));
    }
}
