//! Bar chart renderable — horizontal bars with labels.
//!
//! Equivalent to Rich's `bar.py`. Renders a set of labeled horizontal bars
//! with optional title, value display, and custom bar characters.

use crate::color::Color;
use crate::console::{ConsoleOptions, Renderable, RenderResult};
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Bar
// ---------------------------------------------------------------------------

/// A single bar in a bar chart.
#[derive(Debug, Clone)]
pub struct Bar {
    /// The label displayed to the left of the bar.
    pub label: String,
    /// The numeric value determining bar length.
    pub value: f64,
    /// The color of the bar.
    pub color: Color,
    /// The style applied to the bar text.
    pub style: Style,
}

impl Bar {
    /// Create a new `Bar` with the given label and value.
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: Color::default(),
            style: Style::new(),
        }
    }

    /// Builder: set the bar color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

// ---------------------------------------------------------------------------
// BarChart
// ---------------------------------------------------------------------------

/// A bar chart with multiple bars.
///
/// Displays horizontal bars with left-aligned labels, optionally showing
/// a title, bar values, and a custom bar character.
///
/// # Example
///
/// ```rust
/// use rusty_rich::{BarChart, Bar, Color};
///
/// let chart = BarChart::new()
///     .title("Sales by Quarter")
///     .show_values(true)
///     .bar_char('\u{2588}')
///     .width(60);
/// ```
#[derive(Debug, Clone)]
pub struct BarChart {
    bars: Vec<Bar>,
    width: Option<usize>,
    max_value: Option<f64>,
    title: Option<String>,
    show_values: bool,
    bar_char: char,
    bar_width: usize,
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new()
    }
}

impl BarChart {
    /// Create a new empty `BarChart`.
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            width: None,
            max_value: None,
            title: None,
            show_values: false,
            bar_char: '\u{2588}',
            bar_width: 40,
        }
    }

    /// Add a bar to the chart (mutable, chaining).
    pub fn add(&mut self, bar: Bar) -> &mut Self {
        self.bars.push(bar);
        self
    }

    /// Builder: set the chart width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: set the chart title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Builder: show numeric values after each bar.
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Builder: set the character used to draw bars.
    pub fn bar_char(mut self, ch: char) -> Self {
        self.bar_char = ch;
        self
    }

    /// Builder: set the maximum value for bar scaling.
    pub fn max_value(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Builder: set the width of the bar area in characters.
    pub fn bar_width(mut self, width: usize) -> Self {
        self.bar_width = width;
        self
    }

    /// Auto-compute max value from bars.
    fn compute_max(&self) -> f64 {
        self.max_value.unwrap_or_else(|| {
            self.bars
                .iter()
                .map(|b| b.value)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(1.0)
        })
    }
}

impl Renderable for BarChart {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let max = self.compute_max();
        let available = self.width.unwrap_or(options.max_width).saturating_sub(20);
        let bar_width = self.bar_width.max(available);

        let mut lines = Vec::new();

        // Optional title
        if let Some(ref title) = self.title {
            lines.push(vec![Segment::styled(title, Style::new().bold(true))]);
            lines.push(vec![Segment::line()]);
        }

        for bar in &self.bars {
            let filled = ((bar.value / max) * bar_width as f64) as usize;
            let bar_str: String = self.bar_char.to_string().repeat(filled);
            let label = format!("{:>15} ", bar.label);
            let value_str = if self.show_values {
                format!(" {:.1}", bar.value)
            } else {
                String::new()
            };
            let line_str = format!("{}{}{}", label, bar_str, value_str);

            let mut seg = Segment::new(line_str);
            seg.style = Some(bar.style.clone().color(bar.color));
            lines.push(vec![seg, Segment::line()]);
        }

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_bar_creation() {
        let bar = Bar::new("Test", 42.0).color(Color::parse("red").unwrap());
        assert_eq!(bar.label, "Test");
        assert_eq!(bar.value, 42.0);
    }

    #[test]
    fn test_barchart_creation() {
        let mut chart = BarChart::new().title("Chart").show_values(true);
        chart.add(Bar::new("A", 10.0));
        chart.add(Bar::new("B", 20.0));
        assert_eq!(chart.bars.len(), 2);
    }

    #[test]
    fn test_barchart_render() {
        let mut chart = BarChart::new().width(60);
        chart.add(Bar::new("Foo", 50.0));
        chart.add(Bar::new("Bar", 100.0));
        let opts = ConsoleOptions::default();
        let result = chart.render(&opts);
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_compute_max() {
        let chart = BarChart::new();
        assert_eq!(chart.compute_max(), 1.0);

        let mut chart = BarChart::new();
        chart.add(Bar::new("A", 5.0));
        chart.add(Bar::new("B", 15.0));
        chart.add(Bar::new("C", 10.0));
        assert!((chart.compute_max() - 15.0).abs() < f64::EPSILON);
    }
}
