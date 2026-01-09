//! Diff Canvas View - displays file diffs on an infinite canvas
//!
//! This module provides a canvas view for displaying file diffs with
//! pan/zoom functionality and textured rendering.
//!
//! Controls:
//! - Middle mouse button: Pan the canvas
//! - Scroll wheel: Zoom in/out (centered on cursor)

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme, Icon, IconName};
use infinite_canvas::prelude::*;

use buffer_diff::{BufferDiff, DiffLineType};

/// Diff data for a single file in a commit
#[derive(Clone)]
pub struct FileDiff {
    pub path: String,
    pub old_content: String,
    pub new_content: String,
    pub buffer_diff: BufferDiff,
}

/// A view that displays file diffs on an infinite canvas
pub struct DiffCanvasView {
    provider: TexturedCanvasItemsProvider,
    camera: Camera,
    is_panning: bool,
    last_mouse_pos: Point<Pixels>,
    /// The diffs currently displayed
    diffs: Vec<FileDiff>,
    /// Commit info for display
    commit_info: Option<(String, String)>, // (short_hash, message)
    /// Canvas options
    options: CanvasOptions,
}

impl DiffCanvasView {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let provider = TexturedCanvasItemsProvider::with_sizing(ItemSizing::FixedWidth {
            width: px(500.0),
            estimated_height: px(800.0),
        });

        // Configure canvas options
        let options = CanvasOptions::new()
            .min_zoom(0.1)
            .max_zoom(3.0)
            .zoom_speed(1.0)
            .show_grid(false);

        Self {
            provider,
            camera: Camera::with_offset_and_zoom(point(px(50.0), px(50.0)), 1.0),
            is_panning: false,
            last_mouse_pos: point(px(0.0), px(0.0)),
            diffs: Vec::new(),
            commit_info: None,
            options,
        }
    }

    /// Set the diffs to display on the canvas
    pub fn set_diffs(
        &mut self,
        diffs: Vec<FileDiff>,
        commit_info: Option<(String, String)>,
        _cx: &mut Context<Self>,
    ) {
        self.diffs = diffs.clone();
        self.commit_info = commit_info;

        // Clear existing items
        let item_ids: Vec<String> = self.provider.item_ids().map(|s| s.to_string()).collect();
        for id in item_ids {
            self.provider.remove_item(&id);
        }

        // Layout diffs in a grid pattern
        let card_width = 500.0;
        let card_spacing = 30.0;
        let cards_per_row = 3;

        for (i, diff) in diffs.into_iter().enumerate() {
            let row = i / cards_per_row;
            let col = i % cards_per_row;

            let x = col as f32 * (card_width + card_spacing);
            // Estimate height based on diff size
            let estimated_height = Self::estimate_diff_height(&diff);
            let y = if row == 0 {
                0.0
            } else {
                // For now, use a fixed row height - in a real implementation
                // we'd track actual heights
                row as f32 * (estimated_height + card_spacing)
            };

            let diff_clone = diff.clone();
            self.provider.add_item_at(
                format!("diff-{}", i),
                point(px(x), px(y)),
                move || Self::render_diff_card(&diff_clone),
            );
        }

        // Reset camera to show content
        self.camera = Camera::with_offset_and_zoom(point(px(50.0), px(50.0)), 1.0);
    }

    /// Estimate the height of a diff card based on content
    fn estimate_diff_height(diff: &FileDiff) -> f32 {
        let line_count = diff
            .buffer_diff
            .hunks()
            .iter()
            .fold(0, |acc, hunk| acc + hunk.line_types.len());
        // Header (40) + padding (16) + lines (18 each)
        40.0 + 16.0 + (line_count as f32 * 18.0)
    }

    /// Render a single diff as a card element
    fn render_diff_card(diff: &FileDiff) -> AnyElement {
        let path = diff.path.clone();
        let old_lines: Vec<&str> = diff.old_content.lines().collect();
        let new_lines: Vec<&str> = diff.new_content.lines().collect();
        let hunks = diff.buffer_diff.hunks();

        // Collect all diff lines
        let mut diff_lines: Vec<(Option<usize>, Option<usize>, String, DiffLineKind)> = Vec::new();

        for (hunk_idx, hunk) in hunks.iter().enumerate() {
            let mut old_offset = 0;
            let mut new_offset = 0;

            for &line_type in hunk.line_types.iter() {
                match line_type {
                    DiffLineType::OldOnly => {
                        let old_line_idx = hunk.old_range.start + old_offset;
                        if let Some(line_content) = old_lines.get(old_line_idx) {
                            diff_lines.push((
                                Some(old_line_idx + 1),
                                None,
                                line_content.to_string(),
                                DiffLineKind::Removed,
                            ));
                        }
                        old_offset += 1;
                    }
                    DiffLineType::NewOnly => {
                        let new_line_idx = hunk.new_range.start + new_offset;
                        if let Some(line_content) = new_lines.get(new_line_idx) {
                            diff_lines.push((
                                None,
                                Some(new_line_idx + 1),
                                line_content.to_string(),
                                DiffLineKind::Added,
                            ));
                        }
                        new_offset += 1;
                    }
                    DiffLineType::Both => {
                        let old_line_idx = hunk.old_range.start + old_offset;
                        let new_line_idx = hunk.new_range.start + new_offset;
                        if let Some(line_content) = old_lines.get(old_line_idx) {
                            diff_lines.push((
                                Some(old_line_idx + 1),
                                Some(new_line_idx + 1),
                                line_content.to_string(),
                                DiffLineKind::Context,
                            ));
                        }
                        old_offset += 1;
                        new_offset += 1;
                    }
                }
            }
        }

        // Build the card
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
            // Diff content
            .child(
                div()
                    .w_full()
                    .child(v_flex().w_full().children(
                        diff_lines.into_iter().map(
                            |(old_num, new_num, content, kind)| {
                                Self::render_diff_line_element(old_num, new_num, &content, kind)
                            },
                        ),
                    )),
            )
            .into_any_element()
    }

    /// Render a single diff line
    fn render_diff_line_element(
        old_line_num: Option<usize>,
        new_line_num: Option<usize>,
        content: &str,
        kind: DiffLineKind,
    ) -> AnyElement {
        let (bg_color, sign, text_color) = match kind {
            DiffLineKind::Added => (rgb(0x1a3d2e), "+", rgb(0x3fb950)),
            DiffLineKind::Removed => (rgb(0x3d1a1a), "-", rgb(0xf85149)),
            DiffLineKind::Context => (rgb(0x1e1e1e), " ", rgb(0xcccccc)),
        };

        h_flex()
            .w_full()
            .bg(bg_color)
            .px_2()
            .py_0p5()
            .child(
                div()
                    .w(px(35.))
                    .text_xs()
                    .text_color(rgb(0x6e7681))
                    .child(format!(
                        "{:>4}",
                        old_line_num
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| " ".to_string())
                    )),
            )
            .child(
                div()
                    .w(px(35.))
                    .text_xs()
                    .text_color(rgb(0x6e7681))
                    .child(format!(
                        "{:>4}",
                        new_line_num
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| " ".to_string())
                    )),
            )
            .child(
                div()
                    .w(px(15.))
                    .text_xs()
                    .text_color(text_color)
                    .child(sign.to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .text_xs()
                    .font_family("monospace")
                    .text_color(text_color)
                    .child(content.to_string()),
            )
            .into_any_element()
    }

    /// Check if the canvas has any content
    pub fn has_content(&self) -> bool {
        !self.diffs.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
enum DiffLineKind {
    Added,
    Removed,
    Context,
}

impl Render for DiffCanvasView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // If no content, show placeholder (skip all provider operations)
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

        // Process render queue (only when we have content)
        if self.provider.tick() {
            cx.notify();
        }

        // Keep polling if there are active/pending renders
        if self.provider.active_count() > 0 || self.provider.pending_count() > 0 {
            cx.notify();
        }

        let viewport = window.viewport_size();
        let items = self.provider.items();
        let camera = self.camera.clone();

        // Collect rendered items
        let mut rendered_items: Vec<AnyElement> = Vec::new();

        for item in &items {
            // Request texture if not already requested
            if let TextureState::NotRequested = self.provider.texture_state(&item.id) {
                self.provider.request_texture(&item.id);
                cx.notify();
            }

            // Transform bounds by camera
            let screen_bounds = camera.canvas_to_screen_bounds(item.bounds);

            // Simple culling - skip items outside viewport
            let viewport_bounds = Bounds::new(point(px(0.0), px(0.0)), viewport);
            if !bounds_intersect(&screen_bounds, &viewport_bounds) {
                continue;
            }

            // Create element based on texture state
            let content: AnyElement = match self.provider.texture_state(&item.id) {
                TextureState::Ready { ref image, .. } => {
                    img(image.clone())
                        .size_full()
                        .object_fit(ObjectFit::Fill)
                        .into_any_element()
                }
                TextureState::Rendering => div()
                    .size_full()
                    .bg(rgb(0x2d2d2d))
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x8b949e))
                            .child("Rendering..."),
                    )
                    .into_any_element(),
                TextureState::NotRequested => div()
                    .size_full()
                    .bg(rgb(0x2d2d2d))
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6e7681))
                            .child("Loading..."),
                    )
                    .into_any_element(),
                TextureState::Failed(ref msg) => div()
                    .size_full()
                    .bg(rgb(0x3d1a1a))
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xf85149))
                            .child(format!("Error: {}", msg)),
                    )
                    .into_any_element(),
            };

            // Position the item
            rendered_items.push(
                div()
                    .absolute()
                    .left(screen_bounds.origin.x)
                    .top(screen_bounds.origin.y)
                    .w(screen_bounds.size.width)
                    .h(screen_bounds.size.height)
                    .child(content)
                    .into_any_element(),
            );
        }

        // Build the canvas with controls
        let commit_info = self.commit_info.clone();
        let options = self.options.clone();

        div()
            .size_full()
            .relative()
            .bg(cx.theme().background)
            .overflow_hidden()
            // Canvas area
            .child(
                div()
                    .id("diff-canvas")
                    .size_full()
                    .relative()
                    .children(rendered_items)
                    // Middle mouse button pan - start
                    .on_mouse_down(
                        MouseButton::Middle,
                        cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                            this.is_panning = true;
                            this.last_mouse_pos = event.position;
                        }),
                    )
                    // Middle mouse button pan - end
                    .on_mouse_up(
                        MouseButton::Middle,
                        cx.listener(|this, _: &MouseUpEvent, _window, _cx| {
                            this.is_panning = false;
                        }),
                    )
                    // Mouse move for panning
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        if this.is_panning && event.pressed_button == Some(MouseButton::Middle) {
                            let delta = point(
                                event.position.x - this.last_mouse_pos.x,
                                event.position.y - this.last_mouse_pos.y,
                            );
                            this.camera.pan(delta);
                            this.last_mouse_pos = event.position;
                            cx.notify();
                        }
                    }))
                    // Scroll wheel zoom (centered on cursor)
                    .on_scroll_wheel(cx.listener(move |this, event: &ScrollWheelEvent, _window, cx| {
                        let delta = match event.delta {
                            ScrollDelta::Lines(lines) => lines.y * 0.1,
                            ScrollDelta::Pixels(pixels) => f32::from(pixels.y) * 0.001,
                        };

                        // Calculate zoom factor (positive delta = zoom in, negative = zoom out)
                        // Note: delta sign is reversed from default to match natural scroll expectation
                        let zoom_factor = 1.0 + delta * options.zoom_speed;

                        // Use Camera::zoom_around to zoom centered on cursor position
                        this.camera.zoom_around(
                            zoom_factor,
                            event.position,
                            options.min_zoom,
                            options.max_zoom,
                        );

                        cx.notify();
                    })),
            )
            // Controls overlay - commit info
            .child(
                div()
                    .absolute()
                    .top_3()
                    .left_3()
                    .flex()
                    .gap_2()
                    .when_some(commit_info, |el: Div, info| {
                        el.child(
                            div()
                                .px_3()
                                .py_1()
                                .bg(cx.theme().muted.opacity(0.9))
                                .rounded_md()
                                .text_sm()
                                .child(format!("{}: {}", info.0, info.1)),
                        )
                    }),
            )
            // Zoom indicator
            .child(
                div()
                    .absolute()
                    .bottom_3()
                    .right_3()
                    .px_3()
                    .py_1()
                    .bg(cx.theme().muted.opacity(0.9))
                    .rounded_md()
                    .text_sm()
                    .child(format!("{:.0}%", self.camera.zoom * 100.0)),
            )
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

/// Check if two bounds intersect
fn bounds_intersect(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> bool {
    a.origin.x < b.origin.x + b.size.width
        && a.origin.x + a.size.width > b.origin.x
        && a.origin.y < b.origin.y + b.size.height
        && a.origin.y + a.size.height > b.origin.y
}
