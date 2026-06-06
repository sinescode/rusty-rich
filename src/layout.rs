//! Layout — split-pane layout system. Equivalent to Rich's `layout.py`.

use std::collections::HashMap;

use crate::console::{Console, ConsoleOptions, DynRenderable, RenderResult, Renderable};
use crate::segment::Segment;

// Re-export Region from the standalone region module
pub use crate::region::Region;

// ---------------------------------------------------------------------------
// Direction
// ---------------------------------------------------------------------------

/// Direction of a split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Split content side by side (left to right).
    Horizontal,
    /// Split content stacked (top to bottom).
    Vertical,
}

// ---------------------------------------------------------------------------
// LayoutNode
// ---------------------------------------------------------------------------

/// A layout node — can be a leaf (containing a renderable) or a split.
#[derive(Debug, Clone)]
pub enum LayoutNode {
    /// A split container with children and a direction.
    Split {
        /// Direction of the split (horizontal or vertical).
        direction: Direction,
        /// Relative size ratios for children.
        sizes: Vec<usize>,
        /// Child layout nodes.
        children: Vec<LayoutNode>,
    },
    /// A leaf with a renderable name (placeholder) and optional fixed size.
    Leaf {
        /// Name identifier for this leaf.
        name: String,
        /// Optional label for the renderable.
        renderable: Option<String>,
        /// Optional fixed size constraint.
        size: Option<usize>,
    },
}

impl LayoutNode {
    /// Create a new split node with equal-size children.
    ///
    /// Each child is assigned an initial ratio of 1. Use
    /// [`sizes`](LayoutNode::sizes) to customize the ratios.
    pub fn split(direction: Direction, children: Vec<LayoutNode>) -> Self {
        let sizes = vec![1; children.len()];
        Self::Split {
            direction,
            sizes,
            children,
        }
    }

    /// Builder: set the size ratios for the children of this split node.
    pub fn sizes(mut self, sizes: Vec<usize>) -> Self {
        if let Self::Split {
            sizes: ref mut s, ..
        } = self
        {
            *s = sizes;
        }
        self
    }
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

/// The Layout compute engine. Assigns screen regions to a tree of layout
/// nodes by recursively splitting available space.
#[derive(Debug)]
pub struct Layout {
    /// The root [`LayoutNode`] defining the split hierarchy.
    pub root: LayoutNode,
    /// Whether the layout is visible.
    pub visible: bool,
    /// Minimum size for any region.
    pub minimum_size: usize,
    /// Named renderables for leaf nodes.
    pub renderables: HashMap<String, DynRenderable>,
    /// Named splitters for dividing regions among children.
    /// When a `Split` node is encountered during layout computation,
    /// the engine looks up a splitter by child-name to determine how
    /// to partition the available space.  If no matching splitter is
    /// registered the default ratio-based partitioning is used.
    pub splitters: HashMap<String, Box<dyn Splitter>>,
    /// Auto-incrementing pane ID counter.
    next_pane_id: usize,
}

impl Layout {
    /// Create a new layout with the given root node.
    pub fn new(root: LayoutNode) -> Self {
        Self {
            root,
            visible: true,
            minimum_size: 1,
            renderables: HashMap::new(),
            splitters: HashMap::new(),
            next_pane_id: 0,
        }
    }

    /// Create a new layout from a named renderable (single-pane leaf).
    ///
    /// This is a convenience constructor that wraps the renderable in a leaf
    /// node with the given name.
    pub fn from_renderable(
        name: impl Into<String>,
        renderable: impl Renderable + Send + Sync + 'static,
    ) -> Self {
        let name = name.into();
        let node = LayoutNode::Leaf {
            name: name.clone(),
            renderable: None,
            size: None,
        };
        let mut renderables = HashMap::new();
        renderables.insert(name, DynRenderable::new(renderable));
        Self {
            root: node,
            visible: true,
            minimum_size: 1,
            renderables,
            splitters: HashMap::new(),
            next_pane_id: 1,
        }
    }

    /// Split the current root node into the given direction.
    ///
    /// The existing root becomes the first child of the new split, and a new
    /// empty leaf is added as the second child.
    /// Returns a mutable reference to the root node.
    pub fn split(&mut self, direction: Direction) -> &mut LayoutNode {
        let name_a = format!("_split_a_{}", self.next_pane_id);
        let name_b = format!("_split_b_{}", self.next_pane_id + 1);
        self.next_pane_id += 2;

        let old_root = std::mem::replace(
            &mut self.root,
            LayoutNode::Split {
                direction,
                sizes: vec![1, 1],
                children: vec![
                    LayoutNode::Leaf {
                        name: name_a,
                        renderable: None,
                        size: None,
                    },
                    LayoutNode::Leaf {
                        name: name_b,
                        renderable: None,
                        size: None,
                    },
                ],
            },
        );

        // Put the old root back as the first child
        if let LayoutNode::Split {
            ref mut children, ..
        } = self.root
        {
            children[0] = old_root;
        }

        &mut self.root
    }

    /// Remove the split at the root.  If the root is a split, it is replaced
    /// with its first child.  If the root is already a leaf, this is a no-op.
    pub fn unsplit(&mut self) {
        let replacement = std::mem::replace(
            &mut self.root,
            LayoutNode::Leaf {
                name: String::new(),
                renderable: None,
                size: None,
            },
        );
        match replacement {
            LayoutNode::Split { mut children, .. } if !children.is_empty() => {
                self.root = children.remove(0);
            }
            other => {
                self.root = other;
            }
        }
    }

    /// Split the root into the given direction with the provided children.
    ///
    /// Each child can be either a [`LayoutNode`] or a [`Renderable`] (which is
    /// automatically wrapped in a leaf). This matches Python Rich's
    /// `Layout.split(*layouts, splitter=...)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rusty_rich::layout::{Layout, LayoutNode, Direction};
    ///
    /// let mut layout = Layout::new(LayoutNode::Leaf {
    ///     name: "root".into(), renderable: None, size: None,
    /// });
    /// layout.split_with(
    ///     Direction::Horizontal,
    ///     vec![
    ///         LayoutNode::Leaf { name: "left".into(), renderable: None, size: None },
    ///         LayoutNode::Leaf { name: "right".into(), renderable: None, size: None },
    ///     ],
    /// );
    /// ```
    pub fn split_with(&mut self, direction: Direction, children: Vec<LayoutNode>) -> &mut Self {
        let sizes = vec![1; children.len()];
        self.root = LayoutNode::Split {
            direction,
            sizes,
            children,
        };
        self
    }

    /// Split the root into a horizontal row of children (side by side).
    ///
    /// Equivalent to Python Rich's `Layout.split_row(*layouts)`. Each child
    /// can be any renderable — it will be wrapped in a named leaf.
    pub fn split_row_with(
        &mut self,
        children: Vec<(&str, impl Renderable + Send + Sync + 'static)>,
    ) -> &mut Self {
        let nodes: Vec<LayoutNode> = children
            .into_iter()
            .map(|(name, renderable)| {
                let name = name.to_string();
                self.renderables
                    .insert(name.clone(), DynRenderable::new(renderable));
                LayoutNode::Leaf {
                    name,
                    renderable: None,
                    size: None,
                }
            })
            .collect();
        self.split_with(Direction::Vertical, nodes)
    }

    /// Split the root into a vertical column of children (stacked).
    ///
    /// Equivalent to Python Rich's `Layout.split_column(*layouts)`. Each child
    /// can be any renderable — it will be wrapped in a named leaf.
    pub fn split_column_with(
        &mut self,
        children: Vec<(&str, impl Renderable + Send + Sync + 'static)>,
    ) -> &mut Self {
        let nodes: Vec<LayoutNode> = children
            .into_iter()
            .map(|(name, renderable)| {
                let name = name.to_string();
                self.renderables
                    .insert(name.clone(), DynRenderable::new(renderable));
                LayoutNode::Leaf {
                    name,
                    renderable: None,
                    size: None,
                }
            })
            .collect();
        self.split_with(Direction::Horizontal, nodes)
    }

    /// Convenience: split the root into a 2-pane column layout (horizontal split).
    pub fn split_column(&mut self) -> &mut Self {
        self.split(Direction::Horizontal);
        self
    }

    /// Convenience: split the root into a 2-pane row layout (vertical split).
    pub fn split_row(&mut self) -> &mut Self {
        self.split(Direction::Vertical);
        self
    }

    /// Recursively find a named layout node in the tree.
    ///
    /// Equivalent to Python Rich's `Layout.get(name)`. Searches depth-first,
    /// returning the first node whose `name` matches.
    pub fn find_node(&self, name: &str) -> Option<&LayoutNode> {
        find_layout_node(&self.root, name)
    }

    /// Add child nodes to an existing split (converting leaf to split if needed).
    ///
    /// Like [`add_split`](Self::add_split) but accepts multiple children at
    /// once. Each child gets ratio 1.
    pub fn add_splits(
        &mut self,
        children: Vec<(&str, impl Renderable + Send + Sync + 'static)>,
    ) -> usize {
        let count = children.len();
        for (name, renderable) in children {
            self.add_split(renderable, 1);
            // The last-added pane gets auto-named; we can't rename it here
            // but the renderable is registered
        }
        count
    }

    /// Add a child pane with a renderable and a ratio weight.
    ///
    /// If the root is already a `Split` node, the new child is appended.
    /// If the root is a `Leaf`, it is first converted to a `Split` containing
    /// the old leaf and the new child.
    ///
    /// Returns the pane ID (index of the new child in the children list).
    pub fn add_split(
        &mut self,
        renderable: impl Renderable + Send + Sync + 'static,
        ratio: usize,
    ) -> usize {
        let id = self.next_pane_id;
        self.next_pane_id += 1;
        let name = format!("_pane_{}", id);
        self.renderables
            .insert(name.clone(), DynRenderable::new(renderable));

        match &mut self.root {
            LayoutNode::Split {
                children, sizes, ..
            } => {
                children.push(LayoutNode::Leaf {
                    name: name.clone(),
                    renderable: None,
                    size: None,
                });
                sizes.push(ratio);
                children.len() - 1
            }
            LayoutNode::Leaf { .. } => {
                // Convert leaf root to a Split containing old + new children
                let old_root = std::mem::replace(
                    &mut self.root,
                    LayoutNode::Split {
                        direction: Direction::Vertical,
                        sizes: vec![1, ratio],
                        children: vec![
                            LayoutNode::Leaf {
                                name: String::new(),
                                renderable: None,
                                size: None,
                            },
                            LayoutNode::Leaf {
                                name: name.clone(),
                                renderable: None,
                                size: None,
                            },
                        ],
                    },
                );
                if let LayoutNode::Split {
                    ref mut children, ..
                } = self.root
                {
                    children[0] = old_root;
                }
                1
            }
        }
    }

    /// Get the root renderable (if the root is a leaf and has a renderable).
    pub fn renderable(&self) -> Option<&dyn Renderable> {
        match &self.root {
            LayoutNode::Leaf { name, .. } => {
                self.renderables.get(name).map(|dr| dr as &dyn Renderable)
            }
            _ => None,
        }
    }

    /// Get child layout nodes (if the root is a split).
    pub fn children(&self) -> &[LayoutNode] {
        match &self.root {
            LayoutNode::Split { children, .. } => children,
            _ => &[],
        }
    }

    /// Get the active splitters.
    pub fn splitters(&self) -> Vec<&dyn Splitter> {
        self.splitters.values().map(|s| s.as_ref()).collect()
    }

    /// Register a named splitter that will be used when a child leaf with
    /// the given `name` appears inside a `Split` node during layout
    /// computation.
    pub fn set_splitter(
        &mut self,
        name: impl Into<String>,
        splitter: impl Splitter + 'static,
    ) {
        self.splitters.insert(name.into(), Box::new(splitter));
    }

    /// Remove a named splitter.  Returns `true` if the splitter was
    /// present.
    pub fn remove_splitter(&mut self, name: &str) -> bool {
        self.splitters.remove(name).is_some()
    }

    /// Get the layout tree root.
    pub fn tree(&self) -> &LayoutNode {
        &self.root
    }

    /// Apply a function to all leaf renderables, replacing each with the
    /// result.
    pub fn map(&mut self, f: impl Fn(&dyn Renderable) -> DynRenderable) {
        let mut new_renderables = HashMap::new();
        for (name, dr) in &self.renderables {
            let new_dr = f(dr as &dyn Renderable);
            new_renderables.insert(name.clone(), new_dr);
        }
        self.renderables = new_renderables;
    }

    /// Get a named renderable from the tree (flat lookup in renderables map).
    pub fn get_renderable(&self, name: &str) -> Option<&dyn Renderable> {
        self.renderables.get(name).map(|dr| dr as &dyn Renderable)
    }

    /// Recursively find a named node in the layout tree and return its
    /// renderable (if any).
    ///
    /// Equivalent to Python Rich's `Layout.get(name)` which does a
    /// depth-first search through the entire layout tree.
    pub fn get(&self, name: &str) -> Option<&dyn Renderable> {
        let node = find_layout_node(&self.root, name)?;
        match node {
            LayoutNode::Leaf { name, .. } => {
                self.renderables.get(name).map(|dr| dr as &dyn Renderable)
            }
            _ => None,
        }
    }

    /// Update a named renderable, returning `true` if it existed.
    pub fn update(
        &mut self,
        name: &str,
        renderable: impl Renderable + Send + Sync + 'static,
    ) -> bool {
        if self.renderables.contains_key(name) {
            self.renderables
                .insert(name.to_string(), DynRenderable::new(renderable));
            true
        } else {
            false
        }
    }

    /// Refresh the layout on screen by re-rendering all visible regions.
    ///
    /// Computes the layout for the current terminal size and renders each
    /// leaf's renderable into the console.
    pub fn refresh_screen(&mut self, console: &mut Console) {
        if !self.visible {
            return;
        }
        let dims = crate::console::ConsoleDimensions::detect();
        let regions = self.compute(dims.width, dims.height);
        // Sort regions top-to-bottom so they render in order
        for (name, _region) in &regions {
            if let Some(renderable) = self.renderables.get(name) {
                // Render each pane — in a full implementation we'd clip to
                // the region; here we just print sequentially.
                let rendered = renderable.render(&ConsoleOptions::default());
                let text = rendered.to_ansi();
                if !text.is_empty() {
                    console.print_str(&text);
                }
            }
        }
    }

    /// Compute region assignments by recursively splitting the given area.
    ///
    /// Returns a list of `(name, region)` pairs for each leaf node in the
    /// layout tree.
    pub fn compute(&self, total_width: usize, total_height: usize) -> Vec<(String, Region)> {
        let mut regions = Vec::new();
        let region = Region {
            x: 0,
            y: 0,
            width: total_width,
            height: total_height,
        };
        self.layout_node(&self.root, region, &mut regions);
        regions
    }

    /// Recursively layout a node.  When the node is a `Split` the engine
    /// first checks whether a named [`Splitter`] has been registered for any
    /// of the direct child leaves (via [`set_splitter`](Self::set_splitter)).
    /// If one is found its [`Splitter::split`] method is used; otherwise the
    /// space is divided proportionally according to the `sizes` ratios (the
    /// same behaviour as the built-in `NoSplitter`).
    fn layout_node(
        &self,
        node: &LayoutNode,
        region: Region,
        out: &mut Vec<(String, Region)>,
    ) {
        match node {
            LayoutNode::Leaf { name, size, .. } => {
                let mut r = region;
                if let Some(s) = size {
                    r.width = r.width.min(*s);
                    r.height = r.height.min(*s);
                } else {
                    r.width = r.width.max(2);
                    r.height = r.height.max(1);
                }
                out.push((name.clone(), r));
            }
            LayoutNode::Split {
                direction,
                sizes,
                children,
            } => {
                if children.is_empty() {
                    return;
                }

                // Look for a registered splitter matching any direct
                // child-leaf name.  First match wins.
                let registered_splitter: Option<&Box<dyn Splitter>> =
                    children.iter().find_map(|child| match child {
                        LayoutNode::Leaf { name, .. } => self.splitters.get(name),
                        _ => None,
                    });

                if let Some(splitter) = registered_splitter {
                    let child_regions =
                        splitter.split(&region, children, direction);
                    for (child, child_region) in
                        children.iter().zip(child_regions.iter())
                    {
                        self.layout_node(child, *child_region, out);
                    }
                } else {
                    let total_size: usize = sizes.iter().sum();
                    if total_size == 0 {
                        return;
                    }
                    let count = children.len();

                    match direction {
                        Direction::Horizontal => {
                            let mut x = region.x;
                            let total_spacing = count.saturating_sub(1);
                            let avail =
                                region.width.saturating_sub(total_spacing);
                            for (i, child) in children.iter().enumerate() {
                                let ratio =
                                    sizes.get(i).copied().unwrap_or(1);
                                let child_w =
                                    (avail * ratio) / total_size;
                                let child_r = Region {
                                    x,
                                    y: region.y,
                                    width: child_w.max(1),
                                    height: region.height,
                                };
                                self.layout_node(child, child_r, out);
                                x += child_w + 1; // 1 char gutter
                            }
                        }
                        Direction::Vertical => {
                            let mut y = region.y;
                            for (i, child) in children.iter().enumerate() {
                                let ratio =
                                    sizes.get(i).copied().unwrap_or(1);
                                let child_h =
                                    (region.height * ratio) / total_size;
                                let child_r = Region {
                                    x: region.x,
                                    y,
                                    width: region.width,
                                    height: child_h.max(1),
                                };
                                self.layout_node(child, child_r, out);
                                y += child_h;
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Splitter trait and implementations
// ---------------------------------------------------------------------------

/// Trait for layout splitters (interface).
///
/// A `Splitter` defines how to divide a [`Region`] among a list of child
/// [`LayoutNode`]s given a [`Direction`].
pub trait Splitter: std::fmt::Debug {
    /// Split `region` among `children` according to `direction`.
    ///
    /// Returns one [`Region`] per child.
    fn split(&self, region: &Region, children: &[LayoutNode], direction: &Direction)
        -> Vec<Region>;
}

/// Default splitter that divides space equally among all children.
#[derive(Debug)]
pub struct NoSplitter;

impl Splitter for NoSplitter {
    fn split(
        &self,
        region: &Region,
        children: &[LayoutNode],
        direction: &Direction,
    ) -> Vec<Region> {
        let count = children.len().max(1);
        match direction {
            Direction::Horizontal => {
                let col_width = region.width / count;
                children
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Region {
                        x: region.x + i * col_width,
                        y: region.y,
                        width: col_width,
                        height: region.height,
                    })
                    .collect()
            }
            Direction::Vertical => {
                let row_height = region.height / count;
                children
                    .iter()
                    .enumerate()
                    .map(|(i, _)| Region {
                        x: region.x,
                        y: region.y + i * row_height,
                        width: region.width,
                        height: row_height,
                    })
                    .collect()
            }
        }
    }
}

/// Splits a region into equal-width columns (ignores the direction).
#[derive(Debug)]
pub struct ColumnSplitter;

impl Splitter for ColumnSplitter {
    fn split(
        &self,
        region: &Region,
        children: &[LayoutNode],
        _direction: &Direction,
    ) -> Vec<Region> {
        let count = children.len().max(1);
        let col_width = region.width / count;
        children
            .iter()
            .enumerate()
            .map(|(i, _)| Region {
                x: region.x + i * col_width,
                y: region.y,
                width: col_width,
                height: region.height,
            })
            .collect()
    }
}

/// Splits a region into equal-height rows (ignores the direction).
#[derive(Debug)]
pub struct RowSplitter;

impl Splitter for RowSplitter {
    fn split(
        &self,
        region: &Region,
        children: &[LayoutNode],
        _direction: &Direction,
    ) -> Vec<Region> {
        let count = children.len().max(1);
        let row_height = region.height / count;
        children
            .iter()
            .enumerate()
            .map(|(i, _)| Region {
                x: region.x,
                y: region.y + i * row_height,
                width: region.width,
                height: row_height,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// LayoutRender
// ---------------------------------------------------------------------------

/// A renderable that visualises a [`Layout`] as a tree diagram.
///
/// Useful for debugging layout configurations — shows the split hierarchy
/// with pane names, directions, and size ratios.
///
/// # Example
///
/// ```rust
/// use rusty_rich::layout::{Layout, LayoutNode, Direction, LayoutRender};
///
/// let node = LayoutNode::split(
///     Direction::Horizontal,
///     vec![
///         LayoutNode::Leaf { name: "left".into(), renderable: None, size: None },
///         LayoutNode::Leaf { name: "right".into(), renderable: None, size: None },
///     ],
/// );
/// let layout = Layout::new(node);
/// let renderable = LayoutRender::new(&layout);
/// ```
#[derive(Debug, Clone)]
pub struct LayoutRender {
    /// Reference to the layout being visualised.
    pub layout: LayoutSnapshot,
}

/// A snapshot of a layout (avoids lifetime issues with borrowing).
#[derive(Debug, Clone)]
pub struct LayoutSnapshot {
    pub root: LayoutNode,
    pub visible: bool,
    pub minimum_size: usize,
}

impl LayoutSnapshot {
    /// Create a snapshot from a [`Layout`].
    pub fn from_layout(layout: &Layout) -> Self {
        Self {
            root: layout.root.clone(),
            visible: layout.visible,
            minimum_size: layout.minimum_size,
        }
    }
}

impl LayoutRender {
    /// Create a new `LayoutRender` from a [`Layout`] reference.
    pub fn new(layout: &Layout) -> Self {
        Self {
            layout: LayoutSnapshot::from_layout(layout),
        }
    }

    /// Create a new `LayoutRender` from a snapshot.
    pub fn from_snapshot(snapshot: LayoutSnapshot) -> Self {
        Self {
            layout: snapshot,
        }
    }
}

impl Renderable for LayoutRender {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let mut lines: Vec<Vec<Segment>> = Vec::new();
        render_layout_node(&self.layout.root, &mut lines, "", options);
        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

/// Recursively render a layout node as a tree.
///
/// Each node's children are rendered with the appropriate tree-drawing
/// characters: `├── ` for non-last children and `└── ` for the last child.
/// Continuation guides (`│   `) connect ancestor levels.
fn render_layout_node(
    node: &LayoutNode,
    lines: &mut Vec<Vec<Segment>>,
    prefix: &str,
    _options: &ConsoleOptions,
) {
    // Build the label for this node
    let label = match node {
        LayoutNode::Leaf { name, size, .. } => {
            let size_str = size.map(|s| format!(" [size={s}]")).unwrap_or_default();
            format!("📄 {name}{size_str}")
        }
        LayoutNode::Split {
            direction,
            sizes,
            ..
        } => {
            let dir_str = match direction {
                Direction::Horizontal => "Horizontal",
                Direction::Vertical => "Vertical",
            };
            let ratio_str = if sizes.iter().all(|&s| s == 1) {
                String::new()
            } else {
                format!(
                    " [{}]",
                    sizes
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!("🔀 Split({dir_str}){ratio_str}")
        }
    };

    lines.push(vec![Segment::new(format!("{prefix}{label}")), Segment::line()]);

    // Render children (only Split nodes have children)
    if let LayoutNode::Split { children, .. } = node {
        let last_idx = children.len().saturating_sub(1);
        for (i, child) in children.iter().enumerate() {
            let is_last = i == last_idx;
            let connector = if is_last { "└── " } else { "├── " };
            let continuation = if is_last { "    " } else { "│   " };
            let child_prefix = format!("{prefix}{connector}");
            let deeper_prefix = format!("{prefix}{continuation}");
            // Render child line
            lines.push(vec![
                Segment::new(format!("{child_prefix}{}", node_label(child))),
                Segment::line(),
            ]);
            // Recurse for nested splits
            render_layout_node(child, lines, &deeper_prefix, _options);
        }
    }
}

/// Get the display label for a layout node (without tree prefix).
fn node_label(node: &LayoutNode) -> String {
    match node {
        LayoutNode::Leaf { name, size, .. } => {
            let size_str = size.map(|s| format!(" [size={s}]")).unwrap_or_default();
            format!("📄 {name}{size_str}")
        }
        LayoutNode::Split {
            direction,
            sizes,
            ..
        } => {
            let dir_str = match direction {
                Direction::Horizontal => "Horizontal",
                Direction::Vertical => "Vertical",
            };
            let ratio_str = if sizes.iter().all(|&s| s == 1) {
                String::new()
            } else {
                format!(
                    " [{}]",
                    sizes
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!("🔀 Split({dir_str}){ratio_str}")
        }
    }
}

/// Convenience function: create a layout and return a [`LayoutRender`] for it.
///
/// This is the Rust equivalent of Python Rich's `Layout.make_layout()`.
pub fn make_layout(root: LayoutNode) -> LayoutRender {
    let layout = Layout::new(root);
    LayoutRender::new(&layout)
}

/// Recursively search for a layout node by name in the tree.
///
/// Used by [`Layout::find_node`] and [`Layout::get`] for Python Rich parity.
fn find_layout_node<'a>(node: &'a LayoutNode, name: &str) -> Option<&'a LayoutNode> {
    match node {
        LayoutNode::Leaf { name: n, .. } if n == name => Some(node),
        LayoutNode::Split { children, .. } => {
            for child in children {
                if let found @ Some(_) = find_layout_node(child, name) {
                    return found;
                }
            }
            None
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_defaults() {
        let r = Region {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        assert_eq!(r.width, 80);
        assert_eq!(r.height, 24);
    }

    #[test]
    fn test_layout_single_leaf() {
        let node = LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        };
        let layout = Layout::new(node);
        let regions = layout.compute(80, 24);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].0, "root");
    }

    #[test]
    fn test_layout_horizontal_split() {
        let children = vec![
            LayoutNode::Leaf {
                name: "left".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "right".into(),
                renderable: None,
                size: None,
            },
        ];
        let node = LayoutNode::split(Direction::Horizontal, children);
        let layout = Layout::new(node);
        let regions = layout.compute(80, 24);
        assert_eq!(regions.len(), 2);
        assert!(regions[0].1.x < regions[1].1.x);
    }

    #[test]
    fn test_layout_vertical_split() {
        let children = vec![
            LayoutNode::Leaf {
                name: "top".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "bottom".into(),
                renderable: None,
                size: None,
            },
        ];
        let node = LayoutNode::split(Direction::Vertical, children);
        let layout = Layout::new(node);
        let regions = layout.compute(80, 24);
        assert_eq!(regions.len(), 2);
        assert!(regions[0].1.y < regions[1].1.y);
    }

    #[test]
    fn test_split_method() {
        let mut layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        layout.split(Direction::Horizontal);
        // Root should now be a Split
        match &layout.root {
            LayoutNode::Split { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("expected Split after split()"),
        }
    }

    #[test]
    fn test_unsplit_method() {
        let mut layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        layout.split(Direction::Horizontal);
        layout.unsplit();
        // Root should be back to a Leaf (the original)
        match &layout.root {
            LayoutNode::Leaf { .. } => {} // ok
            _ => panic!("expected Leaf after unsplit()"),
        }
    }

    #[test]
    fn test_split_column() {
        let mut layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        layout.split_column();
        match &layout.root {
            LayoutNode::Split { direction, .. } => {
                assert_eq!(*direction, Direction::Horizontal);
            }
            _ => panic!("expected Horizontal split"),
        }
    }

    #[test]
    fn test_split_row() {
        let mut layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        layout.split_row();
        match &layout.root {
            LayoutNode::Split { direction, .. } => {
                assert_eq!(*direction, Direction::Vertical);
            }
            _ => panic!("expected Vertical split"),
        }
    }

    #[test]
    fn test_children_method() {
        let mut layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        // Before split, children is empty
        assert!(layout.children().is_empty());
        layout.split(Direction::Horizontal);
        assert_eq!(layout.children().len(), 2);
    }

    #[test]
    fn test_tree_method() {
        let layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        match layout.tree() {
            LayoutNode::Leaf { name, .. } => assert_eq!(name, "root"),
            _ => panic!("expected Leaf"),
        }
    }

    #[test]
    fn test_renderable_none_for_empty_layout() {
        let layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        // No renderable registered
        assert!(layout.renderable().is_none());
    }

    #[test]
    fn test_from_renderable() {
        let layout = Layout::from_renderable("main", "hello world");
        assert!(layout.get("main").is_some());
    }

    #[test]
    fn test_get_and_update() {
        let mut layout = Layout::from_renderable("main", "initial");
        assert!(layout.get("main").is_some());

        let updated = layout.update("main", "updated");
        assert!(updated);

        // Non-existent key
        let not_found = layout.update("nonexistent", "nope");
        assert!(!not_found);
    }

    #[test]
    fn test_map() {
        let mut layout = Layout::from_renderable("main", "hello");
        layout.map(|_r| DynRenderable::new("mapped"));
        assert!(layout.get("main").is_some());
    }

    #[test]
    fn test_add_split_to_leaf() {
        let mut layout = Layout::from_renderable("main", "content");
        let id = layout.add_split("second", 2);
        // Root should now be a Split
        match &layout.root {
            LayoutNode::Split {
                children, sizes, ..
            } => {
                assert_eq!(children.len(), 2);
                assert_eq!(*sizes, vec![1, 2]);
                assert_eq!(id, 1);
            }
            _ => panic!("expected Split after add_split"),
        }
    }

    #[test]
    fn test_no_splitter() {
        let splitter = NoSplitter;
        let children = vec![
            LayoutNode::Leaf {
                name: "a".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "b".into(),
                renderable: None,
                size: None,
            },
        ];
        let region = Region {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        let regions = splitter.split(&region, &children, &Direction::Horizontal);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].width, 40);
        assert_eq!(regions[1].width, 40);
    }

    #[test]
    fn test_column_splitter() {
        let splitter = ColumnSplitter;
        let children = vec![
            LayoutNode::Leaf {
                name: "a".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "b".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "c".into(),
                renderable: None,
                size: None,
            },
        ];
        let region = Region {
            x: 0,
            y: 0,
            width: 90,
            height: 24,
        };
        let regions = splitter.split(&region, &children, &Direction::Vertical);
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].width, 30);
        assert_eq!(regions[1].x, 30);
        assert_eq!(regions[2].x, 60);
    }

    #[test]
    fn test_row_splitter() {
        let splitter = RowSplitter;
        let children = vec![
            LayoutNode::Leaf {
                name: "a".into(),
                renderable: None,
                size: None,
            },
            LayoutNode::Leaf {
                name: "b".into(),
                renderable: None,
                size: None,
            },
        ];
        let region = Region {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
        };
        let regions = splitter.split(&region, &children, &Direction::Horizontal);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].height, 12);
        assert_eq!(regions[1].y, 12);
    }

    #[test]
    fn test_splitters_method() {
        let layout = Layout::new(LayoutNode::Leaf {
            name: "root".into(),
            renderable: None,
            size: None,
        });
        assert!(layout.splitters().is_empty());
    }

    #[test]
    fn test_compute_with_fixed_size() {
        let node = LayoutNode::Leaf {
            name: "fixed".into(),
            renderable: None,
            size: Some(10),
        };
        let layout = Layout::new(node);
        let regions = layout.compute(80, 24);
        assert_eq!(regions[0].1.width, 10);
        assert_eq!(regions[0].1.height, 10);
    }
}
