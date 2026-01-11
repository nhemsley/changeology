//! History panel - displays commit history
//!
//! This panel shows the git commit history and allows
//! selecting commits to view their diffs.

use gpui::*;
use gpui_component::{v_flex, ActiveTheme, Icon, IconName};

/// Placeholder commit data structure
#[allow(dead_code)]
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

/// History panel state
#[allow(dead_code)]
pub struct HistoryPanel {
    /// List of commits (to be populated from git)
    #[allow(dead_code)]
    commits: Vec<Commit>,
    /// Currently selected commit index
    #[allow(dead_code)]
    selected_index: Option<usize>,
}

impl HistoryPanel {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
            selected_index: None,
        }
    }

    /// Render the history panel
    #[allow(dead_code)]
    pub fn render(&self, _window: &mut Window, cx: &App) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .items_center()
            .justify_center()
            .gap_3()
            .text_color(cx.theme().muted_foreground)
            .child(
                Icon::new(IconName::Inbox)
                    .size(px(32.))
                    .text_color(cx.theme().muted_foreground),
            )
            .child("Commit history coming soon...")
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("This will show recent commits with messages, authors, and dates"),
            )
    }
}

impl Default for HistoryPanel {
    fn default() -> Self {
        Self::new()
    }
}
