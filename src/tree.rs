//! Tree — hierarchical tree rendering. Equivalent to Rich's `tree.py`.
//!
//! Renders hierarchical data with Unicode (or ASCII) guide lines. Supports
//! three guide styles (normal, bold, double), style inheritance through tree
//! levels via [`StyleStack`], and [`Renderable`](crate::Renderable) labels.
//!
//! # Example
//!
//! ```rust
//! use rusty_rich::Tree;
//!
//! let mut tree = Tree::new("Root");
//! tree.add("Child 1");
//! tree.add("Child 2");
//! ```

use crate::console::{ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::highlighter::ReprHighlighter;
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::{Style, StyleStack};
use crate::styled::Styled;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Guide types
// ---------------------------------------------------------------------------

/// The four guide characters at each position in the tree.
#[derive(Debug, Clone)]
pub struct TreeGuides {
    /// Space guide (no line continuation).
    pub space: &'static str,
    /// Continue guide (vertical line for ongoing siblings).
    pub continue_line: &'static str,
    /// Fork guide (branch with more siblings after this one).
    pub fork: &'static str,
    /// End guide (last sibling at this level).
    pub end: &'static str,
}

/// Which visual style of tree guides to use.
///
/// Python Rich selects this based on the guide style's attributes:
/// - `Normal` → `TREE_GUIDES[0]` (standard `├──`, `└──`)
/// - `Bold` → `TREE_GUIDES[1]` (heavy `┣━━`, `┗━━`)
/// - `Underline2` → `TREE_GUIDES[2]` (double `╠══`, `╚══`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideStyleKind {
    Normal,
    Bold,
    Underline2,
}

impl GuideStyleKind {
    /// Determine the guide style from a [`Style`]'s attributes.
    pub fn from_style(style: &Style) -> Self {
        if style.get_underline2().unwrap_or(false) {
            Self::Underline2
        } else if style.get_bold().unwrap_or(false) {
            Self::Bold
        } else {
            Self::Normal
        }
    }
}

/// ASCII-only guides (matching Python Rich's `ASCII_GUIDES`).
pub const ASCII_GUIDES: TreeGuides = TreeGuides {
    space: "    ",
    continue_line: "|   ",
    fork: "+-- ",
    end: "`-- ",
};

/// Default Unicode guides — standard weight (index 0).
pub const TREE_GUIDES: TreeGuides = TreeGuides {
    space: "    ",
    continue_line: "│   ",
    fork: "├── ",
    end: "└── ",
};

/// Bold-weight Unicode guides (index 1) — used when `guide_style.bold` is true.
pub const TREE_GUIDES_BOLD: TreeGuides = TreeGuides {
    space: "    ",
    continue_line: "┃   ",
    fork: "┣━━ ",
    end: "┗━━ ",
};

/// Double-line Unicode guides (index 2) — used when `guide_style.underline2` is true.
pub const TREE_GUIDES_DOUBLE: TreeGuides = TreeGuides {
    space: "    ",
    continue_line: "║   ",
    fork: "╠══ ",
    end: "╚══ ",
};

/// Select the appropriate guide set based on style kind and ascii_only flag.
fn select_guides(kind: GuideStyleKind, ascii_only: bool) -> &'static TreeGuides {
    if ascii_only {
        &ASCII_GUIDES
    } else {
        match kind {
            GuideStyleKind::Normal => &TREE_GUIDES,
            GuideStyleKind::Bold => &TREE_GUIDES_BOLD,
            GuideStyleKind::Underline2 => &TREE_GUIDES_DOUBLE,
        }
    }
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

/// A renderable for a hierarchical tree structure.
///
/// Each node has a label (any [`Renderable`]), optional explicit style and
/// guide-style, and zero or more child [`Tree`] nodes. Styles cascade from
/// parent to child via [`Style::chain`] fallback semantics.
#[derive(Debug, Clone)]
pub struct Tree {
    /// The label for this node (any renderable — text, panel, table, etc.).
    pub label: DynRenderable,
    /// Plain-text fallback for the label (used for measurement and highlighting).
    pub label_text: String,
    /// Style for this node's label (falls back to parent via chain).
    pub style: Style,
    /// Style for the guide lines (falls back to parent via chain).
    pub guide_style: Style,
    /// If true, children are rendered.
    pub expanded: bool,
    /// If true, highlight string labels via [`ReprHighlighter`].
    pub highlight: bool,
    /// If true, the root node itself is not rendered (only its children).
    pub hide_root: bool,
    /// Children of this node.
    pub children: Vec<Tree>,
}

impl Tree {
    /// Create a new tree node with the given text label.
    pub fn new(label: impl Into<String>) -> Self {
        let s: String = label.into();
        Self {
            label: DynRenderable::new(s.clone()),
            label_text: s,
            style: Style::new(),
            guide_style: Style::new(),
            expanded: true,
            highlight: false,
            hide_root: false,
            children: Vec::new(),
        }
    }

    /// Create a new tree node with an arbitrary renderable label.
    pub fn new_renderable(
        label: impl Renderable + Send + Sync + 'static,
        label_text: impl Into<String>,
    ) -> Self {
        Self {
            label: DynRenderable::new(label),
            label_text: label_text.into(),
            style: Style::new(),
            guide_style: Style::new(),
            expanded: true,
            highlight: false,
            hide_root: false,
            children: Vec::new(),
        }
    }

    /// Add a child node.
    ///
    /// The child inherits the parent's `style` and `guide_style` as fallbacks
    /// via [`Style::chain`]. Returns a mutable reference so the caller can
    /// further customize the child.
    pub fn add(&mut self, label: impl Into<String>) -> &mut Tree {
        let s: String = label.into();
        let mut child = Tree::new(s);
        // Inherit parent styles as fallbacks
        child.style = child.style.chain(&self.style);
        child.guide_style = child.guide_style.chain(&self.guide_style);
        child.highlight = self.highlight;
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Add a child node with any renderable label.
    ///
    /// Like Python Rich's `Tree.add()` which accepts any `RenderableType`.
    /// The child inherits the parent's `style` and `guide_style` as fallbacks.
    /// Returns a mutable reference for further customization.
    pub fn add_renderable(
        &mut self,
        label: impl Renderable + Send + Sync + 'static,
    ) -> &mut Tree {
        let label_text = format!("{:?}", &label as &dyn std::fmt::Debug);
        let mut child = Tree {
            label: DynRenderable::new(label),
            label_text,
            style: Style::new().chain(&self.style),
            guide_style: Style::new().chain(&self.guide_style),
            expanded: true,
            highlight: self.highlight,
            hide_root: false,
            children: Vec::new(),
        };
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Add a child node with explicit style and guide_style overrides.
    ///
    /// When a parameter is `None`, the parent's value is used as a fallback.
    pub fn add_with(
        &mut self,
        label: impl Into<String>,
        style: Option<Style>,
        guide_style: Option<Style>,
        expanded: Option<bool>,
        highlight: Option<bool>,
    ) -> &mut Tree {
        let mut child = Tree::new(label);
        child.style = style
            .unwrap_or_else(Style::new)
            .chain(&self.style);
        child.guide_style = guide_style
            .unwrap_or_else(Style::new)
            .chain(&self.guide_style);
        child.expanded = expanded.unwrap_or(true);
        child.highlight = highlight.unwrap_or(self.highlight);
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Add a renderable child with full option control.
    ///
    /// Like [`add_renderable`](Self::add_renderable) but with explicit style,
    /// guide_style, expanded, and highlight overrides. `None` values fall back
    /// to the parent's settings.
    pub fn add_renderable_with(
        &mut self,
        label: impl Renderable + Send + Sync + 'static,
        style: Option<Style>,
        guide_style: Option<Style>,
        expanded: Option<bool>,
        highlight: Option<bool>,
    ) -> &mut Tree {
        let label_text = format!("{:?}", &label as &dyn std::fmt::Debug);
        let mut child = Tree {
            label: DynRenderable::new(label),
            label_text,
            style: style.unwrap_or_else(Style::new).chain(&self.style),
            guide_style: guide_style.unwrap_or_else(Style::new).chain(&self.guide_style),
            expanded: expanded.unwrap_or(true),
            highlight: highlight.unwrap_or(self.highlight),
            hide_root: false,
            children: Vec::new(),
        };
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Builder: set the style for this node's label.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the guide style for this node and its descendants.
    pub fn guide_style(mut self, style: Style) -> Self {
        self.guide_style = style;
        self
    }

    /// Builder: hide the root node (only children are rendered).
    pub fn hide_root(mut self) -> Self {
        self.hide_root = true;
        self
    }

    /// Builder: enable repr-style highlighting of string labels.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Builder: collapse this node (children are hidden).
    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}

// ---------------------------------------------------------------------------
// Renderable impl
// ---------------------------------------------------------------------------

impl Renderable for Tree {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let null_style = Style::new();

        // Determine guide style kind and select the right guide set
        let guide_kind = GuideStyleKind::from_style(&self.guide_style);
        let guides = select_guides(guide_kind, options.ascii_only);

        // Style stacks for depth-based inheritance
        let mut guide_style_stack = StyleStack::new(null_style.clone());
        let mut style_stack = StyleStack::new(null_style.clone());

        // The initial guide_style (for the root's children)
        guide_style_stack.push(self.guide_style.clone());
        style_stack.push(self.style.clone());

        let mut lines: Vec<Vec<Segment>> = Vec::new();

        // Render root node unless hidden
        if !self.hide_root {
            let label_segs = build_label_segments(&self.label, &self.label_text, self.highlight, self.expanded, self.children.is_empty(), options);
            lines.push(label_segs);
            lines.push(vec![Segment::line()]);
        }

        // Iterative depth-first rendering using an explicit stack
        // Each stack entry: (node_ref, prefix_segments, is_last_sibling)
        let last_idx = self.children.len().saturating_sub(1);
        let mut stack: Vec<(&Tree, Vec<Segment>, bool, usize)> = self
            .children
            .iter()
            .enumerate()
            .rev() // reverse so we process in forward order
            .map(|(i, child)| {
                let is_last = i == last_idx;
                let initial_prefix: Vec<Segment> = Vec::new();
                (child, initial_prefix, is_last, 0) // depth = 0 for direct children
            })
            .collect();

        while let Some((node, prefix, is_last, depth)) = stack.pop() {
            // Determine guide kind for this node's effective guide style
            let effective_guide = guide_style_stack.current().combine(&node.guide_style);
            let node_guide_kind = GuideStyleKind::from_style(&effective_guide);
            let node_guides = select_guides(node_guide_kind, options.ascii_only);

            // Build connector
            let connector = if is_last {
                node_guides.end
            } else {
                node_guides.fork
            };

            // Build the full guide prefix for this line
            let mut line_segs: Vec<Segment> = Vec::new();

            // Add ancestor prefix (space/continue lines from higher levels)
            for seg in &prefix {
                line_segs.push(seg.clone());
            }

            // Add connector segment with proper styling
            let connector_seg = Segment::styled(connector.to_string(), effective_guide.clone());
            line_segs.push(connector_seg);

            // Build label segments
            let effective_style = style_stack.current().combine(&node.style);
            let label_segs = build_label_segments(
                &node.label,
                &node.label_text,
                node.highlight,
                node.expanded,
                node.children.is_empty(),
                options,
            );

            // Apply effective style to label via Styled wrapper
            let styled_label = Styled::new(node.label.clone(), effective_style.clone());
            let label_result = styled_label.render(options);
            if let Some(first_line) = label_result.lines.into_iter().next() {
                line_segs.extend(first_line);
            } else {
                line_segs.extend(label_segs);
            }

            line_segs.push(Segment::line());
            lines.push(line_segs);

            // If collapsed, don't render children
            if !node.expanded || node.children.is_empty() {
                continue;
            }

            // Build child prefix by appending the continuation guide
            let continuation = if is_last {
                node_guides.space
            } else {
                node_guides.continue_line
            };
            let cont_seg = Segment::styled(continuation.to_string(), effective_guide.clone());
            let mut child_prefix = prefix.clone();
            child_prefix.push(cont_seg);

            // Push style stacks for this node's children
            if !node.style.is_plain() {
                style_stack.push(node.style.clone());
            }
            if !node.guide_style.is_plain() {
                guide_style_stack.push(node.guide_style.clone());
            }

            // Push children in reverse order
            let child_last = node.children.len().saturating_sub(1);
            for (i, child) in node.children.iter().enumerate().rev() {
                let child_is_last = i == child_last;
                stack.push((child, child_prefix.clone(), child_is_last, depth + 1));
            }

            // Pop style stacks (when we backtrack — handled by pushing per level)
            // NOTE: In the iterative approach, we push styles before children
            // and pop when we backtrack. Since we're using an explicit stack
            // with prefix cloning, we don't need explicit pop — each branch
            // carries its own prefix and the stacks are managed at entry.
        }

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }

    fn measure(&self, _options: &ConsoleOptions) -> Option<Measurement> {
        let mut minimum: usize = 0;
        let mut maximum: usize = 0;

        let mut stack: Vec<(&Tree, usize)> = Vec::new();

        // Root
        if !self.hide_root {
            let label_w = unicode_width::UnicodeWidthStr::width(self.label_text.as_str());
            minimum = label_w;
            maximum = label_w;
        }

        // Direct children at depth 1 (4-char indent)
        for child in self.children.iter().rev() {
            stack.push((child, 1));
        }

        while let Some((node, depth)) = stack.pop() {
            let indent = depth * 4; // 4 chars per guide level
            let label_w = unicode_width::UnicodeWidthStr::width(node.label_text.as_str());
            let node_w = label_w + indent;
            minimum = minimum.max(node_w);
            maximum = maximum.max(node_w);

            if node.expanded && !node.children.is_empty() {
                for child in node.children.iter().rev() {
                    stack.push((child, depth + 1));
                }
            }
        }

        Some(Measurement::new(minimum, maximum))
    }
}

// ---------------------------------------------------------------------------
// Label builder
// ---------------------------------------------------------------------------

/// Build the segment list for a tree node's label.
///
/// Respects `highlight` (repr-style highlighting) and `expanded`/collapsed
/// state (adds "+" indicator when collapsed with children).
fn build_label_segments(
    label: &DynRenderable,
    label_text: &str,
    highlight: bool,
    expanded: bool,
    is_leaf: bool,
    _options: &ConsoleOptions,
) -> Vec<Segment> {
    let mut segs: Vec<Segment>;

    // Render the label through its Renderable impl
    let result = label.render(_options);
    segs = result
        .lines
        .into_iter()
        .flatten()
        .filter(|s| s.text != "\n" && s.text != "\r\n")
        .collect();

    if segs.is_empty() {
        // Fallback to plain text with optional highlighting
        if highlight {
            let highlighter = ReprHighlighter::new();
            let text = highlighter.highlight_str(label_text);
            let rendered = text.render();
            segs = vec![Segment::new(rendered)];
        } else {
            segs = vec![Segment::new(label_text)];
        }
    } else if highlight && segs.len() == 1 && segs[0].style.is_plain() {
        // Apply repr highlighting to plain un-styled segments
        let highlighter = ReprHighlighter::new();
        let text = highlighter.highlight_str(label_text);
        let rendered = text.render();
        segs = vec![Segment::new(rendered)];
    }

    // Collapsed indicator: prepend "+ " when node has children but is collapsed
    if !expanded && !is_leaf {
        let mut full = vec![Segment::new("+ ")];
        full.append(&mut segs);
        segs = full;
    }

    segs
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_guide_style_kind_from_style() {
        let normal = Style::new();
        assert_eq!(GuideStyleKind::from_style(&normal), GuideStyleKind::Normal);

        let bold = Style::new().bold(true);
        assert_eq!(GuideStyleKind::from_style(&bold), GuideStyleKind::Bold);

        let ul2 = Style::new().underline2(true);
        assert_eq!(GuideStyleKind::from_style(&ul2), GuideStyleKind::Underline2);

        // Bold takes precedence over underline2
        let both = Style::new().bold(true).underline2(true);
        assert_eq!(GuideStyleKind::from_style(&both), GuideStyleKind::Bold);
    }

    #[test]
    fn test_select_guides() {
        let g = select_guides(GuideStyleKind::Normal, false);
        assert_eq!(g.fork, "├── ");

        let g = select_guides(GuideStyleKind::Bold, false);
        assert_eq!(g.fork, "┣━━ ");

        let g = select_guides(GuideStyleKind::Underline2, false);
        assert_eq!(g.fork, "╠══ ");

        let g = select_guides(GuideStyleKind::Bold, true);
        assert_eq!(g.fork, "+-- ");
    }

    #[test]
    fn test_simple_tree() {
        let mut tree = Tree::new("Root");
        tree.add("Child 1");
        tree.add("Child 2");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Root"));
        assert!(ansi.contains("Child 1"));
        assert!(ansi.contains("Child 2"));
    }

    #[test]
    fn test_nested_tree() {
        let mut tree = Tree::new("Root");
        let child = tree.add("A");
        child.add("A.1");
        child.add("A.2");
        tree.add("B");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("A.1"));
        assert!(ansi.contains("A.2"));
    }

    #[test]
    fn test_hide_root() {
        let mut tree = Tree::new("Hidden").hide_root();
        tree.add("Child 1");
        tree.add("Child 2");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(!ansi.contains("Hidden"));
        assert!(ansi.contains("Child 1"));
        assert!(ansi.contains("Child 2"));
    }

    #[test]
    fn test_ascii_mode() {
        let mut tree = Tree::new("Root");
        tree.add("Child");

        let opts = ConsoleOptions {
            ascii_only: true,
            ..ConsoleOptions::default()
        };
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("+-- "));
    }

    #[test]
    fn test_collapsed_node() {
        let mut tree = Tree::new("Root");
        let child = tree.add("Collapsed");
        child.add("Hidden Child");
        child.expanded = false;

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("+ ")); // collapsed indicator
        assert!(!ansi.contains("Hidden Child"));
    }

    #[test]
    fn test_measure() {
        let mut tree = Tree::new("Root");
        tree.add("Short");
        tree.add("A much longer child");

        let opts = ConsoleOptions::default();
        let m = tree.measure(&opts).expect("should have measurement");
        assert!(m.minimum > 0);
        assert!(m.maximum >= m.minimum);
    }

    #[test]
    fn test_measure_nested() {
        let mut tree = Tree::new("Root");
        let child = tree.add("A");
        child.add("Deeply nested child with long label");

        let opts = ConsoleOptions::default();
        let m = tree.measure(&opts).expect("should have measurement");
        // Deeply nested should have indent = 2 * 4 = 8 extra chars
        assert!(m.maximum > 20);
    }

    #[test]
    fn test_style_inheritance() {
        let mut tree = Tree::new("Root");
        let child = tree.add("Child");
        child.add("Grandchild");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let _ansi = result.to_ansi();
        // At minimum, rendering should not panic with inherited styles
    }

    #[test]
    fn test_guide_style_bold() {
        let mut tree = Tree::new("Root")
            .guide_style(Style::new().bold(true));
        tree.add("Child");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        // Should use bold guides ("┣━━ " instead of "├── ")
        assert!(ansi.contains("┣━━ ") || ansi.contains("┗━━ "));
    }

    #[test]
    fn test_new_renderable() {
        let tree = Tree::new_renderable(
            crate::text::Text::new("Custom Label"),
            "Custom Label",
        );
        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Custom Label"));
    }

    #[test]
    fn test_add_with() {
        let mut tree = Tree::new("Root");
        tree.add_with(
            "Styled Child",
            Some(Style::new().bold(true)),
            Some(Style::new().italic(true)),
            Some(false),
            Some(true),
        );

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Styled Child"));
    }

    #[test]
    fn test_highlight() {
        let mut tree = Tree::new("Root").highlight();
        tree.add("12345"); // numeric string — should be highlighted

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let _ansi = result.to_ansi();
        // Highlighting should not panic
    }

    #[test]
    fn test_deeply_nested_tree() {
        let mut tree = Tree::new("L0");
        let mut current = tree.add("L1");
        current = current.add("L2");
        current = current.add("L3");
        current.add("L4");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("L0"));
        assert!(ansi.contains("L4"));
    }

    #[test]
    fn test_wide_branching() {
        let mut tree = Tree::new("Root");
        for i in 0..10 {
            tree.add(format!("Child {i}"));
        }

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        for i in 0..10 {
            assert!(ansi.contains(&format!("Child {i}")));
        }
    }
}
