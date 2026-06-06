//! Screen region — equivalent to Rich's `region.py`.
//!
//! A [`Region`] defines a rectangular area of the terminal: an x/y offset and
//! width/height in character cells. Used by the layout engine to partition
//! screen space among renderables.

/// A rectangular region of the terminal screen.
///
/// Equivalent to Python Rich's `Region(NamedTuple)` with `x`, `y`, `width`,
/// `height` fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    /// Horizontal offset from the left edge of the terminal (0-based).
    pub x: usize,
    /// Vertical offset from the top of the terminal (0-based).
    pub y: usize,
    /// Width of the region in character cells.
    pub width: usize,
    /// Height of the region in character cells.
    pub height: usize,
}

impl Region {
    /// Create a new [`Region`] at the given position with the given size.
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns `true` if the region has zero area.
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// The right edge (exclusive) of the region.
    pub fn right(&self) -> usize {
        self.x + self.width
    }

    /// The bottom edge (exclusive) of the region.
    pub fn bottom(&self) -> usize {
        self.y + self.height
    }

    /// Returns `true` if the given point falls within this region.
    pub fn contains(&self, px: usize, py: usize) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    /// Split this region horizontally at the given column offset.
    ///
    /// Returns two regions: the left portion and the right portion.
    pub fn split_horizontal(&self, at: usize) -> (Region, Region) {
        let left_width = at.min(self.width);
        (
            Region {
                x: self.x,
                y: self.y,
                width: left_width,
                height: self.height,
            },
            Region {
                x: self.x + left_width,
                y: self.y,
                width: self.width.saturating_sub(left_width),
                height: self.height,
            },
        )
    }

    /// Split this region vertically at the given row offset.
    ///
    /// Returns two regions: the top portion and the bottom portion.
    pub fn split_vertical(&self, at: usize) -> (Region, Region) {
        let top_height = at.min(self.height);
        (
            Region {
                x: self.x,
                y: self.y,
                width: self.width,
                height: top_height,
            },
            Region {
                x: self.x,
                y: self.y + top_height,
                width: self.width,
                height: self.height.saturating_sub(top_height),
            },
        )
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Region(x={}, y={}, width={}, height={})",
            self.x, self.y, self.width, self.height
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_new() {
        let r = Region::new(0, 0, 80, 24);
        assert_eq!(r.width, 80);
        assert_eq!(r.height, 24);
    }

    #[test]
    fn test_region_edges() {
        let r = Region::new(5, 10, 20, 30);
        assert_eq!(r.right(), 25);
        assert_eq!(r.bottom(), 40);
    }

    #[test]
    fn test_region_contains() {
        let r = Region::new(0, 0, 10, 10);
        assert!(r.contains(0, 0));
        assert!(r.contains(9, 9));
        assert!(!r.contains(10, 10));
        assert!(!r.contains(0, 10));
    }

    #[test]
    fn test_region_is_empty() {
        assert!(Region::new(0, 0, 0, 10).is_empty());
        assert!(Region::new(0, 0, 10, 0).is_empty());
        assert!(!Region::new(0, 0, 10, 10).is_empty());
    }

    #[test]
    fn test_split_horizontal() {
        let r = Region::new(0, 0, 100, 24);
        let (left, right) = r.split_horizontal(40);
        assert_eq!(left.width, 40);
        assert_eq!(left.x, 0);
        assert_eq!(right.width, 60);
        assert_eq!(right.x, 40);
    }

    #[test]
    fn test_split_vertical() {
        let r = Region::new(0, 0, 80, 24);
        let (top, bottom) = r.split_vertical(10);
        assert_eq!(top.height, 10);
        assert_eq!(top.y, 0);
        assert_eq!(bottom.height, 14);
        assert_eq!(bottom.y, 10);
    }

    #[test]
    fn test_display() {
        let r = Region::new(1, 2, 3, 4);
        assert!(r.to_string().contains("x=1"));
        assert!(r.to_string().contains("y=2"));
    }
}
