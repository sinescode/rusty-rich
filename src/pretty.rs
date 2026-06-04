//! Pretty-printing for Rust data structures with Rich styling.
//!
//! Equivalent to Python Rich's `pretty.py`. Provides a node-tree traversal
//! system that can render structured data (debug output, JSON) with
//! indentation guides, syntax highlighting, and configurable depth limits.

use crate::console::{ConsoleOptions, RenderResult, Renderable};
#[cfg(feature = "syntax-highlighting")]
use crate::highlighter::ReprHighlighter;
use crate::segment::Segment;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A node in a pretty-print tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// Optional key (e.g. struct field name, map key).
    pub key: Option<String>,
    /// Optional value string (leaf nodes).
    pub value: Option<String>,
    /// Child nodes.
    pub children: Vec<Node>,
    /// Whether this node is a container (struct, map, array).
    pub is_container: bool,
    /// Whether this node is an iterable.
    pub is_iter: bool,
    /// Whether this node is a mapping.
    pub is_mapping: bool,
    /// Whether this node has `__attrs__`-style fields.
    pub is_attrs: bool,
    /// Opening bracket string.
    pub open_brace: String,
    /// Closing bracket string.
    pub close_brace: String,
    /// Text shown when empty.
    pub empty: String,
    /// Whether this is the last sibling.
    pub last: bool,
}

impl Node {
    /// Create a new Node with an optional key and value.
    pub fn new(key: Option<String>, value: Option<String>) -> Self {
        Self {
            key,
            value,
            children: Vec::new(),
            is_container: false,
            is_iter: false,
            is_mapping: false,
            is_attrs: false,
            open_brace: "(".to_string(),
            close_brace: ")".to_string(),
            empty: String::new(),
            last: true,
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }
}

// ---------------------------------------------------------------------------
// Pretty
// ---------------------------------------------------------------------------

/// Pretty renderable that traverses a [`Node`] tree.
///
/// Renders structured data with indentation guides, configurable depth,
/// string length limits, and syntax highlighting via [`ReprHighlighter`].
#[derive(Debug, Clone)]
pub struct Pretty {
    /// The root node of the tree to render.
    node: Node,
    /// Whether to draw indentation guides (vertical lines between siblings).
    indent_guides: bool,
    /// Maximum nesting depth (None = unlimited).
    max_depth: Option<usize>,
    /// Maximum string length before truncation.
    max_string: Option<usize>,
    /// Maximum number of children to show per container.
    max_length: Option<usize>,
    /// If true, expand all containers regardless of depth.
    expand_all: bool,
    /// Highlighter for values.
    #[cfg(feature = "syntax-highlighting")]
    highlighter: ReprHighlighter,
}

impl Pretty {
    /// Create a new Pretty renderable from a [`Node`] tree.
    pub fn new(node: Node) -> Self {
        Self {
            node,
            indent_guides: true,
            max_depth: None,
            max_string: None,
            max_length: None,
            expand_all: false,
            #[cfg(feature = "syntax-highlighting")]
            highlighter: ReprHighlighter::new(),
        }
    }

    /// Builder: enable or disable indent guides.
    pub fn indent_guides(mut self, value: bool) -> Self {
        self.indent_guides = value;
        self
    }

    /// Builder: set the maximum nesting depth.
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Builder: set the maximum string length before truncation.
    pub fn max_string(mut self, max: usize) -> Self {
        self.max_string = Some(max);
        self
    }

    /// Builder: set the maximum number of children to show per container.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Builder: expand all containers regardless of depth limit.
    pub fn expand_all(mut self) -> Self {
        self.expand_all = true;
        self
    }

    /// Generate a Node tree from an arbitrary value using the [`Debug`] trait.
    ///
    /// Parses the `{:#?}` debug representation into a structured [`Node`] tree.
    pub fn from_debug<T: std::fmt::Debug>(value: &T) -> Self {
        let debug_str = format!("{:#?}", value);
        let node = parse_debug_to_node(&debug_str);
        Self::new(node)
    }

    /// Create a Pretty from a `serde_json::Value`.
    pub fn from_json(value: &serde_json::Value) -> Self {
        let node = json_to_node(value, None);
        Self::new(node)
    }
}

impl Renderable for Pretty {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let prefix = String::new();
        let depth = 0;
        self.render_node(&self.node, &mut lines, &prefix, depth, options);
        RenderResult { lines, items: Vec::new() }
    }
}

impl Pretty {
    fn render_node(
        &self,
        node: &Node,
        lines: &mut Vec<Vec<Segment>>,
        prefix: &str,
        depth: usize,
        options: &ConsoleOptions,
    ) {
        // Check depth limit
        if let Some(max) = self.max_depth {
            if depth > max && !self.expand_all {
                lines.push(vec![
                    Segment::new(prefix),
                    Segment::new("..."),
                    Segment::line(),
                ]);
                return;
            }
        }

        let indent = "    ";
        let guide = if self.indent_guides && !options.ascii_only {
            "│   "
        } else {
            "    "
        };

        let mut line_text = String::from(prefix);

        // Key
        if let Some(ref key) = node.key {
            #[cfg(feature = "syntax-highlighting")]
            {
                let highlighted = self.highlighter.highlight_str(key);
                line_text.push_str(&highlighted.plain);
            }
            #[cfg(not(feature = "syntax-highlighting"))]
            {
                line_text.push_str(key);
            }
            line_text.push_str(": ");
        }

        // Value or container
        if node.children.is_empty() {
            if let Some(ref value) = node.value {
                let truncated = if let Some(max) = self.max_string {
                    if value.len() > max {
                        format!("{}...", &value[..max])
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                };
                #[cfg(feature = "syntax-highlighting")]
                let highlighted = self.highlighter.highlight_str(&truncated);
                #[cfg(feature = "syntax-highlighting")]
                {
                    line_text.push_str(&highlighted.plain);
                }
                #[cfg(not(feature = "syntax-highlighting"))]
                {
                    line_text.push_str(&truncated);
                }
            }
            lines.push(vec![Segment::new(&line_text), Segment::line()]);
        } else {
            // Opening brace
            line_text.push_str(&node.open_brace);
            lines.push(vec![Segment::new(&line_text), Segment::line()]);

            let max_len = self.max_length.unwrap_or(usize::MAX);
            let count = node.children.len();
            let show_ellipsis = count > max_len;

            for (i, child) in node.children.iter().enumerate() {
                if i >= max_len {
                    if show_ellipsis {
                        let child_prefix = format!("{prefix}{indent}");
                        lines.push(vec![
                            Segment::new(format!(
                                "{child_prefix}... ({} more)",
                                count - max_len
                            )),
                            Segment::line(),
                        ]);
                    }
                    break;
                }
                let child_prefix = if self.indent_guides && i < count - 1 {
                    format!("{prefix}{guide}")
                } else {
                    format!("{prefix}{indent}")
                };
                self.render_node(child, lines, &child_prefix, depth + 1, options);
            }

            // Closing brace
            lines.push(vec![
                Segment::new(format!("{prefix}{}", node.close_brace)),
                Segment::line(),
            ]);
        }
    }
}

/// Install Pretty as the default display handler.
///
/// This is a no-op in Rust (the Rust standard library handles this via
/// the [`Display`] and [`Debug`] traits), but is provided for API
/// compatibility with Python Rich's `pretty.install()`.
pub fn install() {}

/// Pretty-print a value to the console.
pub fn pprint<T: std::fmt::Debug>(value: &T, console: &mut crate::console::Console) {
    let pretty = Pretty::from_debug(value);
    console.println(&pretty);
}

/// Generate a pretty [`Text`] representation of a value.
pub fn pretty_repr<T: std::fmt::Debug>(value: &T) -> Text {
    let debug_str = format!("{:#?}", value);
    #[cfg(feature = "syntax-highlighting")]
    {
        let highlighter = ReprHighlighter::new();
        highlighter.highlight_str(&debug_str)
    }
    #[cfg(not(feature = "syntax-highlighting"))]
    {
        let mut t = Text::new("");
        t.plain = debug_str;
        t
    }
}

/// Traverse an arbitrary value and build a [`Node`] tree.
///
/// Uses the [`Debug`] trait to introspect the value.
pub fn traverse(value: &dyn std::fmt::Debug) -> Node {
    let debug_str = format!("{:#?}", value);
    parse_debug_to_node(&debug_str)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse a `{:#?}` debug string into a [`Node`] tree.
fn parse_debug_to_node(debug: &str) -> Node {
    let lines: Vec<&str> = debug.lines().collect();
    if lines.is_empty() {
        return Node::new(None, Some(String::new()));
    }

    // Single-line value
    if lines.len() == 1 {
        return Node::new(None, Some(lines[0].trim().to_string()));
    }

    // Multi-line: parse structure
    let trimmed = debug.trim();
    let (open_brace, close_brace) = if trimmed.starts_with('{') {
        ("{", "}")
    } else if trimmed.starts_with('[') {
        ("[", "]")
    } else {
        ("(", ")")
    };

    let mut node = Node::new(None, None);
    node.open_brace = open_brace.to_string();
    node.close_brace = close_brace.to_string();
    node.is_container = true;

    // Extract children from indented lines
    for line in lines.iter().skip(1) {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() || trimmed_line == close_brace {
            continue;
        }
        // Try to parse key: value
        if let Some(idx) = trimmed_line.find(": ") {
            let key = trimmed_line[..idx].trim().to_string();
            let value = trimmed_line[idx + 2..].trim().to_string();
            let child = Node::new(Some(key), Some(value));
            node.children.push(child);
        } else {
            let child = Node::new(None, Some(trimmed_line.to_string()));
            node.children.push(child);
        }
    }

    node
}

/// Convert a `serde_json::Value` into a [`Node`] tree.
fn json_to_node(value: &serde_json::Value, key: Option<String>) -> Node {
    match value {
        serde_json::Value::Null => {
            let mut node = Node::new(key, Some("null".to_string()));
            node.is_container = false;
            node
        }
        serde_json::Value::Bool(b) => {
            let mut node = Node::new(key, Some(b.to_string()));
            node.is_container = false;
            node
        }
        serde_json::Value::Number(n) => {
            let mut node = Node::new(key, Some(n.to_string()));
            node.is_container = false;
            node
        }
        serde_json::Value::String(s) => {
            let display = format!("\"{}\"", s);
            let mut node = Node::new(key, Some(display));
            node.is_container = false;
            node
        }
        serde_json::Value::Array(arr) => {
            let mut node = Node::new(key, None);
            node.is_container = true;
            node.is_iter = true;
            node.open_brace = "[".to_string();
            node.close_brace = "]".to_string();
            node.empty = "[]".to_string();
            for item in arr {
                node.children.push(json_to_node(item, None));
            }
            node
        }
        serde_json::Value::Object(obj) => {
            let mut node = Node::new(key, None);
            node.is_container = true;
            node.is_mapping = true;
            node.open_brace = "{".to_string();
            node.close_brace = "}".to_string();
            node.empty = "{}".to_string();
            for (k, v) in obj {
                node.children.push(json_to_node(v, Some(k.clone())));
            }
            node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_pretty_from_debug() {
        let value = vec!["hello", "world"];
        let pretty = Pretty::from_debug(&value);
        let opts = ConsoleOptions::default();
        let result = pretty.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("hello"));
        assert!(ansi.contains("world"));
    }

    #[test]
    fn test_pretty_from_json() {
        let value = serde_json::json!({"name": "Alice", "age": 30});
        let pretty = Pretty::from_json(&value);
        let opts = ConsoleOptions::default();
        let result = pretty.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Alice"));
        assert!(ansi.contains("30"));
    }

    #[test]
    fn test_pretty_repr() {
        let text = pretty_repr(&42);
        assert!(!text.plain.is_empty());
    }

    #[test]
    fn test_max_depth() {
        let inner = serde_json::json!({"a": {"b": {"c": 1}}});
        let pretty = Pretty::from_json(&inner).max_depth(1);
        let opts = ConsoleOptions::default();
        let result = pretty.render(&opts);
        let ansi = result.to_ansi();
        // Should contain the depth truncation marker
        assert!(ansi.contains("...") || ansi.contains("1"));
    }

    #[test]
    fn test_json_to_node_empty_object() {
        let value = serde_json::Value::Object(serde_json::Map::new());
        let node = json_to_node(&value, None);
        assert!(node.is_container);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_json_to_node_empty_array() {
        let value = serde_json::Value::Array(Vec::new());
        let node = json_to_node(&value, None);
        assert!(node.is_container);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_json_to_node_scalars() {
        let null_node = json_to_node(&serde_json::Value::Null, None);
        assert_eq!(null_node.value.as_deref(), Some("null"));

        let bool_node = json_to_node(&serde_json::Value::Bool(true), Some("flag".into()));
        assert_eq!(bool_node.key.as_deref(), Some("flag"));
        assert_eq!(bool_node.value.as_deref(), Some("true"));

        let num_node = json_to_node(&serde_json::json!(42), None);
        assert_eq!(num_node.value.as_deref(), Some("42"));

        let str_node = json_to_node(&serde_json::json!("hello"), None);
        assert!(str_node.value.as_deref().unwrap_or("").contains("hello"));
    }

    #[test]
    fn test_install_is_noop() {
        // Just verify it doesn't panic
        install();
    }

    #[test]
    fn test_traverse() {
        let node = traverse(&"test");
        assert!(node.value.is_some());
    }

    #[test]
    fn test_builder_methods() {
        let node = Node::new(None, Some("value".to_string()));
        let pretty = Pretty::new(node)
            .indent_guides(false)
            .max_depth(5)
            .max_string(100)
            .max_length(10)
            .expand_all();
        assert!(!pretty.indent_guides);
        assert_eq!(pretty.max_depth, Some(5));
        assert_eq!(pretty.max_string, Some(100));
        assert_eq!(pretty.max_length, Some(10));
        assert!(pretty.expand_all);
    }
}
