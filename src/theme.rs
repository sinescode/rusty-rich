//! Theme system — equivalent to Rich's `theme.py`.
//!
//! A Theme maps style names (like "repr.number", "repr.str") to Style values,
//! allowing customizable color schemes for different renderable types.

use std::collections::HashMap;

use crate::style::Style;

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// A set of named styles that can be applied to themed output.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Mapping from style name → Style.
    pub styles: HashMap<String, Style>,
    /// Optional inherited base theme.
    pub inherit: Option<Box<Theme>>,
}

impl Theme {
    /// Create a new empty theme.
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
            inherit: None,
        }
    }

    /// Create a theme that inherits from another.
    pub fn with_inherit(inherit: Theme) -> Self {
        Self {
            styles: HashMap::new(),
            inherit: Some(Box::new(inherit)),
        }
    }

    /// Look up a style by name. Falls back to the inherited theme.
    pub fn get(&self, name: &str) -> Option<&Style> {
        self.styles
            .get(name)
            .or_else(|| self.inherit.as_ref().and_then(|i| i.get(name)))
    }

    /// Set a named style.
    pub fn set(&mut self, name: impl Into<String>, style: Style) {
        self.styles.insert(name.into(), style);
    }

    /// Merge another theme's styles into this one.
    pub fn merge(&mut self, other: &Theme) {
        for (name, style) in &other.styles {
            self.styles.entry(name.clone()).or_insert_with(|| style.clone());
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ThemeStack
// ---------------------------------------------------------------------------

/// A stack of themes — when looking up a style, themes are checked from
/// top to bottom until a match is found.
#[derive(Debug, Clone)]
pub struct ThemeStack {
    themes: Vec<Theme>,
}

impl ThemeStack {
    pub fn new() -> Self {
        Self { themes: Vec::new() }
    }

    pub fn push(&mut self, theme: Theme) {
        self.themes.push(theme);
    }

    pub fn pop(&mut self) -> Option<Theme> {
        self.themes.pop()
    }

    /// Look up a style name across the stack (top-down).
    pub fn get(&self, name: &str) -> Option<Style> {
        for theme in self.themes.iter().rev() {
            if let Some(s) = theme.get(name) {
                return Some(s.clone());
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Default themes
// ---------------------------------------------------------------------------

/// Built-in theme style names used by various Rich renderables.
pub mod names {
    // repr / pretty
    pub const REPR_NUMBER: &str = "repr.number";
    pub const REPR_STR: &str = "repr.str";
    pub const REPR_BOOL_TRUE: &str = "repr.bool_true";
    pub const REPR_BOOL_FALSE: &str = "repr.bool_false";
    pub const REPR_NONE: &str = "repr.none";
    pub const REPR_URL: &str = "repr.url";
    pub const REPR_PATH: &str = "repr.path";
    pub const REPR_IPV4: &str = "repr.ipv4";
    pub const REPR_IPV6: &str = "repr.ipv6";
    pub const REPR_ELLIPSIS: &str = "repr.ellipsis";
    pub const REPR_ATTRIB_NAME: &str = "repr.attrib_name";
    pub const REPR_ATTRIB_VALUE: &str = "repr.attrib_value";
    pub const REPR_TAG_NAME: &str = "repr.tag_name";
    pub const REPR_TAG_CONTENTS: &str = "repr.tag_contents";
    pub const REPR_TAG_PUNCTUATION: &str = "repr.tag_punctuation";

    // table
    pub const TABLE_HEADER: &str = "table.header";
    pub const TABLE_FOOTER: &str = "table.footer";
    pub const TABLE_TITLE: &str = "table.title";
    pub const TABLE_CAPTION: &str = "table.caption";
    pub const TABLE_BORDER: &str = "table.border";

    // logging
    pub const LOGGING_KEYWORD: &str = "logging.keyword";
    pub const LOGGING_LEVEL_DEBUG: &str = "logging.level.debug";
    pub const LOGGING_LEVEL_INFO: &str = "logging.level.info";
    pub const LOGGING_LEVEL_WARNING: &str = "logging.level.warning";
    pub const LOGGING_LEVEL_ERROR: &str = "logging.level.error";
    pub const LOGGING_LEVEL_CRITICAL: &str = "logging.level.critical";

    // traceback
    pub const TRACEBACK_BORDER: &str = "traceback.border";
    pub const TRACEBACK_TITLE: &str = "traceback.title";
    pub const TRACEBACK_ERROR: &str = "traceback.error";
    pub const TRACEBACK_ERROR_MARK: &str = "traceback.error_mark";
    pub const TRACEBACK_FILENAME: &str = "traceback.filename";
    pub const TRACEBACK_LINE_NO: &str = "traceback.line_no";
    pub const TRACEBACK_LOCALS_HEADER: &str = "traceback.locals_header";

    // general
    pub const RULE_LINE: &str = "rule.line";
    pub const RULE_TEXT: &str = "rule.text";
    pub const BAR_COMPLETE: &str = "bar.complete";
    pub const BAR_FINISHED: &str = "bar.finished";
    pub const BAR_PULSE: &str = "bar.pulse";
    pub const PROGRESS_DESCRIPTION: &str = "progress.description";
    pub const PROGRESS_PERCENTAGE: &str = "progress.percentage";
    pub const PROGRESS_REMAINING: &str = "progress.remaining";
    pub const PROGRESS_ELAPSED: &str = "progress.elapsed";
    pub const PROGRESS_DATA: &str = "progress.data";
    pub const TREE: &str = "tree";
    pub const TREE_LINE: &str = "tree.line";
    pub const MARKDOWN_H1: &str = "markdown.h1";
    pub const MARKDOWN_H2: &str = "markdown.h2";
    pub const MARKDOWN_CODE: &str = "markdown.code";
    pub const MARKDOWN_LINK: &str = "markdown.link";
    pub const MARKDOWN_ITEM: &str = "markdown.item";
    pub const MARKDOWN_BLOCKQUOTE: &str = "markdown.blockquote";
    pub const JSON_KEY: &str = "json.key";
    pub const JSON_STR: &str = "json.str";
    pub const JSON_NUMBER: &str = "json.number";
    pub const JSON_BOOL: &str = "json.bool";
    pub const JSON_NULL: &str = "json.null";
    pub const JSON_BRACE: &str = "json.brace";

    // syntax highlighting (via syntect)
    pub const SYNTAX_COMMENT: &str = "syntax.comment";
    pub const SYNTAX_KEYWORD: &str = "syntax.keyword";
    pub const SYNTAX_STRING: &str = "syntax.string";
    pub const SYNTAX_NUMBER: &str = "syntax.number";
    pub const SYNTAX_FUNCTION: &str = "syntax.function";
    pub const SYNTAX_TYPE: &str = "syntax.type";
}

/// Create the default Rich-like theme.
pub fn default_theme() -> Theme {
    let mut t = Theme::new();
    use crate::color::Color;
    use crate::style::Style;

    // repr styles
    t.set(names::REPR_NUMBER, Style::new().color(Color::parse("cyan").unwrap()).bold(true));
    t.set(names::REPR_STR, Style::new().color(Color::parse("green").unwrap()));
    t.set(names::REPR_BOOL_TRUE, Style::new().color(Color::parse("bright_green").unwrap()).bold(true));
    t.set(names::REPR_BOOL_FALSE, Style::new().color(Color::parse("bright_red").unwrap()).bold(true));
    t.set(names::REPR_NONE, Style::new().color(Color::parse("bright_yellow").unwrap()).dim(true));
    t.set(names::REPR_URL, Style::new().color(Color::parse("bright_blue").unwrap()).underline(true));
    t.set(names::REPR_PATH, Style::new().color(Color::parse("magenta").unwrap()));
    t.set(names::REPR_ATTRIB_NAME, Style::new().color(Color::parse("bright_cyan").unwrap()));
    t.set(names::REPR_ATTRIB_VALUE, Style::new().color(Color::parse("white").unwrap()));

    // table styles
    t.set(names::TABLE_HEADER, Style::new().bold(true).color(Color::parse("white").unwrap()));
    t.set(names::TABLE_FOOTER, Style::new().bold(true));
    t.set(names::TABLE_TITLE, Style::new().bold(true));
    t.set(names::TABLE_CAPTION, Style::new().dim(true));
    t.set(names::TABLE_BORDER, Style::new().color(Color::parse("bright_black").unwrap()));

    // rule
    t.set(names::RULE_LINE, Style::new().color(Color::parse("bright_black").unwrap()));
    t.set(names::RULE_TEXT, Style::new().bold(true));

    // tree
    t.set(names::TREE, Style::new().color(Color::parse("white").unwrap()));
    t.set(names::TREE_LINE, Style::new().color(Color::parse("bright_black").unwrap()));

    // progress
    t.set(names::BAR_COMPLETE, Style::new().color(Color::parse("green").unwrap()));
    t.set(names::BAR_FINISHED, Style::new().color(Color::parse("bright_green").unwrap()));
    t.set(names::BAR_PULSE, Style::new().color(Color::parse("bright_cyan").unwrap()));
    t.set(names::PROGRESS_DESCRIPTION, Style::new().color(Color::parse("white").unwrap()));
    t.set(names::PROGRESS_PERCENTAGE, Style::new().color(Color::parse("cyan").unwrap()));

    // markdown
    t.set(names::MARKDOWN_H1, Style::new().bold(true).color(Color::parse("bright_cyan").unwrap()));
    t.set(names::MARKDOWN_H2, Style::new().bold(true).color(Color::parse("cyan").unwrap()));
    t.set(names::MARKDOWN_CODE, Style::new().color(Color::parse("bright_yellow").unwrap()).bgcolor(Color::parse("black").unwrap()));
    t.set(names::MARKDOWN_LINK, Style::new().color(Color::parse("bright_blue").unwrap()).underline(true));

    // json
    t.set(names::JSON_KEY, Style::new().color(Color::parse("cyan").unwrap()));
    t.set(names::JSON_STR, Style::new().color(Color::parse("green").unwrap()));
    t.set(names::JSON_NUMBER, Style::new().color(Color::parse("bright_blue").unwrap()).bold(true));
    t.set(names::JSON_BOOL, Style::new().color(Color::parse("bright_yellow").unwrap()).bold(true));
    t.set(names::JSON_NULL, Style::new().color(Color::parse("bright_red").unwrap()).dim(true));
    t.set(names::JSON_BRACE, Style::new().color(Color::parse("bright_black").unwrap()));

    // traceback
    t.set(names::TRACEBACK_BORDER, Style::new().color(Color::parse("red").unwrap()));
    t.set(names::TRACEBACK_TITLE, Style::new().bold(true));
    t.set(names::TRACEBACK_ERROR, Style::new().color(Color::parse("bright_red").unwrap()).bold(true));
    t.set(names::TRACEBACK_ERROR_MARK, Style::new().color(Color::parse("bright_red").unwrap()).bold(true));
    t.set(names::TRACEBACK_FILENAME, Style::new().color(Color::parse("cyan").unwrap()));
    t.set(names::TRACEBACK_LINE_NO, Style::new().color(Color::parse("bright_black").unwrap()));
    t.set(names::TRACEBACK_LOCALS_HEADER, Style::new().bold(true));

    t
}
