//! Object introspection — equivalent to Rich's `_inspect.py`.
//!
//! Provides the [`Inspect`] renderable for displaying structured information
//! about Rust values: their type name, debug representation, attributes (via
//! serde or manual key-value pairs), and documentation.
//!
//! Since Rust lacks Python's runtime introspection, this module works with
//! the [`std::fmt::Debug`] trait, `serde::Serialize`, and manual attribute
//! maps to produce Rich-styled inspection output.
//!
//! # Quick Example
//!
//! ```rust,ignore
//! use rusty_rich::inspect::Inspect;
//!
//! let obj = vec![1, 2, 3];
//! let insp = Inspect::new(&obj)
//!     .title("my_vec")
//!     .methods(true)
//!     .docs(true);
//! ```

use crate::color::Color;
use crate::console::{ConsoleOptions, RenderResult, Renderable};
#[cfg(feature = "syntax-highlighting")]
use crate::highlighter::ReprHighlighter;
use crate::panel::Panel;
use crate::segment::Segment;
use crate::style::Style;
use crate::table::{Column, Table};

// ---------------------------------------------------------------------------
// Inspect — structured object inspection
// ---------------------------------------------------------------------------

/// A renderable that displays structured inspection of a value.
///
/// Shows the type name, debug/value representation, and optional attribute
/// table with types and documentation.
#[derive(Debug, Clone)]
pub struct Inspect {
    /// The debug/value representation of the object.
    value_repr: String,
    /// The type name to display.
    title: Option<String>,
    /// Show full help-style output (more detail).
    help: bool,
    /// Show callable/method-like attributes.
    methods: bool,
    /// Show doc/comment strings.
    docs: bool,
    /// Show private attributes (prefixed with `_`).
    private: bool,
    /// Show dunder attributes (prefixed with `__`).
    dunder: bool,
    /// Sort attributes alphabetically.
    sort: bool,
    /// Show all attributes regardless of prefix.
    all: bool,
    /// Pretty-print values.
    value: bool,
    /// Attribute definitions: (name, type_str, value_str, doc_string).
    attrs: Vec<(String, String, String, Option<String>)>,
    /// Method definitions: (name, signature, doc_string).
    method_list: Vec<(String, String, Option<String>)>,
    /// Doc/summary text for the object itself.
    doc_text: Option<String>,
}

impl Inspect {
    /// Create a new `Inspect` for a value that implements [`std::fmt::Debug`].
    ///
    /// The debug representation is captured immediately.
    pub fn new(value: &dyn std::fmt::Debug) -> Self {
        Self {
            value_repr: format!("{value:?}"),
            title: None,
            help: false,
            methods: false,
            docs: true,
            private: false,
            dunder: false,
            sort: true,
            all: false,
            value: true,
            attrs: Vec::new(),
            method_list: Vec::new(),
            doc_text: None,
        }
    }

    /// Create a new `Inspect` from a string value representation.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value_repr: impl Into<String>) -> Self {
        Self {
            value_repr: value_repr.into(),
            title: None,
            help: false,
            methods: false,
            docs: true,
            private: false,
            dunder: false,
            sort: true,
            all: false,
            value: true,
            attrs: Vec::new(),
            method_list: Vec::new(),
            doc_text: None,
        }
    }

    /// Builder: set a custom title (overrides type name).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Builder: show full detail / help mode.
    pub fn help(mut self, value: bool) -> Self {
        self.help = value;
        self
    }

    /// Builder: include method-like entries.
    pub fn methods(mut self, value: bool) -> Self {
        self.methods = value;
        self
    }

    /// Builder: show documentation text.
    pub fn docs(mut self, value: bool) -> Self {
        self.docs = value;
        self
    }

    /// Builder: show private attributes (`_` prefix).
    pub fn private(mut self, value: bool) -> Self {
        self.private = value;
        self
    }

    /// Builder: show dunder attributes (`__` prefix).
    pub fn dunder(mut self, value: bool) -> Self {
        self.dunder = value;
        self
    }

    /// Builder: sort attributes alphabetically.
    pub fn sort(mut self, value: bool) -> Self {
        self.sort = value;
        self
    }

    /// Builder: show all attributes.
    pub fn all(mut self, value: bool) -> Self {
        self.all = value;
        self
    }

    /// Builder: enable value pretty-printing.
    pub fn value(mut self, value: bool) -> Self {
        self.value = value;
        self
    }

    /// Add an attribute to the inspection output.
    ///
    /// Parameters:
    /// - `name`: the attribute name
    /// - `type_name`: the type of the attribute (e.g. `"String"`, `"i32"`)
    /// - `value`: the debug/display representation of the value
    pub fn add_attr(
        mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.attrs
            .push((name.into(), type_name.into(), value.into(), None));
        self
    }

    /// Add an attribute with documentation.
    pub fn add_attr_doc(
        mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        value: impl Into<String>,
        doc: impl Into<String>,
    ) -> Self {
        self.attrs.push((
            name.into(),
            type_name.into(),
            value.into(),
            Some(doc.into()),
        ));
        self
    }

    /// Add a method to the inspection output.
    pub fn add_method(mut self, name: impl Into<String>, signature: impl Into<String>) -> Self {
        self.method_list.push((name.into(), signature.into(), None));
        self
    }

    /// Add a method with documentation.
    pub fn add_method_doc(
        mut self,
        name: impl Into<String>,
        signature: impl Into<String>,
        doc: impl Into<String>,
    ) -> Self {
        self.method_list
            .push((name.into(), signature.into(), Some(doc.into())));
        self
    }

    /// Set the object-level documentation text.
    pub fn doc_text(mut self, doc: impl Into<String>) -> Self {
        self.doc_text = Some(doc.into());
        self
    }

    /// Build the attribute table from a list of `(name, type, value)` tuples.
    pub fn with_attrs(mut self, attrs: Vec<(String, String, String)>) -> Self {
        self.attrs = attrs.into_iter().map(|(n, t, v)| (n, t, v, None)).collect();
        self
    }

    /// Get the effective title for this inspection.
    fn effective_title(&self) -> String {
        self.title.clone().unwrap_or_else(|| "Object".to_string())
    }

    /// Determine if an attribute name should be visible based on filter settings.
    fn is_visible(&self, name: &str) -> bool {
        if self.all {
            return true;
        }
        if name.starts_with("__") && name.ends_with("__") {
            return self.dunder;
        }
        if name.starts_with('_') {
            return self.private;
        }
        true
    }
}

impl Renderable for Inspect {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        #[cfg(feature = "syntax-highlighting")]
        let highlighter = ReprHighlighter::new();
        let mut segments: Vec<Segment> = Vec::new();
        let mut items: Vec<Box<dyn Renderable>> = Vec::new();

        // -- Build the styled title --
        let title_style = Style::new().bold(true);
        let title_text = title_style.render(&self.effective_title());
        segments.push(Segment::line());
        segments.push(Segment::new(title_text));

        // -- Value representation --
        if self.value {
            #[cfg(feature = "syntax-highlighting")]
            {
                let highlighted = highlighter.highlight_str(&self.value_repr);
                let rendered = highlighted.render();
                segments.push(Segment::new(rendered));
            }
            #[cfg(not(feature = "syntax-highlighting"))]
            {
                segments.push(Segment::new(&self.value_repr));
            }
            segments.push(Segment::line());
        }

        // -- Doc text --
        if self.docs {
            if let Some(ref doc) = self.doc_text {
                segments.push(Segment::line());
                let doc_style = Style::new()
                    .italic(true)
                    .color(Color::parse("bright_black").unwrap_or_else(|_| Color::default()));
                for line in doc.lines() {
                    segments.push(Segment::styled(format!("  {line}"), doc_style.clone()));
                    segments.push(Segment::line());
                }
            }
        }

        // -- Attributes table --
        let visible_attrs: Vec<_> = self
            .attrs
            .iter()
            .filter(|(name, _, _, _)| self.is_visible(name))
            .collect();

        if !visible_attrs.is_empty() {
            let mut sorted_attrs: Vec<_> = visible_attrs.iter().collect();
            if self.sort {
                sorted_attrs.sort_by(|a, b| {
                    let a_name = a.0.trim_start_matches('_');
                    let b_name = b.0.trim_start_matches('_');
                    a_name.to_lowercase().cmp(&b_name.to_lowercase())
                });
            }

            let mut table = Table::new();
            table.show_header = true;
            table.show_edge = false;
            table.show_lines = false;
            table.add_column(
                Column::new("Attribute").header_style(
                    Style::new()
                        .bold(true)
                        .color(Color::parse("bright_cyan").unwrap_or_else(|_| Color::default())),
                ),
            );
            table.add_column(
                Column::new("Type").header_style(
                    Style::new()
                        .bold(true)
                        .color(Color::parse("bright_green").unwrap_or_else(|_| Color::default())),
                ),
            );
            table.add_column(
                Column::new("Value").header_style(
                    Style::new()
                        .bold(true)
                        .color(Color::parse("bright_yellow").unwrap_or_else(|_| Color::default())),
                ),
            );

            for (name, type_name, value_repr, _doc) in &sorted_attrs {
                let name_style =
                    Style::new().color(Color::parse("cyan").unwrap_or_else(|_| Color::default()));
                let type_style = Style::new()
                    .color(Color::parse("green").unwrap_or_else(|_| Color::default()))
                    .italic(true);

                let name_text = name_style.render(name);
                let type_text = type_style.render(type_name);
                #[cfg(feature = "syntax-highlighting")]
                let val_text = highlighter.highlight_str(value_repr).render();
                #[cfg(not(feature = "syntax-highlighting"))]
                let val_text = (*value_repr).to_string();

                table.add_row(vec![
                    crate::table::Cell::new(name_text),
                    crate::table::Cell::new(type_text),
                    crate::table::Cell::new(val_text),
                ]);
            }

            items.push(Box::new(table));
        }

        // -- Methods table --
        if self.methods && !self.method_list.is_empty() {
            let mut table = Table::new();
            table.show_header = true;
            table.show_edge = false;
            table.show_lines = false;
            table.add_column(
                Column::new("Method").header_style(
                    Style::new()
                        .bold(true)
                        .color(Color::parse("bright_magenta").unwrap_or_else(|_| Color::default())),
                ),
            );
            table.add_column(
                Column::new("Signature").header_style(
                    Style::new()
                        .bold(true)
                        .color(Color::parse("bright_blue").unwrap_or_else(|_| Color::default())),
                ),
            );

            for (name, sig, _doc) in &self.method_list {
                let name_style = Style::new()
                    .bold(true)
                    .color(Color::parse("magenta").unwrap_or_else(|_| Color::default()));
                let sig_style = Style::new().italic(true);

                let name_text = name_style.render(name);
                let sig_text = sig_style.render(sig);

                table.add_row(vec![
                    crate::table::Cell::new(name_text),
                    crate::table::Cell::new(sig_text),
                ]);
            }

            items.push(Box::new(table));
        }

        // -- Collect all rendered lines --
        let mut all_lines: Vec<Vec<Segment>> = Vec::new();

        // Add intro segments
        if !segments.is_empty() {
            all_lines.push(segments);
        }

        // Render child tables
        for item in &items {
            let result = item.render(options);
            all_lines.extend(result.lines);
        }

        // -- Wrap in a Panel --
        // Build the panel content from the collected segments
        let panel_content = all_lines
            .into_iter()
            .flatten()
            .map(|s| s.text)
            .collect::<Vec<_>>()
            .join("\n");

        let panel = Panel::new(panel_content)
            .title(self.effective_title())
            .border_style(
                Style::new()
                    .color(Color::parse("bright_blue").unwrap_or_else(|_| Color::default())),
            );

        panel.render(options)
    }
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

/// Create an `Inspect` for a value that implements `Debug`.
///
/// This is the Rust equivalent of Python Rich's `inspect()` function.
pub fn inspect(value: &dyn std::fmt::Debug) -> Inspect {
    Inspect::new(value)
}

/// Create an `Inspect` from a string value with a custom title.
pub fn inspect_str(title: impl Into<String>, value: impl Into<String>) -> Inspect {
    Inspect::from_str(value).title(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_new() {
        let val = vec![1, 2, 3];
        let insp = Inspect::new(&val);
        assert!(insp.value_repr.contains("1"));
        assert!(insp.value_repr.contains("2"));
        assert!(insp.value_repr.contains("3"));
    }

    #[test]
    fn test_inspect_title() {
        let insp = Inspect::from_str("test_value").title("MyStruct");
        assert_eq!(insp.effective_title(), "MyStruct");
    }

    #[test]
    fn test_inspect_add_attr() {
        let insp = Inspect::from_str("test")
            .add_attr("name", "String", "\"hello\"")
            .add_attr("count", "i32", "42");
        assert_eq!(insp.attrs.len(), 2);
        assert_eq!(insp.attrs[0].0, "name");
        assert_eq!(insp.attrs[1].0, "count");
    }

    #[test]
    fn test_inspect_add_method() {
        let insp =
            Inspect::from_str("test").add_method("do_thing", "fn do_thing(&self, x: i32) -> bool");
        assert_eq!(insp.method_list.len(), 1);
        assert_eq!(insp.method_list[0].0, "do_thing");
    }

    #[test]
    fn test_inspect_visibility() {
        let insp = Inspect::from_str("test");
        assert!(!insp.is_visible("_private"));
        assert!(insp.is_visible("public"));

        let insp2 = Inspect::from_str("test").private(true);
        assert!(insp2.is_visible("_private"));
        assert!(!insp2.is_visible("__dunder__"));

        let insp3 = Inspect::from_str("test").dunder(true);
        assert!(insp3.is_visible("__dunder__"));
    }

    #[test]
    fn test_inspect_all() {
        let insp = Inspect::from_str("test").all(true);
        assert!(insp.is_visible("_private"));
        assert!(insp.is_visible("__dunder__"));
        assert!(insp.is_visible("public"));
    }

    #[test]
    fn test_inspect_render() {
        let insp = Inspect::from_str("hello world")
            .title("TestObject")
            .add_attr("field1", "String", "\"value1\"")
            .add_attr("field2", "u64", "42");
        let opts = ConsoleOptions::default();
        let result = insp.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("TestObject"));
        assert!(ansi.contains("field1"));
        assert!(ansi.contains("field2"));
    }

    #[test]
    fn test_inspect_with_methods() {
        let insp = Inspect::from_str("obj")
            .title("MyType")
            .methods(true)
            .add_method("run", "fn run(&mut self) -> Result<()>")
            .add_method("stop", "fn stop(&self)");
        let opts = ConsoleOptions::default();
        let result = insp.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("MyType"));
        assert!(ansi.contains("run"));
        assert!(ansi.contains("stop"));
    }

    #[test]
    fn test_inspect_sorting() {
        let insp = Inspect::from_str("obj")
            .add_attr("zebra", "i32", "1")
            .add_attr("alpha", "i32", "2")
            .add_attr("beta", "i32", "3");
        let opts = ConsoleOptions::default();
        let result = insp.render(&opts);
        // Verify it renders without panicking (sorting is internal)
        let ansi = result.to_ansi();
        assert!(!ansi.is_empty());
    }
}
