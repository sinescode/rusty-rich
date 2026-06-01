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
    Horizontal,
    Vertical,
}

/// A layout node — can be a leaf (containing a renderable) or a split.
#[derive(Debug, Clone)]
pub enum LayoutNode {
    /// A split container with children and a direction.
    Split {
        direction: Direction,
        sizes: Vec<usize>, // relative sizes (ratios)
        children: Vec<LayoutNode>,
    },
    /// A leaf with a renderable name (placeholder) and optional fixed size.
    Leaf {
        name: String,
        renderable: Option<String>, // label for now
        size: Option<usize>,
    },
}

impl LayoutNode {
    /// Create a new vertical split.
    pub fn split(direction: Direction, children: Vec<LayoutNode>) -> Self {
        let sizes = vec![1; children.len()];
        Self::Split { direction, sizes, children }
    }

    /// Update the size ratios.
    pub fn sizes(mut self, sizes: Vec<usize>) -> Self {
        if let Self::Split { sizes: ref mut s, .. } = self {
            *s = sizes;
        }
        self
    }
}

/// The Layout compute engine.
#[derive(Debug, Clone)]
pub struct Layout {
    pub root: LayoutNode,
    pub visible: bool,
    pub minimum_size: usize,
}

impl Layout {
    pub fn new(root: LayoutNode) -> Self {
        Self {
            root,
            visible: true,
            minimum_size: 1,
        }
    }

    /// Compute region assignments by recursively splitting the given area.
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
