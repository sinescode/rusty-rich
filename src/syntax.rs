//! Syntax highlighting — equivalent to Rich's `syntax.py`.
//!
//! Uses `syntect` for syntax highlighting (Rust equivalent of Pygments).

use std::collections::HashMap;
use std::path::Path;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::color::Color;
use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;

/// A syntax-highlighted source code renderable.
#[derive(Debug, Clone)]
pub struct Syntax {
    /// The source code.
    pub code: String,
    /// The language name (e.g. "rust", "python", "javascript").
    pub language: String,
    /// Optional theme name.
    pub theme: String,
    /// Starting line number (for line numbers).
    pub start_line: usize,
    /// If true, show line numbers.
    pub line_numbers: bool,
    /// If true, highlight the code.
    pub highlight: bool,
    /// Optional background color.
    pub background_color: Option<crate::color::Color>,
    /// Tab size.
    pub tab_size: usize,
    /// Per-line styles for line range highlighting (used by `stylize_range`).
    pub line_styles: HashMap<usize, Style>,
}

impl Syntax {
    /// Create a new Syntax renderable for the given code and language.
    pub fn new(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language: language.into(),
            theme: "base16-ocean.dark".to_string(),
            start_line: 1,
            line_numbers: false,
            highlight: true,
            background_color: None,
            tab_size: 4,
            line_styles: HashMap::new(),
        }
    }

    /// Builder: set the syntect theme name (e.g. `"base16-ocean.dark"`, `"monokai"`).
    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    /// Builder: enable line numbers in the rendered output.
    pub fn line_numbers(mut self) -> Self {
        self.line_numbers = true;
        self
    }

    /// Builder: set the starting line number for display (default 1).
    pub fn start_line(mut self, n: usize) -> Self {
        self.start_line = n;
        self
    }

    /// Builder: set a background color for the code block.
    pub fn background(mut self, color: crate::color::Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Create a Syntax from a file path, auto-detecting the language from the extension.
    ///
    /// Reads the file contents and infers the programming language from the
    /// file extension. Optionally enables line numbers and sets a theme.
    ///
    /// # Errors
    ///
    /// Returns an IO error if the file cannot be read.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rusty_rich::Syntax;
    ///
    /// let syntax = Syntax::from_path("main.rs", true, Some("monokai")).unwrap();
    /// ```
    pub fn from_path(
        path: impl AsRef<Path>,
        line_numbers: bool,
        theme: Option<&str>,
    ) -> std::io::Result<Self> {
        let path = path.as_ref();
        let code = std::fs::read_to_string(path)?;
        let language = Self::guess_lexer(path).unwrap_or_default();
        let mut syntax = Syntax::new(code, language);
        if line_numbers {
            syntax = syntax.line_numbers();
        }
        if let Some(t) = theme {
            syntax = syntax.theme(t);
        }
        Ok(syntax)
    }

    /// Guess the syntax lexer name from a file path's extension.
    ///
    /// Delegates to [`guess_lexer_for_filename`] by extracting the file stem
    /// and extension from the provided path.
    pub fn guess_lexer(path: impl AsRef<Path>) -> Option<String> {
        guess_lexer_for_filename(path.as_ref().to_str()?)
    }

    /// Apply a background style to a range of lines (for highlighting).
    ///
    /// Returns a new [`Syntax`] with the style applied to the specified lines
    /// (1-based, inclusive). This is useful for highlighting a specific range
    /// of lines, e.g. the current line in a debugger.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rusty_rich::{Syntax, Style, Color};
    ///
    /// let syntax = Syntax::new("line1\nline2", "text")
    ///     .stylize_range(1, 1, Style::new().bgcolor(Color::from_rgb(255, 255, 200)));
    /// ```
    pub fn stylize_range(mut self, start_line: usize, end_line: usize, style: Style) -> Self {
        for line in start_line..=end_line {
            self.line_styles.insert(line, style.clone());
        }
        self
    }

    /// Get the current theme name.
    pub fn get_theme(&self) -> &str {
        &self.theme
    }

    /// Return the default lexer name (`"text"`).
    pub fn default_lexer() -> &'static str {
        "text"
    }
}

impl Renderable for Syntax {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        if !self.highlight || self.language.is_empty() {
            // No highlighting — just render as plain text
            let mut lines: Vec<Vec<Segment>> = self
                .code
                .lines()
                .map(|line| vec![Segment::new(line), Segment::line()])
                .collect();

            // Apply per-line styles
            apply_line_styles(&mut lines, self.start_line, &self.line_styles);

            return RenderResult {
                lines,
                items: Vec::new(),
            };
        }

        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let syntax = ss
            .find_syntax_by_name(&self.language)
            .or_else(|| ss.find_syntax_by_extension(&self.language))
            .unwrap_or_else(|| ss.find_syntax_plain_text());

        let theme = ts
            .themes
            .get(&self.theme)
            .unwrap_or_else(|| &ts.themes["base16-ocean.dark"]);

        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let line_num_width = if self.line_numbers {
            (self.code.lines().count().saturating_add(self.start_line))
                .to_string()
                .len()
        } else {
            0
        };

        for (i, line) in LinesWithEndings::from(&self.code).enumerate() {
            let mut line_segments: Vec<Segment> = Vec::new();

            // Line number
            if self.line_numbers {
                let num = i + self.start_line;
                let num_str = format!("{:>width$} │ ", num, width = line_num_width);
                line_segments.push(Segment::new(num_str));
            }

            // Highlight the line
            match highlighter.highlight_line(line, &ss) {
                Ok(highlighted) => {
                    for (syntect_style, text) in &highlighted {
                        let style = syntect_to_rich_style(syntect_style);
                        line_segments.push(Segment::styled(text.to_string(), style));
                    }
                }
                Err(_) => {
                    line_segments.push(Segment::new(line));
                }
            }

            lines.push(line_segments);
        }

        // Apply per-line styles
        apply_line_styles(&mut lines, self.start_line, &self.line_styles);

        RenderResult {
            lines,
            items: Vec::new(),
        }
    }
}

/// Apply per-line styles to rendered segment lines.
///
/// For each line that has a matching style in `line_styles`, the background
/// color from that style is applied to every segment on that line.
fn apply_line_styles(
    lines: &mut [Vec<Segment>],
    start_line: usize,
    line_styles: &HashMap<usize, Style>,
) {
    if line_styles.is_empty() {
        return;
    }
    for (i, line) in lines.iter_mut().enumerate() {
        let line_num = start_line + i;
        if let Some(style) = line_styles.get(&line_num) {
            if let Some(bg) = style.bgcolor {
                for seg in line.iter_mut() {
                    if let Some(ref mut s) = seg.style {
                        s.bgcolor = Some(bg);
                    } else {
                        seg.style = Some(Style::new().bgcolor(bg));
                    }
                }
            }
        }
    }
}

/// Convert a syntect `Style` to our `Style`.
fn syntect_to_rich_style(ss: &SyntectStyle) -> Style {
    let mut style = Style::new();
    let fg = ss.foreground;
    style = style.color(crate::color::Color::from_rgb(fg.r, fg.g, fg.b));

    if ss
        .font_style
        .contains(syntect::highlighting::FontStyle::BOLD)
    {
        style = style.bold(true);
    }
    if ss
        .font_style
        .contains(syntect::highlighting::FontStyle::ITALIC)
    {
        style = style.italic(true);
    }
    if ss
        .font_style
        .contains(syntect::highlighting::FontStyle::UNDERLINE)
    {
        style = style.underline(true);
    }
    style
}

/// A syntax theme that maps to ANSI colors (lightweight, no Pygments dependency).
///
/// Provides a simple token-to-style mapping for common syntax token types
/// like "keyword", "string", "comment", "number", "type", and "function".
/// Pre-built themes are available via [`ANSISyntaxTheme::monokai`] and
/// [`ANSISyntaxTheme::default_light`].
#[derive(Debug, Clone)]
pub struct ANSISyntaxTheme {
    /// Optional background color for the code block.
    pub background: Option<Color>,
    /// Optional default foreground color.
    pub foreground: Option<Color>,
    /// Token name to style mapping.
    pub styles: HashMap<String, Style>,
}

impl ANSISyntaxTheme {
    /// Create a new empty `ANSISyntaxTheme`.
    pub fn new() -> Self {
        Self {
            background: None,
            foreground: None,
            styles: HashMap::new(),
        }
    }

    /// Set the style for a token type.
    ///
    /// Common token names include: `"comment"`, `"keyword"`, `"string"`,
    /// `"number"`, `"type"`, `"function"`.
    pub fn set(&mut self, token: &str, style: Style) {
        self.styles.insert(token.to_string(), style);
    }

    /// Get the style for a token type, if one has been set.
    pub fn get(&self, token: &str) -> Option<&Style> {
        self.styles.get(token)
    }

    /// Create a Monokai-inspired theme.
    ///
    /// Features a dark background with vibrant foreground colors
    /// commonly associated with the Monokai color scheme.
    pub fn monokai() -> Self {
        let mut theme = Self::new();
        theme.background = Some(Color::from_rgb(39, 40, 34));
        theme.foreground = Some(Color::from_rgb(248, 248, 242));
        theme.set("comment", Style::new().color(Color::from_rgb(117, 113, 94)));
        theme.set("keyword", Style::new().color(Color::from_rgb(249, 38, 114)));
        theme.set("string", Style::new().color(Color::from_rgb(230, 219, 116)));
        theme.set("number", Style::new().color(Color::from_rgb(174, 129, 255)));
        theme.set("type", Style::new().color(Color::from_rgb(102, 217, 239)));
        theme.set(
            "function",
            Style::new().color(Color::from_rgb(166, 226, 46)),
        );
        theme
    }

    /// Create a default light theme.
    ///
    /// Provides a white background with blue keywords, red strings,
    /// and navy numbers — a familiar light-mode syntax scheme.
    pub fn default_light() -> Self {
        let mut theme = Self::new();
        theme.background = Some(Color::from_rgb(255, 255, 255));
        theme.foreground = Some(Color::from_rgb(0, 0, 0));
        theme.set("comment", Style::new().color(Color::from_rgb(0, 128, 0)));
        theme.set("keyword", Style::new().color(Color::from_rgb(0, 0, 255)));
        theme.set("string", Style::new().color(Color::from_rgb(163, 21, 21)));
        theme.set("number", Style::new().color(Color::from_rgb(0, 0, 128)));
        theme.set("type", Style::new().color(Color::from_rgb(128, 128, 0)));
        theme.set("function", Style::new().color(Color::from_rgb(128, 0, 128)));
        theme
    }
}

impl Default for ANSISyntaxTheme {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for syntax themes.
///
/// Implementors provide token-to-style mappings and an optional
/// background color for syntax-highlighted code blocks.
pub trait SyntaxTheme {
    /// Get the style for a given token type (e.g. `"keyword"`, `"string"`).
    fn get_style(&self, token: &str) -> Option<Style>;
    /// Get the optional background color for the entire code block.
    fn background_color(&self) -> Option<Color>;
}

impl SyntaxTheme for ANSISyntaxTheme {
    fn get_style(&self, token: &str) -> Option<Style> {
        self.styles.get(token).cloned()
    }

    fn background_color(&self) -> Option<Color> {
        self.background
    }
}

/// Resolve a lexer name (case-insensitive) to a canonical name.
///
/// Supports common short aliases:
///
/// | Alias | Canonical |
/// |-------|-----------|
/// | `py` | `python` |
/// | `rs` | `rust` |
/// | `js` | `javascript` |
/// | `ts` | `typescript` |
/// | `cpp` | `cpp` |
/// | `rb` | `ruby` |
/// | `md` | `markdown` |
/// | `sh` / `bash` | `bash` |
///
/// If no alias matches, returns the input as-is so that syntect can attempt
/// to resolve it natively.
pub fn get_lexer_by_name(name: &str) -> Option<String> {
    match name.to_lowercase().as_str() {
        "py" => Some("python".to_string()),
        "rs" => Some("rust".to_string()),
        "js" => Some("javascript".to_string()),
        "ts" => Some("typescript".to_string()),
        "cpp" => Some("c++".to_string()),
        "rb" => Some("ruby".to_string()),
        "md" => Some("markdown".to_string()),
        "sh" | "bash" => Some("bash".to_string()),
        "yml" | "yaml" => Some("yaml".to_string()),
        _ => Some(name.to_string()),
    }
}

/// Get a pre-built [`ANSISyntaxTheme`] by name.
///
/// Supported names: `"monokai"`, `"light"`, `"nord"`, `"dracula"`, `"github"`.
///
/// Returns `None` for unrecognized theme names.
pub fn get_style_by_name(name: &str) -> Option<ANSISyntaxTheme> {
    match name.to_lowercase().as_str() {
        "monokai" => Some(ANSISyntaxTheme::monokai()),
        "light" => Some(ANSISyntaxTheme::default_light()),
        "nord" => {
            let mut theme = ANSISyntaxTheme::new();
            theme.background = Some(Color::from_rgb(46, 52, 64));
            theme.foreground = Some(Color::from_rgb(216, 222, 233));
            theme.set("comment", Style::new().color(Color::from_rgb(76, 86, 106)));
            theme.set(
                "keyword",
                Style::new().color(Color::from_rgb(143, 188, 187)),
            );
            theme.set("string", Style::new().color(Color::from_rgb(163, 190, 140)));
            theme.set("number", Style::new().color(Color::from_rgb(208, 135, 112)));
            theme.set("type", Style::new().color(Color::from_rgb(136, 192, 208)));
            theme.set(
                "function",
                Style::new().color(Color::from_rgb(129, 161, 193)),
            );
            Some(theme)
        }
        "dracula" => {
            let mut theme = ANSISyntaxTheme::new();
            theme.background = Some(Color::from_rgb(40, 42, 54));
            theme.foreground = Some(Color::from_rgb(248, 248, 242));
            theme.set("comment", Style::new().color(Color::from_rgb(98, 114, 164)));
            theme.set(
                "keyword",
                Style::new().color(Color::from_rgb(255, 121, 198)),
            );
            theme.set("string", Style::new().color(Color::from_rgb(241, 250, 140)));
            theme.set("number", Style::new().color(Color::from_rgb(189, 147, 249)));
            theme.set("type", Style::new().color(Color::from_rgb(139, 233, 253)));
            theme.set(
                "function",
                Style::new().color(Color::from_rgb(80, 250, 123)),
            );
            Some(theme)
        }
        "github" => {
            let mut theme = ANSISyntaxTheme::new();
            theme.background = Some(Color::from_rgb(255, 255, 255));
            theme.foreground = Some(Color::from_rgb(36, 41, 46));
            theme.set(
                "comment",
                Style::new().color(Color::from_rgb(106, 115, 125)),
            );
            theme.set("keyword", Style::new().color(Color::from_rgb(215, 58, 73)));
            theme.set("string", Style::new().color(Color::from_rgb(3, 47, 98)));
            theme.set("number", Style::new().color(Color::from_rgb(0, 92, 197)));
            theme.set("type", Style::new().color(Color::from_rgb(227, 98, 9)));
            theme.set(
                "function",
                Style::new().color(Color::from_rgb(111, 66, 193)),
            );
            Some(theme)
        }
        _ => None,
    }
}

/// Guess the syntax lexer name from a filename or file path.
///
/// Maps common file extensions to their corresponding lexer names:
///
/// | Extension | Lexer |
/// |-----------|-------|
/// | `.rs` | `rust` |
/// | `.py` | `python` |
/// | `.js` | `javascript` |
/// | `.ts` | `typescript` |
/// | `.java` | `java` |
/// | `.go` | `go` |
/// | `.rb` | `ruby` |
/// | `.php` | `php` |
/// | `.c`, `.h` | `c` |
/// | `.cpp`, `.hpp` | `c++` |
/// | `.cs` | `csharp` |
/// | `.html` | `html` |
/// | `.css` | `css` |
/// | `.scss` | `scss` |
/// | `.json` | `json` |
/// | `.xml` | `xml` |
/// | `.yaml`, `.yml` | `yaml` |
/// | `.md` | `markdown` |
/// | `.sql` | `sql` |
/// | `.sh`, `.bash` | `bash` |
/// | `.toml` | `toml` |
/// | `.ini`, `.cfg` | `ini` |
/// | `Dockerfile` | `dockerfile` |
/// | `Makefile` | `makefile` |
///
/// Returns `None` for unrecognized filenames.
pub fn guess_lexer_for_filename(filename: &str) -> Option<String> {
    let name = filename.trim();
    // Check for well-known filenames without extensions
    if name.eq_ignore_ascii_case("Dockerfile") {
        return Some("dockerfile".to_string());
    }
    if name.eq_ignore_ascii_case("Makefile") {
        return Some("makefile".to_string());
    }
    // Extract the extension
    let path = Path::new(name);
    let ext = path.extension()?.to_str()?;
    match ext.to_lowercase().as_str() {
        "rs" => Some("rust".to_string()),
        "py" => Some("python".to_string()),
        "js" => Some("javascript".to_string()),
        "ts" => Some("typescript".to_string()),
        "java" => Some("java".to_string()),
        "go" => Some("go".to_string()),
        "rb" => Some("ruby".to_string()),
        "php" => Some("php".to_string()),
        "c" | "h" => Some("c".to_string()),
        "cpp" | "hpp" | "cxx" | "hxx" => Some("c++".to_string()),
        "cs" => Some("csharp".to_string()),
        "html" | "htm" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        "scss" | "sass" => Some("scss".to_string()),
        "json" => Some("json".to_string()),
        "xml" | "svg" | "xhtml" => Some("xml".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "md" | "markdown" => Some("markdown".to_string()),
        "sql" => Some("sql".to_string()),
        "sh" | "bash" | "zsh" | "ksh" => Some("bash".to_string()),
        "toml" => Some("toml".to_string()),
        "ini" | "cfg" | "conf" => Some("ini".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_no_highlight() {
        let s = Syntax::new("fn main() {}", "rust");
        let opts = ConsoleOptions::default();
        let result = s.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("fn main"));
    }

    #[test]
    fn test_syntax_line_numbers() {
        let s = Syntax::new("line1\nline2\nline3", "").line_numbers();
        let opts = ConsoleOptions::default();
        let result = s.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("1"));
    }

    #[test]
    fn test_from_path() {
        use std::io::Write;
        let path = std::env::temp_dir().join("rusty_rich_test_syntax_from_path.rs");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "fn main() {{}}").unwrap();
        let syntax = Syntax::from_path(&path, false, None).unwrap();
        assert_eq!(syntax.language, "rust");
        assert!(!syntax.line_numbers);
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_from_path_with_theme() {
        use std::io::Write;
        let path = std::env::temp_dir().join("app.py");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "print('hello')").unwrap();
        let syntax = Syntax::from_path(&path, true, Some("monokai")).unwrap();
        assert_eq!(syntax.language, "python");
        assert!(syntax.line_numbers);
        assert_eq!(syntax.theme, "monokai");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_default_lexer() {
        assert_eq!(Syntax::default_lexer(), "text");
    }

    #[test]
    fn test_get_theme() {
        let s = Syntax::new("test", "rust").theme("monokai");
        assert_eq!(s.get_theme(), "monokai");
    }

    #[test]
    fn test_guess_lexer_for_filename() {
        assert_eq!(
            guess_lexer_for_filename("main.rs"),
            Some("rust".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("app.py"),
            Some("python".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("Dockerfile"),
            Some("dockerfile".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("Makefile"),
            Some("makefile".to_string())
        );
        assert_eq!(guess_lexer_for_filename("unknown.xyz"), None);
    }

    #[test]
    fn test_guess_lexer_for_filename_edge_cases() {
        assert_eq!(
            guess_lexer_for_filename("/path/to/script.sh"),
            Some("bash".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("/path/to/config.yaml"),
            Some("yaml".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("/path/to/file.cpp"),
            Some("c++".to_string())
        );
    }

    #[test]
    fn test_get_lexer_by_name() {
        assert_eq!(get_lexer_by_name("py"), Some("python".to_string()));
        assert_eq!(get_lexer_by_name("rs"), Some("rust".to_string()));
        assert_eq!(get_lexer_by_name("js"), Some("javascript".to_string()));
        assert_eq!(get_lexer_by_name("cpp"), Some("c++".to_string()));
    }

    #[test]
    fn test_get_lexer_by_name_passthrough() {
        // Unknown short names should pass through as-is
        assert_eq!(get_lexer_by_name("python"), Some("python".to_string()));
        assert_eq!(get_lexer_by_name("rust"), Some("rust".to_string()));
    }

    #[test]
    fn test_ansi_theme_monokai() {
        let theme = ANSISyntaxTheme::monokai();
        assert!(theme.background.is_some());
        assert!(theme.foreground.is_some());
        assert!(theme.get("keyword").is_some());
        assert!(theme.get("string").is_some());
        assert!(theme.get("comment").is_some());
    }

    #[test]
    fn test_ansi_theme_default_light() {
        let theme = ANSISyntaxTheme::default_light();
        assert!(theme.background.is_some());
        assert_eq!(theme.background.unwrap(), Color::from_rgb(255, 255, 255));
        assert!(theme.get("keyword").is_some());
    }

    #[test]
    fn test_stylize_range() {
        let s = Syntax::new("line1\nline2\nline3", "text").stylize_range(
            1,
            1,
            Style::new().bgcolor(Color::from_rgb(255, 0, 0)),
        );
        assert_eq!(s.line_styles.len(), 1);
        assert!(s.line_styles.contains_key(&1));
    }

    #[test]
    fn test_stylize_range_multi_line() {
        let s = Syntax::new("line1\nline2\nline3", "text").stylize_range(
            1,
            2,
            Style::new().bgcolor(Color::from_rgb(255, 255, 0)),
        );
        assert_eq!(s.line_styles.len(), 2);
        assert!(s.line_styles.contains_key(&1));
        assert!(s.line_styles.contains_key(&2));
        assert!(!s.line_styles.contains_key(&3));
    }

    #[test]
    fn test_stylize_range_renders() {
        let s = Syntax::new("hello\nworld", "text").stylize_range(
            1,
            1,
            Style::new().bgcolor(Color::from_rgb(255, 0, 0)),
        );
        let opts = ConsoleOptions::default();
        let result = s.render(&opts);
        let ansi = result.to_ansi();
        assert!(ansi.contains("hello"));
        assert!(ansi.contains("world"));
    }

    #[test]
    fn test_guess_lexer_on_syntax() {
        let path = Path::new("/tmp/test.py");
        let result = Syntax::guess_lexer(path);
        assert_eq!(result, Some("python".to_string()));
    }

    #[test]
    fn test_get_style_by_name() {
        let theme = get_style_by_name("monokai");
        assert!(theme.is_some());

        let theme = get_style_by_name("nord");
        assert!(theme.is_some());

        let theme = get_style_by_name("dracula");
        assert!(theme.is_some());

        let theme = get_style_by_name("github");
        assert!(theme.is_some());

        let theme = get_style_by_name("unknown");
        assert!(theme.is_none());
    }

    #[test]
    fn test_syntax_theme_trait() {
        let theme = ANSISyntaxTheme::monokai();
        let trait_obj: &dyn SyntaxTheme = &theme;
        assert!(trait_obj.get_style("keyword").is_some());
        assert!(trait_obj.background_color().is_some());
    }

    #[test]
    fn test_guess_lexer_for_filename_case_insensitive() {
        assert_eq!(
            guess_lexer_for_filename("main.RS"),
            Some("rust".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("App.PY"),
            Some("python".to_string())
        );
        assert_eq!(
            guess_lexer_for_filename("DOCKERFILE"),
            Some("dockerfile".to_string())
        );
    }
}
