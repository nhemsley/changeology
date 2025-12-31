//! Diff Text View - Core component for rendering text diffs
//!
//! This component displays the diff between two text strings with colored
//! backgrounds indicating added, deleted, and unchanged lines.

use buffer_diff::{DiffHunkStatus, DiffLineType, TextDiff};
use gpui::{
    div, prelude::*, px, Context, Hsla, IntoElement, Render, SharedString, Window,
};

use crate::theme::DiffTheme;

/// Style for a single display line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineStyle {
    /// Line exists in both versions (no change)
    Unchanged,
    /// Line was added (only in new version)
    Added,
    /// Line was deleted (only in old version)
    Deleted,
}

/// A single line to display in the diff view
#[derive(Debug, Clone)]
pub struct DiffDisplayLine {
    /// The actual text content of the line
    pub content: SharedString,
    /// How to style this line
    pub style: DiffLineStyle,
}

impl DiffDisplayLine {
    pub fn new(content: impl Into<SharedString>, style: DiffLineStyle) -> Self {
        Self {
            content: content.into(),
            style,
        }
    }

    pub fn unchanged(content: impl Into<SharedString>) -> Self {
        Self::new(content, DiffLineStyle::Unchanged)
    }

    pub fn added(content: impl Into<SharedString>) -> Self {
        Self::new(content, DiffLineStyle::Added)
    }

    pub fn deleted(content: impl Into<SharedString>) -> Self {
        Self::new(content, DiffLineStyle::Deleted)
    }
}

/// The main diff text view component
///
/// This component takes two text strings and displays their differences
/// with colored backgrounds.
pub struct DiffTextView {
    /// The original (old) text
    old_text: String,
    /// The new text
    new_text: String,
    /// Lines to display, computed from the diff
    display_lines: Vec<DiffDisplayLine>,
    /// Theme for colors
    theme: DiffTheme,
}

impl DiffTextView {
    /// Create a new diff text view from two text strings
    pub fn new(old_text: &str, new_text: &str) -> Self {
        let mut view = Self {
            old_text: old_text.to_string(),
            new_text: new_text.to_string(),
            display_lines: Vec::new(),
            theme: DiffTheme::dark(),
        };
        view.compute_display_lines();
        view
    }

    /// Set the theme
    pub fn with_theme(mut self, theme: DiffTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Update the diff with new text
    pub fn update(&mut self, old_text: &str, new_text: &str) {
        self.old_text = old_text.to_string();
        self.new_text = new_text.to_string();
        self.compute_display_lines();
    }

    /// Compute the display lines from the diff
    fn compute_display_lines(&mut self) {
        self.display_lines.clear();

        // Calculate the diff using our buffer-diff crate
        let diff_result = TextDiff::diff(&self.old_text, &self.new_text);

        let diff = match diff_result {
            Ok(d) => d,
            Err(_) => {
                // If diff fails, just show the new text as-is
                for line in self.new_text.lines() {
                    self.display_lines.push(DiffDisplayLine::unchanged(line.to_string()));
                }
                return;
            }
        };

        let snapshot = diff.snapshot();
        let old_lines: Vec<&str> = self.old_text.lines().collect();
        let new_lines: Vec<&str> = self.new_text.lines().collect();

        // Process each hunk
        for hunk in snapshot.hunks() {
            match hunk.status {
                DiffHunkStatus::Unchanged => {
                    // Show unchanged lines from the new text
                    let start = hunk.new_range.start;
                    let end = hunk.new_range.end();
                    for i in start..end {
                        if i < new_lines.len() {
                            self.display_lines
                                .push(DiffDisplayLine::unchanged(new_lines[i].to_string()));
                        }
                    }
                }
                DiffHunkStatus::Added => {
                    // Show added lines with green background
                    let start = hunk.new_range.start;
                    let end = hunk.new_range.end();
                    for i in start..end {
                        if i < new_lines.len() {
                            self.display_lines
                                .push(DiffDisplayLine::added(new_lines[i].to_string()));
                        }
                    }
                }
                DiffHunkStatus::Deleted => {
                    // Show deleted lines with red background
                    let start = hunk.old_range.start;
                    let end = hunk.old_range.end();
                    for i in start..end {
                        if i < old_lines.len() {
                            self.display_lines
                                .push(DiffDisplayLine::deleted(old_lines[i].to_string()));
                        }
                    }
                }
                DiffHunkStatus::Modified => {
                    // For modified hunks, use line_types to show individual changes
                    // line_types tells us exactly which lines are old-only, new-only, or both

                    let mut old_idx = hunk.old_range.start;
                    let mut new_idx = hunk.new_range.start;

                    for line_type in &hunk.line_types {
                        match line_type {
                            DiffLineType::Both => {
                                // Line exists in both - show as unchanged from new text
                                if new_idx < new_lines.len() {
                                    self.display_lines
                                        .push(DiffDisplayLine::unchanged(new_lines[new_idx].to_string()));
                                }
                                old_idx += 1;
                                new_idx += 1;
                            }
                            DiffLineType::OldOnly => {
                                // Line only in old - show as deleted
                                if old_idx < old_lines.len() {
                                    self.display_lines
                                        .push(DiffDisplayLine::deleted(old_lines[old_idx].to_string()));
                                }
                                old_idx += 1;
                            }
                            DiffLineType::NewOnly => {
                                // Line only in new - show as added
                                if new_idx < new_lines.len() {
                                    self.display_lines
                                        .push(DiffDisplayLine::added(new_lines[new_idx].to_string()));
                                }
                                new_idx += 1;
                            }
                        }
                    }
                }
            }
        }

        // If no hunks were produced but we have text, show it unchanged
        if self.display_lines.is_empty() && !self.new_text.is_empty() {
            for line in self.new_text.lines() {
                self.display_lines.push(DiffDisplayLine::unchanged(line.to_string()));
            }
        }
    }

    /// Get the background color for a line style
    fn line_background(&self, style: DiffLineStyle) -> Hsla {
        match style {
            DiffLineStyle::Unchanged => self.theme.editor_background,
            DiffLineStyle::Added => self.theme.added_line_background,
            DiffLineStyle::Deleted => self.theme.deleted_line_background,
        }
    }

    /// Render a single line
    fn render_line(&self, line: &DiffDisplayLine) -> impl IntoElement {
        // Add a prefix indicator for the line type
        let prefix = match line.style {
            DiffLineStyle::Unchanged => "  ",
            DiffLineStyle::Added => "+ ",
            DiffLineStyle::Deleted => "- ",
        };

        let content = if line.content.is_empty() {
            // For empty lines, still show the prefix and some space
            format!("{}", prefix)
        } else {
            format!("{}{}", prefix, line.content)
        };

        div()
            .w_full()
            .px_2()
            .py(px(1.0))
            .bg(self.line_background(line.style))
            .text_color(self.theme.text)
            .font_family("monospace")
            .text_sm()
            .child(content)
    }
}

impl Render for DiffTextView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("diff-text-view")
            .size_full()
            .overflow_y_scroll()
            .bg(self.theme.editor_background)
            .border_1()
            .border_color(self.theme.border)
            .children(self.display_lines.iter().map(|line| self.render_line(line)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_display_line_creation() {
        let line = DiffDisplayLine::added("hello");
        assert_eq!(line.style, DiffLineStyle::Added);
        assert_eq!(line.content.as_ref(), "hello");
    }

    #[test]
    fn test_diff_text_view_creation() {
        let old = "line1\nline2\n";
        let new = "line1\nline3\n";
        let view = DiffTextView::new(old, new);

        // Should have computed some display lines
        assert!(!view.display_lines.is_empty());
    }

    #[test]
    fn test_identical_texts() {
        let text = "same\ntext\n";
        let view = DiffTextView::new(text, text);

        // All lines should be unchanged
        for line in &view.display_lines {
            assert_eq!(line.style, DiffLineStyle::Unchanged);
        }
    }

    #[test]
    fn test_added_lines() {
        let old = "";
        let new = "new line\n";
        let view = DiffTextView::new(old, new);

        // Should have added lines
        assert!(view
            .display_lines
            .iter()
            .any(|l| l.style == DiffLineStyle::Added));
    }

    #[test]
    fn test_deleted_lines() {
        let old = "old line\n";
        let new = "";
        let view = DiffTextView::new(old, new);

        // Should have deleted lines
        assert!(view
            .display_lines
            .iter()
            .any(|l| l.style == DiffLineStyle::Deleted));
    }
}
