//! Spinner — animated spinner. Equivalent to Rich's `spinner.py`.

use std::time::Duration;

// ---------------------------------------------------------------------------
// Spinner frames
// ---------------------------------------------------------------------------

/// Predefined spinner animations (matching Rich's spinner set).
#[derive(Debug, Clone)]
pub struct SpinnerFrames {
    pub frames: &'static [&'static str],
    pub interval: f64, // seconds per frame
}

// ===========================================================================
// Classic / dots spinners
// ===========================================================================

pub const SPINNER_DOTS: SpinnerFrames = SpinnerFrames {
    frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    interval: 0.08,
};

pub const SPINNER_LINE: SpinnerFrames = SpinnerFrames {
    frames: &["-", "\\", "|", "/"],
    interval: 0.1,
};

pub const SPINNER_DOTS2: SpinnerFrames = SpinnerFrames {
    frames: &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
    interval: 0.08,
};

pub const SPINNER_DOTS3: SpinnerFrames = SpinnerFrames {
    frames: &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"],
    interval: 0.08,
};

pub const SPINNER_DOTS4: SpinnerFrames = SpinnerFrames {
    frames: &["⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠰", "⠸", "⠙", "⠋", "⠇", "⠆"],
    interval: 0.08,
};

pub const SPINNER_DOTS5: SpinnerFrames = SpinnerFrames {
    frames: &["⠋", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋"],
    interval: 0.08,
};

pub const SPINNER_DOTS6: SpinnerFrames = SpinnerFrames {
    frames: &[
        "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠴", "⠲", "⠒",
        "⠂", "⠂", "⠒", "⠚", "⠙", "⠉", "⠁",
    ],
    interval: 0.08,
};

pub const SPINNER_DOTS7: SpinnerFrames = SpinnerFrames {
    frames: &[
        "⠈", "⠉", "⠋", "⠓", "⠒", "⠐", "⠐", "⠒", "⠖", "⠦", "⠤", "⠠", "⠠", "⠤", "⠦", "⠖", "⠒",
        "⠐", "⠐", "⠒", "⠓", "⠋", "⠉", "⠈",
    ],
    interval: 0.08,
};

pub const SPINNER_DOTS8: SpinnerFrames = SpinnerFrames {
    frames: &[
        "⠁", "⠁", "⠉", "⠙", "⠚", "⠒", "⠂", "⠂", "⠒", "⠲", "⠴", "⠤", "⠄", "⠄", "⠤", "⠠", "⠠",
        "⠤", "⠦", "⠖", "⠒", "⠐", "⠐", "⠒", "⠓", "⠋", "⠉", "⠈", "⠈",
    ],
    interval: 0.08,
};

pub const SPINNER_DOTS9: SpinnerFrames = SpinnerFrames {
    frames: &["⢹", "⢺", "⢼", "⣸", "⣇", "⡧", "⡗", "⡏"],
    interval: 0.08,
};

pub const SPINNER_DOTS10: SpinnerFrames = SpinnerFrames {
    frames: &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"],
    interval: 0.08,
};

pub const SPINNER_DOTS11: SpinnerFrames = SpinnerFrames {
    frames: &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
    interval: 0.1,
};

pub const SPINNER_SIMPLE_DOTS: SpinnerFrames = SpinnerFrames {
    frames: &[".  ", ".. ", "...", " ..", "  .", "   "],
    interval: 0.2,
};

// ===========================================================================
// Icon / theme spinners
// ===========================================================================

pub const SPINNER_MOON: SpinnerFrames = SpinnerFrames {
    frames: &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
    interval: 0.08,
};

pub const SPINNER_SMILEY: SpinnerFrames = SpinnerFrames {
    frames: &["😄", "😝"],
    interval: 0.2,
};

// ===========================================================================
// New spinner styles (20+ from Python Rich)
// ===========================================================================

pub const SPINNER_ARC: SpinnerFrames = SpinnerFrames {
    frames: &["◜", "◠", "◝", "◞", "◡", "◟"],
    interval: 0.1,
};

pub const SPINNER_ARROW: SpinnerFrames = SpinnerFrames {
    frames: &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
    interval: 0.1,
};

pub const SPINNER_ARROW2: SpinnerFrames = SpinnerFrames {
    frames: &["⬆️", "↗️", "➡️", "↘️", "⬇️", "↙️", "⬅️", "↖️"],
    interval: 0.1,
};

pub const SPINNER_ARROW3: SpinnerFrames = SpinnerFrames {
    frames: &["▹", "▸", "▹", "▸", "▹", "▸"],
    interval: 0.1,
};

pub const SPINNER_BOUNCING_BAR: SpinnerFrames = SpinnerFrames {
    frames: &["[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]"],
    interval: 0.15,
};

pub const SPINNER_BOUNCING_BALL: SpinnerFrames = SpinnerFrames {
    frames: &[
        "( ●    )", "(  ●   )", "(   ●  )", "(    ● )", "(     ●)", "(    ● )", "(   ●  )",
        "(  ●   )",
    ],
    interval: 0.15,
};

pub const SPINNER_CHRISTMAS: SpinnerFrames = SpinnerFrames {
    frames: &["🌲", "🎄"],
    interval: 0.4,
};

pub const SPINNER_CIRCLE: SpinnerFrames = SpinnerFrames {
    frames: &["◐", "◓", "◑", "◒"],
    interval: 0.1,
};

pub const SPINNER_CLOCK: SpinnerFrames = SpinnerFrames {
    frames: &[
        "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
    ],
    interval: 0.1,
};

pub const SPINNER_EARTH: SpinnerFrames = SpinnerFrames {
    frames: &["🌍", "🌎", "🌏"],
    interval: 0.2,
};

pub const SPINNER_GRENADE: SpinnerFrames = SpinnerFrames {
    frames: &["،  💣  ", "۔  💣  ", " ﹒ 💣  "],
    interval: 0.1,
};

pub const SPINNER_GROW_HORIZONTAL: SpinnerFrames = SpinnerFrames {
    frames: &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "▉", "▊", "▋", "▌", "▍", "▎", "▏"],
    interval: 0.08,
};

pub const SPINNER_GROW_VERTICAL: SpinnerFrames = SpinnerFrames {
    frames: &["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▁"],
    interval: 0.08,
};

pub const SPINNER_HAMBURGER: SpinnerFrames = SpinnerFrames {
    frames: &["☱", "☲", "☴"],
    interval: 0.12,
};

pub const SPINNER_HEARTS: SpinnerFrames = SpinnerFrames {
    frames: &["🩷", "❤️", "🧡", "💛", "💚", "💙", "🩵", "💜", "🤎", "🖤", "🩶", "🤍"],
    interval: 0.12,
};

pub const SPINNER_MONKEY: SpinnerFrames = SpinnerFrames {
    frames: &["🐒", "🐒", "🐒", "🐒", "🙈", "🙉", "🙊", "🐒", "🐒", "🐒", "🐒"],
    interval: 0.15,
};

pub const SPINNER_NOISE: SpinnerFrames = SpinnerFrames {
    frames: &["▓", "▒", "░", "▓", "▒", "░", "▓", "▒", "░"],
    interval: 0.08,
};

pub const SPINNER_PONG: SpinnerFrames = SpinnerFrames {
    frames: &[
        "▐⠂       ▌", "▐⠈       ▌", "▐ ⠂      ▌", "▐ ⠠      ▌",
        "▐  ⡀     ▌", "▐  ⠠     ▌", "▐   ⠂    ▌", "▐   ⠈    ▌",
        "▐    ⠂   ▌", "▐    ⠠   ▌", "▐     ⡀  ▌", "▐     ⠠  ▌",
        "▐      ⠂ ▌", "▐      ⠈ ▌", "▐       ⠂▌", "▐       ⠠▌",
        "▐       ⡀▌", "▐      ⠠ ▌", "▐      ⠂ ▌", "▐     ⠈  ▌",
        "▐     ⠂  ▌", "▐    ⠠   ▌", "▐    ⡀   ▌", "▐   ⠠    ▌",
        "▐   ⠂    ▌", "▐  ⠈     ▌", "▐  ⠂     ▌", "▐ ⠠      ▌",
        "▐ ⠂      ▌", "▐⠈       ▌",
    ],
    interval: 0.08,
};

pub const SPINNER_RUNNER: SpinnerFrames = SpinnerFrames {
    frames: &["🚶", "🏃", "🏃", "🏃", "🚶", "🚶"],
    interval: 0.15,
};

pub const SPINNER_SHARK: SpinnerFrames = SpinnerFrames {
    frames: &["🦈", "🌀", "🦈", "🌀", "🦈", "🌀"],
    interval: 0.15,
};

pub const SPINNER_TOGGLE: SpinnerFrames = SpinnerFrames {
    frames: &["⊶", "⊷"],
    interval: 0.2,
};

pub const SPINNER_TRIANGLE: SpinnerFrames = SpinnerFrames {
    frames: &["◢", "◣", "◤", "◥"],
    interval: 0.1,
};

pub const SPINNER_VERTICAL_BARS: SpinnerFrames = SpinnerFrames {
    frames: &["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂", "▁"],
    interval: 0.08,
};

/// Default spinner.
pub const DEFAULT_SPINNER: SpinnerFrames = SPINNER_DOTS;

// ===========================================================================
// Name-based lookup
// =========================================================================--

/// All known spinners mapped by name (lowercase) for runtime lookup.
pub const SPINNERS: &[(&str, &SpinnerFrames)] = &[
    ("arc", &SPINNER_ARC),
    ("arrow", &SPINNER_ARROW),
    ("arrow2", &SPINNER_ARROW2),
    ("arrow3", &SPINNER_ARROW3),
    ("bouncingBar", &SPINNER_BOUNCING_BAR),
    ("bouncingBall", &SPINNER_BOUNCING_BALL),
    ("christmas", &SPINNER_CHRISTMAS),
    ("circle", &SPINNER_CIRCLE),
    ("clock", &SPINNER_CLOCK),
    ("dots", &SPINNER_DOTS),
    ("dots2", &SPINNER_DOTS2),
    ("dots3", &SPINNER_DOTS3),
    ("dots4", &SPINNER_DOTS4),
    ("dots5", &SPINNER_DOTS5),
    ("dots6", &SPINNER_DOTS6),
    ("dots7", &SPINNER_DOTS7),
    ("dots8", &SPINNER_DOTS8),
    ("dots9", &SPINNER_DOTS9),
    ("dots10", &SPINNER_DOTS10),
    ("dots11", &SPINNER_DOTS11),
    ("earth", &SPINNER_EARTH),
    ("grenade", &SPINNER_GRENADE),
    ("growHorizontal", &SPINNER_GROW_HORIZONTAL),
    ("growVertical", &SPINNER_GROW_VERTICAL),
    ("hamburger", &SPINNER_HAMBURGER),
    ("hearts", &SPINNER_HEARTS),
    ("line", &SPINNER_LINE),
    ("monkey", &SPINNER_MONKEY),
    ("moon", &SPINNER_MOON),
    ("noise", &SPINNER_NOISE),
    ("pong", &SPINNER_PONG),
    ("runner", &SPINNER_RUNNER),
    ("shark", &SPINNER_SHARK),
    ("simpleDots", &SPINNER_SIMPLE_DOTS),
    ("smiley", &SPINNER_SMILEY),
    ("toggle", &SPINNER_TOGGLE),
    ("triangle", &SPINNER_TRIANGLE),
    ("verticalBars", &SPINNER_VERTICAL_BARS),
];

/// Get a spinner by name (case-insensitive).
///
/// Returns `None` if no spinner with the given name exists.
///
/// # Example
///
/// ```rust
/// use rusty_rich::get_spinner;
///
/// let s = get_spinner("arc").unwrap();
/// assert_eq!(s.frames.len(), 6);
/// ```
pub fn get_spinner(name: &str) -> Option<&'static SpinnerFrames> {
    // First try direct lowercase match
    for (key, spinner) in SPINNERS {
        if key.eq_ignore_ascii_case(name) {
            return Some(spinner);
        }
    }
    // Fallback: try stripping spaces and hyphens, matching lowercase
    let normalized: String = name.chars().filter(|c| !c.is_whitespace()).collect();
    let normalized = normalized.to_lowercase();
    for (key, spinner) in SPINNERS {
        let key_normalized: String = key.chars().filter(|c| !c.is_whitespace()).collect();
        let key_normalized = key_normalized.to_lowercase();
        if key_normalized == normalized {
            return Some(spinner);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

/// An animated spinner renderable.
#[derive(Debug, Clone)]
pub struct Spinner {
    pub frames: &'static [&'static str],
    pub interval: f64,
    /// Text displayed alongside the spinner.
    pub text: String,
    /// Style for the spinner.
    pub style: crate::style::Style,
}

impl Spinner {
    /// Create a new spinner.
    pub fn new(spinner: &'static SpinnerFrames) -> Self {
        Self {
            frames: spinner.frames,
            interval: spinner.interval,
            text: String::new(),
            style: crate::style::Style::new(),
        }
    }

    /// Builder: set the text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Builder: set the style.
    pub fn style(mut self, style: crate::style::Style) -> Self {
        self.style = style;
        self
    }

    /// Get the frame at the given elapsed time.
    pub fn frame_at(&self, elapsed: Duration) -> &'static str {
        let idx = (elapsed.as_secs_f64() / self.interval) as usize % self.frames.len();
        self.frames[idx]
    }

    /// Get the display string for the current time.
    pub fn render(&self, elapsed: Duration) -> String {
        let frame = self.frame_at(elapsed);
        let style_ansi = self.style.to_ansi();
        let reset = if style_ansi.is_empty() { "" } else { "\x1b[0m" };
        if self.text.is_empty() {
            format!("{style_ansi}{frame}{reset}")
        } else {
            format!("{style_ansi}{frame}{reset} {}", self.text)
        }
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new(&DEFAULT_SPINNER)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frame_at() {
        let s = Spinner::new(&SPINNER_LINE);
        let f = s.frame_at(Duration::from_millis(200));
        assert!(f == "-" || f == "\\" || f == "|" || f == "/");
    }

    #[test]
    fn test_get_spinner_found() {
        let s = get_spinner("dots").unwrap();
        assert!(!s.frames.is_empty());

        let s = get_spinner("DOTS").unwrap();
        assert!(!s.frames.is_empty());

        let s = get_spinner("arc").unwrap();
        assert_eq!(s.frames.len(), 6);
    }

    #[test]
    fn test_get_spinner_not_found() {
        assert!(get_spinner("nonexistent").is_none());
    }

    #[test]
    fn test_get_spinner_case_insensitive() {
        let s1 = get_spinner("ARC").unwrap();
        let s2 = get_spinner("arc").unwrap();
        assert_eq!(s1.frames, s2.frames);
    }

    #[test]
    fn test_get_spinner_camel_case() {
        let s = get_spinner("bouncingBar").unwrap();
        assert!(!s.frames.is_empty());

        let s = get_spinner("BOUNCINGBAR").unwrap();
        assert!(!s.frames.is_empty());
    }

    #[test]
    fn test_spinners_list_all_accessible() {
        for (name, frames) in SPINNERS {
            let found = get_spinner(name).unwrap();
            assert!(!frames.frames.is_empty(), "spinner '{}' has no frames", name);
            // Compare frame content rather than raw pointers, since `const`
            // values may be inlined at different addresses by the compiler.
            assert_eq!(
                frames.frames, found.frames,
                "spinner '{}' points to different frames than expected",
                name
            );
            assert!(
                (frames.interval - found.interval).abs() < f64::EPSILON,
                "spinner '{}' interval mismatch",
                name
            );
        }
    }

    #[test]
    fn test_spinner_arc_frames() {
        assert_eq!(SPINNER_ARC.frames.len(), 6);
        assert!(SPINNER_ARC.interval > 0.0);
    }

    #[test]
    fn test_spinner_arrow_frames() {
        assert_eq!(SPINNER_ARROW.frames.len(), 8);
    }

    #[test]
    fn test_spinner_arrow2_frames() {
        assert_eq!(SPINNER_ARROW2.frames.len(), 8);
    }

    #[test]
    fn test_spinner_arrow3_frames() {
        assert_eq!(SPINNER_ARROW3.frames.len(), 6);
    }

    #[test]
    fn test_spinner_bouncing_bar() {
        assert_eq!(SPINNER_BOUNCING_BAR.frames.len(), 8);
    }

    #[test]
    fn test_spinner_bouncing_ball() {
        assert_eq!(SPINNER_BOUNCING_BALL.frames.len(), 8);
    }

    #[test]
    fn test_spinner_christmas() {
        assert_eq!(SPINNER_CHRISTMAS.frames.len(), 2);
    }

    #[test]
    fn test_spinner_circle() {
        assert_eq!(SPINNER_CIRCLE.frames.len(), 4);
    }

    #[test]
    fn test_spinner_clock() {
        assert_eq!(SPINNER_CLOCK.frames.len(), 12);
    }

    #[test]
    fn test_spinner_earth() {
        assert_eq!(SPINNER_EARTH.frames.len(), 3);
    }

    #[test]
    fn test_spinner_grenade() {
        assert_eq!(SPINNER_GRENADE.frames.len(), 3);
    }

    #[test]
    fn test_spinner_grow_horizontal() {
        assert_eq!(SPINNER_GROW_HORIZONTAL.frames.len(), 14);
    }

    #[test]
    fn test_spinner_grow_vertical() {
        assert_eq!(SPINNER_GROW_VERTICAL.frames.len(), 14);
    }

    #[test]
    fn test_spinner_hamburger() {
        assert_eq!(SPINNER_HAMBURGER.frames.len(), 3);
    }

    #[test]
    fn test_spinner_hearts() {
        assert_eq!(SPINNER_HEARTS.frames.len(), 12);
    }

    #[test]
    fn test_spinner_monkey() {
        assert_eq!(SPINNER_MONKEY.frames.len(), 11);
    }

    #[test]
    fn test_spinner_noise() {
        assert_eq!(SPINNER_NOISE.frames.len(), 9);
    }

    #[test]
    fn test_spinner_pong() {
        assert_eq!(SPINNER_PONG.frames.len(), 30);
    }

    #[test]
    fn test_spinner_runner() {
        assert_eq!(SPINNER_RUNNER.frames.len(), 6);
    }

    #[test]
    fn test_spinner_shark() {
        assert_eq!(SPINNER_SHARK.frames.len(), 6);
    }

    #[test]
    fn test_spinner_toggle() {
        assert_eq!(SPINNER_TOGGLE.frames.len(), 2);
    }

    #[test]
    fn test_spinner_triangle() {
        assert_eq!(SPINNER_TRIANGLE.frames.len(), 4);
    }

    #[test]
    fn test_spinner_vertical_bars() {
        assert_eq!(SPINNER_VERTICAL_BARS.frames.len(), 15);
    }

    #[test]
    fn test_spinner_interval_positive() {
        for (name, frames) in SPINNERS {
            assert!(
                frames.interval > 0.0,
                "spinner '{}' has non-positive interval",
                name
            );
        }
    }

    #[test]
    fn test_default_spinner_is_dots() {
        assert_eq!(DEFAULT_SPINNER.frames, SPINNER_DOTS.frames);
    }
}
