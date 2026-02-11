//! Diff Canvas View - displays file diffs on an infinite canvas
//!
//! This module provides a canvas view for displaying file diffs with
//! pan/zoom functionality and textured rendering.
//!
//! Controls:
//! - Middle mouse button: Pan the canvas
//! - Scroll wheel: Zoom in/out (centered on cursor)

use diff_ui::{DiffTextView, DiffTheme, RenderMode as DiffRenderMode};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme, Icon, IconName};
use infinite_canvas::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Diff data for a single file in a commit
#[derive(Clone)]
pub struct FileDiff {
    pub path: String,
    pub old_content: String,
    pub new_content: String,
}

/// A view that displays file diffs on an infinite canvas
pub struct DiffCanvasView {
    provider: Rc<RefCell<TexturedCanvasItemsProvider>>,
    /// The diffs currently displayed
    diffs: Vec<FileDiff>,
    /// Commit info for display
    commit_info: Option<(String, String)>, // (short_hash, message)
    /// Flag to indicate that items need to be synced to the provider
    needs_sync: bool,
}

impl DiffCanvasView {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let provider = Rc::new(RefCell::new(TexturedCanvasItemsProvider::with_sizing(
            ItemSizing::FixedWidth {
                width: px(500.0),
                estimated_height: px(800.0),
            },
        )));

        Self {
            provider,
            diffs: Vec::new(),
            commit_info: None,
            needs_sync: false,
        }
    }

    /// Set the diffs to display on the canvas.
    /// This stores the diffs and marks items for sync during next render.
    pub fn set_diffs(
        &mut self,
        diffs: Vec<FileDiff>,
        commit_info: Option<(String, String)>,
        _cx: &mut Context<Self>,
    ) {
        self.diffs = diffs;
        self.commit_info = commit_info;
        self.needs_sync = true;
    }

    /// Sync the provider items with the current diffs.
    /// This is called during render when we have window access.
    fn sync_items_if_needed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.needs_sync {
            return;
        }
        self.needs_sync = false;

        // Clear existing items
        self.provider.borrow_mut().clear();

        // Layout diffs in a grid pattern
        let card_width = 500.0;
        let card_spacing = 30.0;
        let cards_per_row = 3;

        for (i, diff) in self.diffs.iter().enumerate() {
            let row = i / cards_per_row;
            let col = i % cards_per_row;

            let x = col as f32 * (card_width + card_spacing);
            // Estimate height based on diff size
            let estimated_height = Self::estimate_diff_height(diff);
            let y = if row == 0 {
                0.0
            } else {
                // For now, use a fixed row height - in a real implementation
                // we'd track actual heights
                row as f32 * (estimated_height + card_spacing)
            };

            let diff_clone = diff.clone();
            self.provider.borrow_mut().add_item(
                format!("diff-{}", i),
                point(px(x), px(y)),
                window,
                cx,
                move || render_diff_card(&diff_clone),
            );
        }
    }

    /// Estimate the height of a diff card based on content
    fn estimate_diff_height(diff: &FileDiff) -> f32 {
        let line_count = diff
            .old_content
            .lines()
            .count()
            .max(diff.new_content.lines().count());
        // Header (40) + padding (16) + lines (18 each)
        40.0 + 16.0 + (line_count as f32 * 18.0)
    }

    /// Check if the canvas has any content
    pub fn has_content(&self) -> bool {
        !self.diffs.is_empty()
    }
}

/// Render a single diff as a card element using diff-ui's DiffTextView
fn render_diff_card(diff: &FileDiff) -> AnyElement {
    let path = diff.path.clone();

    // Create the diff view using diff-ui
    let diff_view = DiffTextView::new(&diff.old_content, &diff.new_content)
        .with_theme(DiffTheme::dark())
        .with_render_mode(DiffRenderMode::FullBuffer);

    // Build the card with header and diff content
    div()
        .flex()
        .flex_col()
        .bg(rgb(0x1e1e1e))
        .rounded_lg()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(0x3c3c3c))
        // File header
        .child(
            div()
                .w_full()
                .px_3()
                .py_2()
                .bg(rgb(0x2d2d2d))
                .border_b_1()
                .border_color(rgb(0x3c3c3c))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(div().text_sm().text_color(rgb(0x8b949e)).child("📄"))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xe6edf3))
                                .child(path),
                        ),
                ),
        )
        // Diff content using DiffTextView
        .child(div().w_full().flex_1().child(diff_view.render_as_element()))
        .into_any_element()
}

impl Render for DiffCanvasView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // If no content, show placeholder
        if !self.has_content() {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(cx.theme().background)
                .text_color(cx.theme().muted_foreground)
                .child(
                    v_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Icon::new(IconName::File)
                                .size(px(48.))
                                .text_color(cx.theme().muted_foreground),
                        )
                        .child("Select a commit to view diffs")
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Click on a commit in the history panel"),
                        ),
                )
                .into_any_element();
        }

        // Sync items if diffs have changed (now we have window access)
        self.sync_items_if_needed(window, cx);

        let commit_info = self.commit_info.clone();

        div()
            .size_full()
            .relative()
            .bg(cx.theme().background)
            .overflow_hidden()
            // Canvas - using InfiniteCanvas like the textured example
            .child(
                InfiniteCanvas::new("diff-canvas", self.provider.clone()).options(
                    CanvasOptions::new()
                        .min_zoom(0.1)
                        .max_zoom(3.0)
                        .zoom_speed(2.0)
                        .show_grid(true),
                ),
            )
            // Controls overlay - commit info
            .child(div().absolute().top_3().left_3().flex().gap_2().when_some(
                commit_info,
                |el: Div, info| {
                    el.child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(cx.theme().muted.opacity(0.9))
                            .rounded_md()
                            .text_sm()
                            .child(format!("{}: {}", info.0, info.1)),
                    )
                },
            ))
            // Help text
            .child(
                div()
                    .absolute()
                    .bottom_3()
                    .left_3()
                    .px_3()
                    .py_1()
                    .bg(cx.theme().muted.opacity(0.7))
                    .rounded_md()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Middle-click to pan • Scroll to zoom"),
            )
            .into_any_element()
    }
}
