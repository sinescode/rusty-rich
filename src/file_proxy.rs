//! File proxy — auto-refreshing display of file content.

use crate::console::{ConsoleOptions, RenderResult, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// A renderable that displays the contents of a file and auto-refreshes.
#[derive(Debug, Clone)]
pub struct FileProxy {
    path: PathBuf,
    content: String,
    last_modified: Option<SystemTime>,
    max_lines: Option<usize>,
    style: Style,
}

impl FileProxy {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let content = fs::read_to_string(&path).unwrap_or_default();
        let last_modified = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        Self {
            path,
            content,
            last_modified,
            max_lines: None,
            style: Style::new(),
        }
    }

    /// Set maximum lines to display.
    pub fn max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }

    /// Set display style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Refresh content from disk.
    pub fn refresh(&mut self) -> bool {
        if let Ok(meta) = fs::metadata(&self.path) {
            if let Ok(modified) = meta.modified() {
                if self.last_modified.is_none_or(|last| modified > last) {
                    if let Ok(content) = fs::read_to_string(&self.path) {
                        self.content = content;
                        self.last_modified = Some(modified);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get current content.
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Renderable for FileProxy {
    fn render(&self, _options: &ConsoleOptions) -> RenderResult {
        let lines: Vec<&str> = self.content.lines().collect();
        let displayed = if let Some(max) = self.max_lines {
            &lines[..lines.len().min(max)]
        } else {
            &lines
        };

        let seg_lines: Vec<Vec<Segment>> = displayed
            .iter()
            .map(|line| vec![Segment::new(line.to_string())])
            .collect();

        RenderResult {
            lines: seg_lines,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_file_proxy() {
        let dir = std::env::temp_dir();
        let path = dir.join("rusty_rich_test_file_proxy.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "line1\nline2\nline3").unwrap();
        drop(f);

        let proxy = FileProxy::new(&path).max_lines(2);
        let content = proxy.content();
        assert!(content.contains("line1"));

        let _ = std::fs::remove_file(&path);
    }
}
