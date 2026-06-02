//! Variable scope inspection and rendering.

use crate::console::Console;
use crate::panel::Panel;
use crate::style::Style;
use crate::table::Table;
use crate::text::Text;

/// Render a variable scope (name → value mapping) as a table.
pub fn render_scope(
    variables: &[(&str, &dyn std::fmt::Display)],
    title: Option<&str>,
    console: &mut Console,
) {
    let mut table = Table::new();
    table.add_column(crate::table::Column::new("Variable"));
    table.add_column(crate::table::Column::new("Value"));
    table.add_column(crate::table::Column::new("Type"));

    for (name, value) in variables {
        let type_name = std::any::type_name_of_val(value);
        table.add_row_str(vec![
            name.to_string(),
            value.to_string(),
            type_name.to_string(),
        ]);
    }

    let styled_table = table;
    if let Some(t) = title {
        let panel = Panel::new(styled_table)
            .title(t)
            .border_style(Style::new().color(crate::color::Color::parse("blue").unwrap()));
        console.println(&panel);
    } else {
        console.println(&styled_table);
    }
}

/// Create a scope summary as Text.
pub fn scope_summary(variables: &[(&str, &dyn std::fmt::Display)]) -> Text {
    let mut text = Text::new("");
    for (i, (name, value)) in variables.iter().enumerate() {
        if i > 0 {
            text.plain.push('\n');
        }
        text.append_styled(
            format!("  {}: ", name),
            Style::new()
                .bold(true)
                .color(crate::color::Color::parse("cyan").unwrap()),
        );
        text.append_styled(value.to_string(), Style::new());
    }
    text
}
