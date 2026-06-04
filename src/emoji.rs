//! Emoji support — replaces `:shortcode:` with Unicode emoji characters.
//!
//! Equivalent to Python Rich's `emoji.py`. Provides both a default emoji
//! replacer and a no-op variant for environments where emoji are undesirable.

/// Renders emoji shortcodes in text.
///
/// Replaces patterns like `:smile:` with the corresponding Unicode emoji (`😊`).
pub struct Emoji;

impl Emoji {
    /// Replace all emoji shortcodes in `text` with their Unicode equivalents.
    ///
    /// If a shortcode is not recognised, it is left unchanged.
    pub fn replace(text: &str) -> String {
        let map = get_emoji_map();
        let mut result = text.to_string();
        for (code, emoji) in &map {
            let pattern = format!(":{code}:");
            result = result.replace(&pattern, emoji);
        }
        result
    }

    /// Check whether `text` contains any known emoji shortcodes.
    pub fn has_emoji(text: &str) -> bool {
        let map = get_emoji_map();
        for (code, _) in &map {
            let pattern = format!(":{code}:");
            if text.contains(&pattern) {
                return true;
            }
        }
        false
    }
}

/// Disables emoji replacement — returns text unchanged.
///
/// Use this when emoji are not desired (e.g. in plain-text exports or
/// legacy terminals).
pub struct NoEmoji;

impl NoEmoji {
    /// Returns the input text unchanged (no emoji replacement).
    pub fn replace(text: &str) -> String {
        text.to_string()
    }
}

/// Return the full emoji mapping table as a list of `(shortcode, unicode)` pairs.
///
/// Contains 100+ common emoji shortcodes used in Rich markup.
fn get_emoji_map() -> Vec<(&'static str, &'static str)> {
    vec![
        // Faces
        ("smile", "\u{1F60A}"),
        ("smiley", "\u{1F603}"),
        ("grin", "\u{1F601}"),
        ("joy", "\u{1F602}"),
        ("laughing", "\u{1F606}"),
        ("sweat_smile", "\u{1F605}"),
        ("wink", "\u{1F609}"),
        ("blush", "\u{1F60A}"),
        ("heart_eyes", "\u{1F60D}"),
        ("kiss", "\u{1F618}"),
        ("thinking", "\u{1F914}"),
        ("sunglasses", "\u{1F60E}"),
        ("neutral", "\u{1F610}"),
        ("expressionless", "\u{1F611}"),
        ("confused", "\u{1F615}"),
        ("worried", "\u{1F61F}"),
        ("frowning", "\u{1F626}"),
        ("anguished", "\u{1F627}"),
        ("cry", "\u{1F622}"),
        ("sob", "\u{1F62D}"),
        ("angry", "\u{1F620}"),
        ("rage", "\u{1F621}"),
        ("sleeping", "\u{1F634}"),
        ("mask", "\u{1F637}"),
        ("nerd", "\u{1F913}"),
        ("party", "\u{1F973}"),
        ("pleading", "\u{1F97A}"),
        ("yawning", "\u{1F971}"),
        ("smirk", "\u{1F60F}"),
        ("disappointed", "\u{1F61E}"),
        // Hearts & emotions
        ("heart", "\u{2764}\u{FE0F}"),
        ("orange_heart", "\u{1F9E1}"),
        ("yellow_heart", "\u{1F49B}"),
        ("green_heart", "\u{1F49A}"),
        ("blue_heart", "\u{1F499}"),
        ("purple_heart", "\u{1F49C}"),
        ("black_heart", "\u{1F5A4}"),
        ("broken_heart", "\u{1F494}"),
        ("sparkles", "\u{2728}"),
        ("star", "\u{2B50}"),
        ("glowing_star", "\u{1F31F}"),
        ("dizzy", "\u{1F4AB}"),
        ("boom", "\u{1F4A5}"),
        ("fire", "\u{1F525}"),
        ("zap", "\u{26A1}"),
        // Objects & symbols
        ("check", "\u{2705}"),
        ("cross", "\u{274C}"),
        ("warning", "\u{26A0}\u{FE0F}"),
        ("info", "\u{2139}\u{FE0F}"),
        ("100", "\u{1F4AF}"),
        ("bulb", "\u{1F4A1}"),
        ("gear", "\u{2699}\u{FE0F}"),
        ("wrench", "\u{1F527}"),
        ("lock", "\u{1F512}"),
        ("unlock", "\u{1F513}"),
        ("key", "\u{1F511}"),
        ("bell", "\u{1F514}"),
        ("book", "\u{1F4D6}"),
        ("pencil", "\u{270F}\u{FE0F}"),
        ("paperclip", "\u{1F4CE}"),
        ("scissors", "\u{2702}\u{FE0F}"),
        ("pin", "\u{1F4CC}"),
        ("email", "\u{1F4E7}"),
        ("phone", "\u{1F4DE}"),
        ("calendar", "\u{1F4C5}"),
        ("clock", "\u{1F550}"),
        ("hourglass", "\u{23F3}"),
        ("stopwatch", "\u{23F1}\u{FE0F}"),
        ("timer", "\u{23F2}\u{FE0F}"),
        ("microscope", "\u{1F52C}"),
        ("bug", "\u{1F41B}"),
        ("rocket", "\u{1F680}"),
        ("satellite", "\u{1F4E1}"),
        // Gestures & people
        ("thumbs_up", "\u{1F44D}"),
        ("thumbs_down", "\u{1F44E}"),
        ("clap", "\u{1F44F}"),
        ("wave", "\u{1F44B}"),
        ("pray", "\u{1F64F}"),
        ("muscle", "\u{1F4AA}"),
        ("raised_hands", "\u{1F64C}"),
        ("point_up", "\u{261D}\u{FE0F}"),
        ("point_down", "\u{1F447}"),
        ("point_left", "\u{1F448}"),
        ("point_right", "\u{1F449}"),
        // Arrows
        ("arrow_right", "\u{27A1}\u{FE0F}"),
        ("arrow_left", "\u{2B05}\u{FE0F}"),
        ("arrow_up", "\u{2B06}\u{FE0F}"),
        ("arrow_down", "\u{2B07}\u{FE0F}"),
        ("arrow_up_down", "\u{2195}\u{FE0F}"),
        ("left_right_arrow", "\u{2194}\u{FE0F}"),
        ("twisted_rightwards_arrows", "\u{1F500}"),
        ("repeat", "\u{1F501}"),
        // Nature & animals
        ("beetle", "\u{1FAB2}"),
        ("ant", "\u{1F41C}"),
        ("bee", "\u{1F41D}"),
        ("snake", "\u{1F40D}"),
        ("turtle", "\u{1F422}"),
        ("rabbit", "\u{1F430}"),
        ("fox", "\u{1F98A}"),
        ("dog", "\u{1F436}"),
        ("cat", "\u{1F431}"),
        ("paw_prints", "\u{1F43E}"),
        ("sunflower", "\u{1F33B}"),
        ("cherry_blossom", "\u{1F338}"),
        ("seedling", "\u{1F331}"),
        ("globe", "\u{1F30E}"),
        ("moon", "\u{1F319}"),
        ("rainbow", "\u{1F308}"),
        // Misc
        ("tada", "\u{1F389}"),
        ("confetti", "\u{1F38A}"),
        ("gift", "\u{1F381}"),
        ("crown", "\u{1F451}"),
        ("medal", "\u{1F3C5}"),
        ("trophy", "\u{1F3C6}"),
        ("moneybag", "\u{1F4B0}"),
        ("chart", "\u{1F4CA}"),
        ("bar_chart", "\u{1F4CA}"),
        ("clipboard", "\u{1F4CB}"),
        ("page_facing_up", "\u{1F4C4}"),
        ("folder", "\u{1F4C1}"),
        ("open_folder", "\u{1F4C2}"),
        ("computer", "\u{1F4BB}"),
        ("printer", "\u{1F5A8}\u{FE0F}"),
        ("game_die", "\u{1F3B2}"),
        ("alien", "\u{1F47D}"),
        ("robot", "\u{1F916}"),
        ("ghost", "\u{1F47B}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_replacement() {
        let result = Emoji::replace("Hello :smile:!");
        assert!(result.contains('\u{1F60A}'));
        assert!(!result.contains(":smile:"));
    }

    #[test]
    fn test_multiple_replacements() {
        let result = Emoji::replace(":smile: and :heart:");
        assert!(result.contains('\u{1F60A}'));
        assert!(result.contains('\u{2764}'));
    }

    #[test]
    fn test_no_replacement() {
        let result = Emoji::replace("Hello world");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_has_emoji_true() {
        assert!(Emoji::has_emoji("Hello :smile:"));
    }

    #[test]
    fn test_has_emoji_false() {
        assert!(!Emoji::has_emoji("Hello world"));
    }

    #[test]
    fn test_no_emoji() {
        let result = NoEmoji::replace("Hello :smile:");
        assert_eq!(result, "Hello :smile:");
    }

    #[test]
    fn test_rocket() {
        let result = Emoji::replace("Launch :rocket:");
        assert!(result.contains('\u{1F680}'));
    }

    #[test]
    fn test_fire() {
        let result = Emoji::replace(":fire:");
        assert!(result.contains('\u{1F525}'));
    }

    #[test]
    fn test_unknown_shortcode() {
        // Unknown shortcodes should pass through unchanged
        let result = Emoji::replace(":unknown_code:");
        assert!(result.contains(":unknown_code:"));
    }
}
