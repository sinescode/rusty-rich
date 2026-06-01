//! Tree — hierarchical tree rendering. Equivalent to Rich's `tree.py`.

use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Guide types
// ---------------------------------------------------------------------------

/// The characters used for tree guide lines at each position.
#[derive(Debug, Clone)]
pub struct TreeGuides {
    /// Space guide (no line).
    pub space: &'static str,
    /// Continue guide (vertical line).
    pub continue_line: &'static str,
    /// Fork guide (branch with more siblings).
    pub fork: &'static str,
    /// End guide (last sibling).
    pub end: &'static str,
}

/// ASCII-only guides.
pub const ASCII_GUIDES: TreeGuides = TreeGuides {
    space: "    ",
    continue_line: "|   ",
    fork: "+-- ",
    end: "`-- ",
};

/// Default Unicode guides (like Rich's `TREE_GUIDES[0]`).
pub const TREE_GUIDES: TreeGuides = TreeGuides {
    space: "    ",
    continue_line: "│   ",
    fork: "├── ",
    end: "└── ",
};

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

/// A renderable for a tree structure.
#[derive(Debug, Clone)]
pub struct Tree {
    /// The label for this node.
    pub label: String,
    /// Style for this node's label.
    pub style: Style,
    /// Style for the guide lines.
    pub guide_style: Style,
    /// If true, children are visible.
    pub expanded: bool,
    /// If true, highlight string labels.
    pub highlight: bool,
    /// If true, don't show the root node.
    pub hide_root: bool,
    /// Children of this node.
    pub children: Vec<Tree>,
}

impl Tree {
    /// Create a new tree node with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: Style::new(),
            guide_style: Style::new(),
            expanded: true,
            highlight: false,
            hide_root: false,
            children: Vec::new(),
        }
    }

    /// Add a child node, returning a mutable reference to it.
    pub fn add(&mut self, label: impl Into<String>) -> &mut Tree {
        let child = Tree::new(label);
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Set the style for this node.
    pub fn style(mut self, style: Style) -> Self { self.style = style; self }

    /// Set the guide style.
    pub fn guide_style(mut self, style: Style) -> Self { self.guide_style = style; self }

    /// Hide the root node.
    pub fn hide_root(mut self) -> Self { self.hide_root = true; self }
}

impl Renderable for Tree {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let guides = if options.ascii_only { &ASCII_GUIDES } else { &TREE_GUIDES };
        let mut lines: Vec<Vec<Segment>> = Vec::new();

        if !self.hide_root {
            lines.push(vec![Segment::new(&self.label), Segment::line()]);
        }

        let last_idx = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().enumerate() {
            let is_last = i == last_idx;
            self.render_node(child, &mut lines, guides, "", is_last, options);
        }

        RenderResult { lines, items: Vec::new() }
    }
}

impl Tree {
    fn render_node(
        &self,
        node: &Tree,
        lines: &mut Vec<Vec<Segment>>,
        guides: &TreeGuides,
        prefix: &str,
        is_last: bool,
        options: &ConsoleOptions,
    ) {
        let connector = if is_last { guides.end } else { guides.fork };
        let guide_ansi = self.guide_style.to_ansi();
        let guide_reset = if guide_ansi.is_empty() { "" } else { "\x1b[0m" };

        // Render this node
        let guide_str = format!("{prefix}{connector}");
        lines.push(vec![
            Segment::new(format!("{guide_ansi}{guide_str}{guide_reset}")),
            Segment::new(&node.label),
            Segment::line(),
        ]);

        // Children continuation prefix
        let child_prefix = if is_last {
            format!("{prefix}{}", guides.space)
        } else {
            format!("{prefix}{}", guides.continue_line)
        };
        let child_prefix_styled = format!("{guide_ansi}{child_prefix}{guide_reset}");

        let last_child = node.children.len().saturating_sub(1);
        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == last_child;
            self.render_node(child, lines, guides, &child_prefix_styled, child_is_last, options);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

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
        tree.add("B");

        let opts = ConsoleOptions::default();
        let result = tree.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("A.1"));
    }
}
