//! JSON pretty printing — equivalent to Rich's `json.py`.

use serde_json::Value;
use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::theme::Theme;

/// Render a JSON value with syntax highlighting.
pub fn render_json(value: &Value) -> JsonRender {
    JsonRender {
        value: value.clone(),
        indent: 2,
        theme: None,
        highlight: true,
    }
}

#[derive(Debug, Clone)]
pub struct JsonRender {
    value: Value,
    indent: usize,
    theme: Option<Theme>,
    highlight: bool,
}

impl JsonRender {
    pub fn indent(mut self, n: usize) -> Self { self.indent = n; self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = Some(t); self }

    fn style_for(&self, name: &str) -> Style {
        if let Some(ref t) = self.theme {
            t.get(name).cloned().unwrap_or(Style::new())
        } else {
            
            let theme = crate::theme::default_theme();
            theme.get(name).cloned().unwrap_or(Style::new())
        }
    }
}

impl Renderable for JsonRender {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        let formatted = format_json_value(
            &self.value,
            self.indent,
            0,
            self.highlight,
            &|name| self.style_for(name),
        );
        let lines: Vec<Vec<Segment>> = formatted
            .lines()
            .map(|line| vec![Segment::new(line)])
            .collect();
        RenderResult { lines, items: Vec::new() }
    }
}

fn format_json_value(
    value: &Value,
    indent: usize,
    level: usize,
    highlight: bool,
    get_style: &dyn Fn(&str) -> Style,
) -> String {
    if !highlight {
        return serde_json::to_string_pretty(value).unwrap_or_else(|_| format!("{value:?}"));
    }

    match value {
        Value::Null => {
            let s = get_style("json.null");
            format!("{}{}null{}", s.to_ansi(), s.reset_ansi(), "")
        }
        Value::Bool(b) => {
            let s = get_style("json.bool");
            format!("{}{b}{}", s.to_ansi(), s.reset_ansi())
        }
        Value::Number(n) => {
            let s = get_style("json.number");
            format!("{}{n}{}", s.to_ansi(), s.reset_ansi())
        }
        Value::String(s) => {
            let st = get_style("json.str");
            format!("{}\"{}\"{}", st.to_ansi(), s.escape_default(), st.reset_ansi())
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let brace = get_style("json.brace");
            let mut out = format!("{}[{}", brace.to_ansi(), brace.reset_ansi());
            let pad = " ".repeat(indent * (level + 1));
            let close_pad = " ".repeat(indent * level);

            for (i, item) in arr.iter().enumerate() {
                let item_str = format_json_value(item, indent, level + 1, highlight, get_style);
                out.push('\n');
                out.push_str(&pad);
                out.push_str(&item_str);
                if i < arr.len() - 1 {
                    out.push(',');
                }
            }
            out.push('\n');
            out.push_str(&close_pad);
            out.push_str(&format!("{}]{}", brace.to_ansi(), brace.reset_ansi()));
            out
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_string();
            }
            let brace = get_style("json.brace");
            let key_style = get_style("json.key");
            let mut out = format!("{}{{{}", brace.to_ansi(), brace.reset_ansi());
            let pad = " ".repeat(indent * (level + 1));
            let close_pad = " ".repeat(indent * level);
            let keys: Vec<&String> = obj.keys().collect();

            for (i, key) in keys.iter().enumerate() {
                let val = &obj[*key];
                let val_str = format_json_value(val, indent, level + 1, highlight, get_style);
                out.push('\n');
                out.push_str(&pad);
                out.push_str(&format!(
                    "{}\"{}\"{}: ",
                    key_style.to_ansi(),
                    key.escape_default(),
                    key_style.reset_ansi()
                ));
                out.push_str(&val_str);
                if i < keys.len() - 1 {
                    out.push(',');
                }
            }
            out.push('\n');
            out.push_str(&close_pad);
            out.push_str(&format!("{}}}{}", brace.to_ansi(), brace.reset_ansi()));
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json() {
        let v: Value = serde_json::from_str(r#"{"name": "Alice", "age": 30}"#).unwrap();
        let rendered = render_json(&v);
        let result = rendered.render(&ConsoleOptions::default());
        assert!(result.to_ansi().contains("Alice"));
    }
}
