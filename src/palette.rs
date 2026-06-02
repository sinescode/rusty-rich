//! Color palette utilities — generate color ramps and palettes.
//!
//! Provides [`Palette`] with factory methods for gradients, rainbow
//! palettes, monochromatic schemes, and the standard ANSI palettes.

use crate::color::{blend_rgb, Color, TerminalTheme};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

/// A palette of colors.
///
/// # Example
///
/// ```rust
/// use rusty_rich::{Palette, Color};
///
/// let mut p = Palette::new("my palette");
/// p.add(Color::parse("red").unwrap());
/// p.add(Color::parse("green").unwrap());
/// assert_eq!(p.get(0), &Color::parse("red").unwrap());
/// ```
#[derive(Debug, Clone)]
pub struct Palette {
    /// The colors in this palette.
    pub colors: Vec<Color>,
    /// A human-readable name for this palette.
    pub name: String,
}

impl Palette {
    /// Create a new empty palette with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            colors: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a color to the palette.
    pub fn add(&mut self, color: Color) -> &mut Self {
        self.colors.push(color);
        self
    }

    /// Get a color by index (wraps around using modulo).
    pub fn get(&self, index: usize) -> &Color {
        &self.colors[index % self.colors.len()]
    }

    /// Generate a palette from a gradient between two colors.
    ///
    /// Produces `steps` equally-spaced interpolated colors.
    pub fn from_gradient(start: Color, end: Color, steps: usize) -> Self {
        let theme = TerminalTheme::default();
        let start_rgb = start.get_truecolor(&theme);
        let end_rgb = end.get_truecolor(&theme);
        let mut colors = Vec::with_capacity(steps);

        for i in 0..steps {
            let t = if steps <= 1 {
                0.0
            } else {
                i as f64 / (steps - 1) as f64
            };
            let rgb = blend_rgb(start_rgb, end_rgb, t);
            colors.push(Color::from_rgb(rgb.0, rgb.1, rgb.2));
        }

        Self {
            colors,
            name: format!("gradient({}, {})", start, end),
        }
    }

    /// Generate a rainbow palette.
    ///
    /// Produces `steps` colors cycling through the visible spectrum.
    pub fn rainbow(steps: usize) -> Self {
        // Classic rainbow anchor points (ROYGBIV)
        let anchors: &[(u8, u8, u8)] = &[
            (255, 0, 0),   // red
            (255, 127, 0), // orange
            (255, 255, 0), // yellow
            (0, 255, 0),   // green
            (0, 0, 255),   // blue
            (75, 0, 130),  // indigo
            (148, 0, 211), // violet
        ];

        if steps <= anchors.len() {
            let mut colors: Vec<Color> = anchors
                .iter()
                .take(steps)
                .map(|&(r, g, b)| Color::from_rgb(r, g, b))
                .collect();
            // Pad with repeats if steps < anchors.len()
            while colors.len() < steps {
                colors.push(Color::from_rgb(255, 0, 0));
            }
            return Self {
                colors,
                name: "rainbow".into(),
            };
        }

        let segments = anchors.len() - 1;
        let per_segment = steps / segments;
        let remainder = steps % segments;
        let mut colors = Vec::with_capacity(steps);

        for seg in 0..segments {
            let count = per_segment + if seg < remainder { 1 } else { 0 };
            let start = anchors[seg];
            let end = anchors[seg + 1];
            for i in 0..count {
                let t = if count <= 1 {
                    0.0
                } else {
                    i as f64 / (count - 1) as f64
                };
                let rgb = blend_rgb(start, end, t);
                colors.push(Color::from_rgb(rgb.0, rgb.1, rgb.2));
            }
        }

        Self {
            colors,
            name: "rainbow".into(),
        }
    }

    /// Generate a monochromatic palette from a single color.
    ///
    /// Produces `steps` colors ranging from a lighter tint through the
    /// base color to a darker shade.
    pub fn monochrome(base: Color, steps: usize) -> Self {
        let theme = TerminalTheme::default();
        let base_rgb = base.get_truecolor(&theme);
        let white = (255, 255, 255);
        let black = (0, 0, 0);
        let mut colors = Vec::with_capacity(steps);

        for i in 0..steps {
            let t = if steps <= 1 {
                0.5
            } else {
                i as f64 / (steps - 1) as f64
            };
            // t=0 -> white tint, t=0.5 -> base, t=1 -> black shade
            let rgb = if t < 0.5 {
                blend_rgb(white, base_rgb, t * 2.0)
            } else {
                blend_rgb(base_rgb, black, (t - 0.5) * 2.0)
            };
            colors.push(Color::from_rgb(rgb.0, rgb.1, rgb.2));
        }

        Self {
            colors,
            name: format!("monochrome({})", base),
        }
    }

    /// Get the standard 8-color palette.
    ///
    /// Colors: black, red, green, yellow, blue, magenta, cyan, white.
    pub fn standard() -> Self {
        Self {
            name: "standard".into(),
            colors: vec![
                Color::parse("black").unwrap(),
                Color::parse("red").unwrap(),
                Color::parse("green").unwrap(),
                Color::parse("yellow").unwrap(),
                Color::parse("blue").unwrap(),
                Color::parse("magenta").unwrap(),
                Color::parse("cyan").unwrap(),
                Color::parse("white").unwrap(),
            ],
        }
    }

    /// Get the extended 16-color palette.
    ///
    /// Includes the standard 8 colors plus their bright variants.
    pub fn extended() -> Self {
        Self {
            name: "extended".into(),
            colors: vec![
                Color::parse("black").unwrap(),
                Color::parse("red").unwrap(),
                Color::parse("green").unwrap(),
                Color::parse("yellow").unwrap(),
                Color::parse("blue").unwrap(),
                Color::parse("magenta").unwrap(),
                Color::parse("cyan").unwrap(),
                Color::parse("white").unwrap(),
                Color::parse("bright_black").unwrap(),
                Color::parse("bright_red").unwrap(),
                Color::parse("bright_green").unwrap(),
                Color::parse("bright_yellow").unwrap(),
                Color::parse("bright_blue").unwrap(),
                Color::parse("bright_magenta").unwrap(),
                Color::parse("bright_cyan").unwrap(),
                Color::parse("bright_white").unwrap(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Linearly interpolate between two RGB triplets.
///
/// `t` is clamped to [0.0, 1.0] where 0.0 yields `a` and 1.0 yields `b`.
pub fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t) as u8,
        (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t) as u8,
        (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_new() {
        let p = Palette::new("test");
        assert_eq!(p.name, "test");
        assert!(p.colors.is_empty());
    }

    #[test]
    fn test_palette_add_and_get() {
        let mut p = Palette::new("test");
        p.add(Color::parse("red").unwrap());
        p.add(Color::parse("blue").unwrap());
        assert_eq!(p.get(0), &Color::parse("red").unwrap());
        assert_eq!(p.get(1), &Color::parse("blue").unwrap());
        // Wraps around
        assert_eq!(p.get(2), &Color::parse("red").unwrap());
    }

    #[test]
    fn test_palette_standard() {
        let p = Palette::standard();
        assert_eq!(p.name, "standard");
        assert_eq!(p.colors.len(), 8);
    }

    #[test]
    fn test_palette_extended() {
        let p = Palette::extended();
        assert_eq!(p.name, "extended");
        assert_eq!(p.colors.len(), 16);
    }

    #[test]
    fn test_from_gradient() {
        let p = Palette::from_gradient(
            Color::parse("red").unwrap(),
            Color::parse("blue").unwrap(),
            5,
        );
        assert_eq!(p.colors.len(), 5);
        // First color should be close to red
        let first = p.colors[0];
        let theme = TerminalTheme::default();
        let first_rgb = first.get_truecolor(&theme);
        // Red should be the dominant channel (greater than green and blue)
        assert!(first_rgb.0 >= first_rgb.1);
        assert!(first_rgb.0 >= first_rgb.2);
    }

    #[test]
    fn test_rainbow() {
        let p = Palette::rainbow(7);
        assert_eq!(p.colors.len(), 7);
        assert_eq!(p.name, "rainbow");
    }

    #[test]
    fn test_rainbow_many_steps() {
        let p = Palette::rainbow(100);
        assert_eq!(p.colors.len(), 100);
    }

    #[test]
    fn test_monochrome() {
        let p = Palette::monochrome(Color::parse("blue").unwrap(), 5);
        assert_eq!(p.colors.len(), 5);
    }

    #[test]
    fn test_lerp_rgb() {
        let result = lerp_rgb((0, 0, 0), (255, 255, 255), 0.5);
        assert_eq!(result, (127, 127, 127));

        // t=0 returns first color
        assert_eq!(lerp_rgb((10, 20, 30), (100, 200, 250), 0.0), (10, 20, 30));

        // t=1 returns second color
        assert_eq!(lerp_rgb((10, 20, 30), (100, 200, 250), 1.0), (100, 200, 250));
    }

    #[test]
    fn test_lerp_rgb_clamp() {
        let result = lerp_rgb((0, 0, 0), (100, 100, 100), -0.5);
        assert_eq!(result, (0, 0, 0));

        let result = lerp_rgb((0, 0, 0), (100, 100, 100), 1.5);
        assert_eq!(result, (100, 100, 100));
    }
}
