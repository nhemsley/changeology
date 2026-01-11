//! Sidebar rendering for the main application
//!
//! Contains the three panel sections: Changes (dirty files), Staged, and History

use gpui::*;

use gpui_component::{
    h_flex, list::ListItem, scroll::Scrollbar, v_flex, ActiveTheme, Icon, IconName,
};

use crate::panels::file_tree;
use git::{Commit, StatusEntry};

/// Render the section header with title and count
pub fn render_section_header(title: &str, count: usize, cx: &App) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(cx.theme().muted_foreground)
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{}", count)),
                ),
        )
}

/// Render a file entry item (used for both dirty and staged files)
pub fn render_file_entry(
    id: impl Into<ElementId>,
    entry: &StatusEntry,
    is_selected: bool,
    cx: &App,
) -> ListItem {
    let status_icon = file_tree::status_indicator(entry.kind);
    let status_color = file_tree::status_color(entry.kind, cx);

    ListItem::new(id).selected(is_selected).py(px(2.)).child(
        h_flex()
            .gap_2()
            .items_center()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(status_color)
                    .child(status_icon),
            )
            .child(div().text_sm().child(entry.path.clone())),
    )
}

/// Render a commit entry item
pub fn render_commit_entry(index: usize, commit: &Commit, is_selected: bool, cx: &App) -> ListItem {
    ListItem::new(format!("commit-{}", index))
        .selected(is_selected)
        .py(px(2.))
        .child(
            v_flex()
                .w_full()
                .gap_1()
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .justify_between()
                        .child(
                            div().text_sm().flex_auto().overflow_hidden().child(
                                commit
                                    .message
                                    .lines()
                                    .next()
                                    .unwrap_or(&commit.message)
                                    .to_string(),
                            ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .flex_shrink_0()
                                .text_color(cx.theme().muted_foreground)
                                .child(commit.short_id.clone()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format_timestamp(commit.time)),
                ),
        )
}

/// Render an empty state with icon and message
pub fn render_empty_state(message: &str, cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .p_4()
        .items_center()
        .justify_center()
        .text_color(cx.theme().muted_foreground)
        .child(
            Icon::new(IconName::Inbox)
                .size(px(24.))
                .text_color(cx.theme().muted_foreground),
        )
        .child(div().text_xs().mt_2().child(message.to_string()))
}

/// Render the history panel with scrollable commit list
#[allow(dead_code)]
pub fn render_history_content(
    commits: &[Commit],
    selected_commit: Option<usize>,
    scroll_handle: &ScrollHandle,
    cx: &App,
) -> impl IntoElement {
    div()
        .id("history-scroll-area")
        .flex_1()
        .overflow_y_scroll()
        .track_scroll(scroll_handle)
        .child(if commits.is_empty() {
            render_empty_state("No commits", cx).into_any_element()
        } else {
            v_flex()
                .w_full()
                .children(commits.iter().enumerate().map(|(i, commit)| {
                    let is_selected = selected_commit == Some(i);
                    render_commit_entry(i, commit, is_selected, cx).into_any_element()
                }))
                .into_any_element()
        })
        .child(Scrollbar::vertical(scroll_handle))
}

/// Format a Unix timestamp as a human-readable relative time string
fn format_timestamp(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < 604800 {
        let days = diff / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < 2592000 {
        let weeks = diff / 604800;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if diff < 31536000 {
        let months = diff / 2592000;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = diff / 31536000;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}
