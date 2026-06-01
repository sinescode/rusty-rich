//! Measurement system — equivalent to Rich's `measure.py`.
//!
//! Used during layout to determine the minimum and maximum widths of
//! renderable objects.

use crate::console::ConsoleOptions;

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

/// A range of valid widths for a renderable: [minimum, maximum].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measurement {
    pub minimum: usize,
    pub maximum: usize,
}

impl Measurement {
    /// Create a new measurement.
    pub fn new(minimum: usize, maximum: usize) -> Self {
        Self { minimum, maximum }
    }

    /// Clamp the measurement to a given maximum.
    pub fn with_maximum(self, maximum: usize) -> Self {
        Self {
            minimum: self.minimum.min(maximum),
            maximum: self.maximum.min(maximum),
        }
    }

    /// Clamp the measurement to a given minimum.
    pub fn with_minimum(self, minimum: usize) -> Self {
        Self {
            minimum: self.minimum.max(minimum),
            maximum: self.maximum.max(minimum),
        }
    }

    /// Shrink both minimum and maximum by `amount`.
    pub fn shrink(self, amount: usize) -> Self {
        Self {
            minimum: self.minimum.saturating_sub(amount),
            maximum: self.maximum.saturating_sub(amount),
        }
    }

    /// Grow both minimum and maximum by `amount`.
    pub fn grow(self, amount: usize) -> Self {
        Self {
            minimum: self.minimum + amount,
            maximum: self.maximum + amount,
        }
    }

    /// Return a fixed-width measurement.
    pub fn fixed(width: usize) -> Self {
        Self {
            minimum: width,
            maximum: width,
        }
    }
}

// ---------------------------------------------------------------------------
// Measurement utilities (equivalent to `measure_renderables`)
// ---------------------------------------------------------------------------

/// Trait for objects that can report their width measurement.
pub trait Measurable {
    fn measure(&self, options: &ConsoleOptions) -> Measurement;
}

impl Measurable for String {
    fn measure(&self, _options: &ConsoleOptions) -> Measurement {
        let w = unicode_width::UnicodeWidthStr::width(self.as_str());
        Measurement::fixed(w)
    }
}

impl Measurable for &str {
    fn measure(&self, _options: &ConsoleOptions) -> Measurement {
        let w = unicode_width::UnicodeWidthStr::width(*self);
        Measurement::fixed(w)
    }
}

/// Measure a collection of renderables and return the aggregate measurement.
///
/// The minimum is the max of all minimums; the maximum is the max of all
/// maximums (assuming items stack vertically, not horizontally).
pub fn measure_renderables<'a>(
    items: impl IntoIterator<Item = &'a dyn Measurable>,
    options: &ConsoleOptions,
) -> Measurement {
    let mut min = 0usize;
    let mut max = 0usize;
    for item in items {
        let m = item.measure(options);
        min = min.max(m.minimum);
        max = max.max(m.maximum);
    }
    Measurement::new(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_clamp() {
        let m = Measurement::new(10, 100).with_maximum(50);
        assert_eq!(m.minimum, 10);
        assert_eq!(m.maximum, 50);
    }
}
