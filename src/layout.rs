//! Layout — split-pane layout system. Equivalent to Rich's `layout.py`.


/// A region on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Direction of a split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Split content side by side (left to right).
    Horizontal,
    /// Split content stacked (top to bottom).
    Vertical,
}

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
        Self::Split { direction, sizes, children }
    }

    /// Builder: set the size ratios for the children of this split node.
    pub fn sizes(mut self, sizes: Vec<usize>) -> Self {
        if let Self::Split { sizes: ref mut s, .. } = self {
            *s = sizes;
        }
        self
    }
}

/// The Layout compute engine. Assigns screen regions to a tree of layout
/// nodes by recursively splitting available space.
#[derive(Debug, Clone)]
pub struct Layout {
    /// The root [`LayoutNode`] defining the split hierarchy.
    pub root: LayoutNode,
    /// Whether the layout is visible.
    pub visible: bool,
    /// Minimum size for any region.
    pub minimum_size: usize,
}

impl Layout {
    /// Create a new layout with the given root node.
    pub fn new(root: LayoutNode) -> Self {
        Self {
            root,
            visible: true,
            minimum_size: 1,
        }
    }

    /// Compute region assignments by recursively splitting the given area.
    ///
    /// Returns a list of `(name, region)` pairs for each leaf node in the
    /// layout tree.
    pub fn compute(&self, total_width: usize, total_height: usize) -> Vec<(String, Region)> {
        let mut regions = Vec::new();
        let region = Region { x: 0, y: 0, width: total_width, height: total_height };
        Self::layout_node(&self.root, region, &mut regions);
        regions
    }

    fn layout_node(node: &LayoutNode, region: Region, out: &mut Vec<(String, Region)>) {
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
            LayoutNode::Split { direction, sizes, children } => {
                let total_size: usize = sizes.iter().sum();
                let count = children.len();

                match direction {
                    Direction::Horizontal => {
                        let mut x = region.x;
                        let total_spacing = count.saturating_sub(1);
                        let avail = region.width.saturating_sub(total_spacing);
                        for (i, child) in children.iter().enumerate() {
                            let ratio = sizes.get(i).copied().unwrap_or(1);
                            let child_w = (avail * ratio) / total_size;
                            let child_r = Region {
                                x,
                                y: region.y,
                                width: child_w.max(1),
                                height: region.height,
                            };
                            Self::layout_node(child, child_r, out);
                            x += child_w + 1; // 1 char gutter
                        }
                    }
                    Direction::Vertical => {
                        let mut y = region.y;
                        for (i, child) in children.iter().enumerate() {
                            let ratio = sizes.get(i).copied().unwrap_or(1);
                            let child_h = (region.height * ratio) / total_size;
                            let child_r = Region {
                                x: region.x,
                                y,
                                width: region.width,
                                height: child_h.max(1),
                            };
                            Self::layout_node(child, child_r, out);
                            y += child_h;
                        }
                    }
                }
            }
        }
    }
}
