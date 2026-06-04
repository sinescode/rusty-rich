//! Layout — split-pane layout system. Equivalent to Rich's `layout.py`.

use std::collections::HashMap;

use crate::console::{Console, ConsoleOptions, DynRenderable, Renderable};

// ---------------------------------------------------------------------------
// Region
// ---------------------------------------------------------------------------

/// A region on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

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
    /// Active splitters for dividing regions among children.
    pub splitters: Vec<Box<dyn Splitter>>,
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
            splitters: Vec::new(),
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
            splitters: Vec::new(),
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

    /// Convenience: split the root into a column layout (horizontal split).
    pub fn split_column(&mut self) -> &mut Self {
        self.split(Direction::Horizontal);
        self
    }

    /// Convenience: split the root into a row layout (vertical split).
    pub fn split_row(&mut self) -> &mut Self {
        self.split(Direction::Vertical);
        self
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
        self.splitters.iter().map(|s| s.as_ref()).collect()
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

    /// Get a named renderable from the tree.
    pub fn get(&self, name: &str) -> Option<&dyn Renderable> {
        self.renderables.get(name).map(|dr| dr as &dyn Renderable)
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
                    let _ = console.print_str(&text);
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
            LayoutNode::Split {
                direction,
                sizes,
                children,
            } => {
                let total_size: usize = sizes.iter().sum();
                if total_size == 0 || children.is_empty() {
                    return;
                }
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
