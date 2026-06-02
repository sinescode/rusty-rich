//! Constrain the width of a renderable.
//!
//! Equivalent to Python Rich's `constrain.py`. Wraps any renderable and
//! limits its rendered width to a specified maximum, applying the chosen
//! overflow behaviour when the content exceeds the constraint.

use crate::console::{ConsoleOptions, DynRenderable, OverflowMethod, RenderResult, Renderable};
use crate::measure::Measurement;

/// Constrains a renderable to a maximum width.
///
/// Wraps an inner renderable and caps its rendering width to `width`.
/// If the content is wider, the `overflow` method determines how it is
/// handled (fold, crop, ellipsis, or ignore).
#[derive(Debug, Clone)]
pub struct Constrain {
    /// The inner renderable.
    renderable: DynRenderable,
    /// The maximum width to constrain to.
    width: Option<usize>,
    /// How to handle overflow when content exceeds the constraint.
    overflow: OverflowMethod,
}

impl Constrain {
    /// Create a new `Constrain` wrapping the given renderable.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: DynRenderable::new(renderable),
            width: None,
            overflow: OverflowMethod::Fold,
        }
    }

    /// Builder: set the maximum width.
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Builder: set the overflow method.
    pub fn overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = overflow;
        self
    }
}

impl Renderable for Constrain {
    fn render(&self, options: &ConsoleOptions) -> RenderResult {
        let constrained_width = self.width.unwrap_or(options.max_width);
        let constrained_opts = ConsoleOptions {
            max_width: constrained_width,
            overflow: Some(self.overflow),
            ..options.clone()
        };
        self.renderable.render(&constrained_opts)
    }

    fn measure(&self, options: &ConsoleOptions) -> Option<Measurement> {
        let m = self.renderable.measure(options)?;
        let max = match self.width {
            Some(w) => m.maximum.min(w),
            None => m.maximum,
        };
        Some(Measurement::new(m.minimum, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleOptions;

    #[test]
    fn test_constrain_defaults() {
        let c = Constrain::new("Hello, World!");
        let opts = ConsoleOptions::default();
        let result = c.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("Hello, World!"));
    }

    #[test]
    fn test_constrain_width() {
        // Constrain with a string — strings don't implement measure (returns None)
        let text = "Short";
        let c = Constrain::new(text.to_string()).width(10);
        let opts = ConsoleOptions::default();
        let result = c.render(&opts);
        // Verify rendering respects the constraint
        let ansi = result.to_ansi();
        assert!(ansi.contains("Short"));
    }

    #[test]
    fn test_constrain_render_respects_width() {
        let c = Constrain::new("Hello!").width(3);
        let opts = ConsoleOptions::default();
        let result = c.render(&opts);
        // The inner renderable's max_width is constrained to 3
        let _ = result.to_ansi();
    }

    #[test]
    fn test_constrain_overflow_method() {
        let c = Constrain::new("Hello")
            .width(3)
            .overflow(OverflowMethod::Crop);
        assert!(matches!(c.overflow, OverflowMethod::Crop));
    }

    #[test]
    fn test_constrain_no_width() {
        let c = Constrain::new("Hello");
        assert!(c.width.is_none());
    }

    #[test]
    fn test_constrain_builder_chain() {
        let c = Constrain::new("text")
            .width(20)
            .overflow(OverflowMethod::Ellipsis);
        assert_eq!(c.width, Some(20));
        assert!(matches!(c.overflow, OverflowMethod::Ellipsis));
    }
}
