//! Example demonstrating textured canvas items with async rendering
//!
//! This example shows how to use `TexturedCanvasItemsProvider` to render
//! canvas items to textures for efficient pan/zoom.
//!
//! Features demonstrated:
//! - Async background rendering (non-blocking)
//! - ItemSizing support (FixedWidth with measured height)
//! - Multiple concurrent texture renders
//! - Downscale modes for preserving syntax highlighting when zoomed out
//!
//! Run with: cargo run -p infinite-canvas --example textured

use gpui::*;
use infinite_canvas::prelude::*;
use std::sync::Arc;

/// Background color for FurthestFromBackground mode (dark editor bg)
const BG_COLOR: [u8; 4] = [0x1e, 0x1e, 0x1e, 0xff];

/// Helper to create a code file card with syntax-highlighted content
fn code_file_card(filename: &str, code_parts: Vec<(&str, u32)>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .bg(rgb(0x1e1e1e))
        .rounded_lg()
        .overflow_hidden()
        // Title bar
        .child(
            div().px_3().py_2().bg(rgb(0x3c3c3c)).child(
                div()
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .child(filename.to_string()),
            ),
        )
        // Code content
        .child(
            div()
                .p_3()
                .flex()
                .flex_wrap()
                .children(code_parts.into_iter().map(|(text, color)| {
                    div()
                        .text_sm()
                        .text_color(rgb(color))
                        .child(text.to_string())
                })),
        )
        .into_any_element()
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Create the provider with FixedWidth sizing
        // This allows items to have different heights based on content
        let mut provider = TexturedCanvasItemsProvider::with_sizing(ItemSizing::FixedWidth {
            width: px(280.0),
            estimated_height: px(150.0),
        });

        // Allow up to 4 concurrent renders for faster loading
        provider.set_max_concurrent_renders(4);

        // Add items that look like code files
        // Colors: keywords=red, functions/types=orange, strings=green, punctuation=light grey
        provider.add_item("main.rs", || {
            code_file_card(
                "main.rs",
                vec![
                    ("fn ", 0xff5555),
                    ("main", 0xffb86c),
                    ("() {", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("println!", 0xff5555),
                    ("(", 0xcccccc),
                    ("\"Hello, world!\"", 0x50fa7b),
                    (")", 0xcccccc),
                    (";", 0xcccccc),
                    ("\n}", 0xcccccc),
                ],
            )
        });

        provider.add_item_at("lib.rs", point(px(320.0), px(0.0)), || {
            code_file_card(
                "lib.rs",
                vec![
                    ("pub mod ", 0xff5555),
                    ("canvas", 0xcccccc),
                    (";", 0xcccccc),
                    ("\n", 0xcccccc),
                    ("pub mod ", 0xff5555),
                    ("renderer", 0xcccccc),
                    (";", 0xcccccc),
                    ("\n\n", 0xcccccc),
                    ("pub use ", 0xff5555),
                    ("canvas", 0xcccccc),
                    ("::", 0xcccccc),
                    ("*", 0xffb86c),
                    (";", 0xcccccc),
                ],
            )
        });

        provider.add_item_at("canvas.rs", point(px(0.0), px(180.0)), || {
            code_file_card(
                "canvas.rs",
                vec![
                    ("pub struct ", 0xff5555),
                    ("Canvas", 0xffb86c),
                    (" {", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("width", 0xcccccc),
                    (": ", 0xcccccc),
                    ("u32", 0xff5555),
                    (",", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("height", 0xcccccc),
                    (": ", 0xcccccc),
                    ("u32", 0xff5555),
                    (",", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("items", 0xcccccc),
                    (": ", 0xcccccc),
                    ("Vec", 0xff5555),
                    ("<", 0xcccccc),
                    ("Item", 0xffb86c),
                    (">", 0xcccccc),
                    (",", 0xcccccc),
                    ("\n}", 0xcccccc),
                ],
            )
        });

        provider.add_item_at("renderer.rs", point(px(320.0), px(180.0)), || {
            code_file_card(
                "renderer.rs",
                vec![
                    ("impl ", 0xff5555),
                    ("Render ", 0xffb86c),
                    ("for ", 0xff5555),
                    ("Canvas", 0xffb86c),
                    (" {", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("fn ", 0xff5555),
                    ("render", 0xffb86c),
                    ("(&", 0xcccccc),
                    ("self", 0xff5555),
                    (") {", 0xcccccc),
                    ("\n        ", 0xcccccc),
                    ("// Draw items", 0x6272a4),
                    ("\n        ", 0xcccccc),
                    ("for ", 0xff5555),
                    ("item ", 0xcccccc),
                    ("in ", 0xff5555),
                    ("&", 0xcccccc),
                    ("self", 0xff5555),
                    (".items {", 0xcccccc),
                    ("\n            ", 0xcccccc),
                    ("item", 0xcccccc),
                    (".", 0xcccccc),
                    ("draw", 0xffb86c),
                    ("();", 0xcccccc),
                    ("\n        }", 0xcccccc),
                    ("\n    }", 0xcccccc),
                    ("\n}", 0xcccccc),
                ],
            )
        });

        provider.add_item_at("item.rs", point(px(0.0), px(400.0)), || {
            code_file_card(
                "item.rs",
                vec![
                    ("#[derive(", 0xffb86c),
                    ("Clone", 0xcccccc),
                    (", ", 0xcccccc),
                    ("Debug", 0xcccccc),
                    (")]", 0xffb86c),
                    ("\n", 0xcccccc),
                    ("pub struct ", 0xff5555),
                    ("Item", 0xffb86c),
                    (" {", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("pub ", 0xff5555),
                    ("id", 0xcccccc),
                    (": ", 0xcccccc),
                    ("String", 0xff5555),
                    (",", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("pub ", 0xff5555),
                    ("bounds", 0xcccccc),
                    (": ", 0xcccccc),
                    ("Bounds", 0xffb86c),
                    (",", 0xcccccc),
                    ("\n}", 0xcccccc),
                ],
            )
        });

        provider.add_item_at("tests.rs", point(px(320.0), px(400.0)), || {
            code_file_card(
                "tests.rs",
                vec![
                    ("#[test]", 0xffb86c),
                    ("\n", 0xcccccc),
                    ("fn ", 0xff5555),
                    ("test_canvas", 0xffb86c),
                    ("() {", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("let ", 0xff5555),
                    ("canvas = ", 0xcccccc),
                    ("Canvas", 0xffb86c),
                    ("::", 0xcccccc),
                    ("new", 0xffb86c),
                    ("();", 0xcccccc),
                    ("\n    ", 0xcccccc),
                    ("assert!", 0xff5555),
                    ("(canvas.", 0xcccccc),
                    ("is_empty", 0xffb86c),
                    ("());", 0xcccccc),
                    ("\n}", 0xcccccc),
                ],
            )
        });

        // Create canvas view with the provider
        let canvas_view = TexturedCanvasView::new(provider);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.0), px(800.0)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Textured Canvas Example (Async Rendering)".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| canvas_view),
        )
        .unwrap();
    });
}

/// Blockiness presets for downscaling
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Blockiness {
    /// Subtle - only downscale at very low zoom
    Subtle,
    /// Normal - balanced blockiness
    #[default]
    Normal,
    /// Chunky - more aggressive blocks
    Chunky,
    /// Extreme - very large blocks
    Extreme,
}

impl Blockiness {
    fn all() -> &'static [Blockiness] {
        &[
            Blockiness::Subtle,
            Blockiness::Normal,
            Blockiness::Chunky,
            Blockiness::Extreme,
        ]
    }

    fn display_name(&self) -> &'static str {
        match self {
            Blockiness::Subtle => "Subtle",
            Blockiness::Normal => "Normal",
            Blockiness::Chunky => "Chunky",
            Blockiness::Extreme => "Extreme",
        }
    }

    /// Get scale factor based on zoom level
    fn scale_factor(&self, zoom: f32) -> u32 {
        match self {
            Blockiness::Subtle => {
                // Only downscale when very zoomed out
                if zoom < 0.25 { 2 } else { 1 }
            }
            Blockiness::Normal => {
                // Balanced downscaling
                if zoom < 0.25 {
                    4
                } else if zoom < 0.5 {
                    2
                } else {
                    1
                }
            }
            Blockiness::Chunky => {
                // More aggressive blocks
                if zoom < 0.25 {
                    8
                } else if zoom < 0.5 {
                    4
                } else if zoom < 0.75 {
                    2
                } else {
                    1
                }
            }
            Blockiness::Extreme => {
                // Maximum blockiness
                if zoom < 0.25 {
                    16
                } else if zoom < 0.5 {
                    8
                } else if zoom < 0.75 {
                    4
                } else {
                    2
                }
            }
        }
    }
}

/// A view that displays a textured canvas
struct TexturedCanvasView {
    provider: TexturedCanvasItemsProvider,
    camera: Camera,
    is_panning: bool,
    last_mouse_pos: Point<Pixels>,
    downscale_mode: DownscaleMode,
    show_dropdown: bool,
    blockiness: Blockiness,
    show_blockiness_dropdown: bool,
}

impl TexturedCanvasView {
    fn new(provider: TexturedCanvasItemsProvider) -> Self {
        Self {
            provider,
            camera: Camera::default(),
            is_panning: false,
            last_mouse_pos: point(px(0.0), px(0.0)),
            downscale_mode: DownscaleMode::MostSaturated,
            show_dropdown: false,
            blockiness: Blockiness::Normal,
            show_blockiness_dropdown: false,
        }
    }
}

impl Render for TexturedCanvasView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process render queue (polls for completed async renders)
        if self.provider.tick() {
            // More work pending or completed, request another frame
            cx.notify();
        }

        // If there are active renders or pending items, keep polling
        if self.provider.active_count() > 0 || self.provider.pending_count() > 0 {
            cx.notify();
        }

        let viewport = window.viewport_size();
        let items = self.provider.items();
        let camera = self.camera.clone();

        // Request textures for all items and collect rendered items
        let mut rendered_items: Vec<AnyElement> = Vec::new();

        for item in &items {
            // Request texture if not already requested
            match self.provider.texture_state(&item.id) {
                TextureState::NotRequested => {
                    self.provider.request_texture(&item.id);
                    cx.notify(); // Trigger re-render to process queue
                }
                _ => {}
            }

            // Transform bounds by camera
            let screen_bounds = transform_bounds(&item.bounds, &camera);

            // Simple culling
            let viewport_bounds = Bounds::new(point(px(0.0), px(0.0)), viewport);
            if !bounds_intersect(&screen_bounds, &viewport_bounds) {
                continue;
            }

            // Calculate downscale factor based on zoom and blockiness setting
            let scale_factor = self.blockiness.scale_factor(self.camera.zoom);

            // Create element based on texture state
            let content: AnyElement = match self.provider.texture_state(&item.id) {
                TextureState::Ready { ref image, .. } => {
                    // Apply downscaling if zoomed out and not using Linear mode
                    if scale_factor > 1 && self.downscale_mode != DownscaleMode::Linear {
                        // Get pixel data from the image
                        if let Some(pixels) = image.as_bytes(0) {
                            let img_size = image.size(0);
                            let width = img_size.width.0 as u32;
                            let height = img_size.height.0 as u32;

                            // Apply the selected downscale algorithm
                            let (scaled_pixels, new_w, new_h) = downscale_pixels(
                                pixels,
                                width,
                                height,
                                scale_factor.min(4), // Cap at 4x downscale
                                self.downscale_mode,
                                BG_COLOR,
                            );

                            // Create new RenderImage from downscaled pixels
                            if let Some(rgba_img) =
                                image::RgbaImage::from_raw(new_w, new_h, scaled_pixels)
                            {
                                let frame = image::Frame::new(rgba_img);
                                let render_img = RenderImage::new(smallvec::smallvec![frame]);
                                img(Arc::new(render_img))
                                    .size_full()
                                    .object_fit(ObjectFit::Fill)
                                    .into_any_element()
                            } else {
                                // Fallback to original
                                img(image.clone())
                                    .size_full()
                                    .object_fit(ObjectFit::Fill)
                                    .into_any_element()
                            }
                        } else {
                            // No pixel data, use original
                            img(image.clone())
                                .size_full()
                                .object_fit(ObjectFit::Fill)
                                .into_any_element()
                        }
                    } else {
                        // No downscaling needed
                        img(image.clone())
                            .size_full()
                            .object_fit(ObjectFit::Fill)
                            .into_any_element()
                    }
                }
                TextureState::Rendering => div()
                    .size_full()
                    .bg(rgb(0xd5d5d5))
                    .flex()
                    .justify_center()
                    .items_center()
                    .text_color(rgb(0x666666))
                    .child("Rendering...")
                    .into_any_element(),
                TextureState::NotRequested => div()
                    .size_full()
                    .bg(rgb(0xe0e0e0))
                    .flex()
                    .justify_center()
                    .items_center()
                    .text_color(rgb(0x888888))
                    .child("Queued")
                    .into_any_element(),
                TextureState::Failed(ref msg) => div()
                    .size_full()
                    .bg(rgb(0xffcccc))
                    .flex()
                    .justify_center()
                    .items_center()
                    .text_color(rgb(0xcc0000))
                    .child(format!("Error: {}", msg))
                    .into_any_element(),
            };

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

        // Status text
        let pending = self.provider.pending_count();
        let active = self.provider.active_count();
        let total = items.len();
        let ready = items
            .iter()
            .filter(|i| {
                matches!(
                    self.provider.texture_state(&i.id),
                    TextureState::Ready { .. }
                )
            })
            .count();

        let current_mode = self.downscale_mode;
        let show_dropdown = self.show_dropdown;
        let current_blockiness = self.blockiness;
        let show_blockiness_dropdown = self.show_blockiness_dropdown;

        // Calculate current scale factor for display
        let scale_factor = self.blockiness.scale_factor(self.camera.zoom);

        div()
            .size_full()
            .bg(rgb(0xf5f5f5))
            .overflow_hidden()
            // Mouse handlers for pan
            .on_mouse_down(MouseButton::Middle, {
                cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                    this.is_panning = true;
                    this.last_mouse_pos = event.position;
                })
            })
            .on_mouse_up(MouseButton::Middle, {
                cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                    this.is_panning = false;
                })
            })
            .on_mouse_move({
                cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                    if this.is_panning {
                        let delta = point(
                            event.position.x - this.last_mouse_pos.x,
                            event.position.y - this.last_mouse_pos.y,
                        );
                        this.camera.offset.x += delta.x / this.camera.zoom;
                        this.camera.offset.y += delta.y / this.camera.zoom;
                        this.last_mouse_pos = event.position;
                        cx.notify();
                    }
                })
            })
            // Scroll for zoom (centered on mouse position)
            .on_scroll_wheel({
                cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                    let delta = event.delta.pixel_delta(px(1.0));
                    let delta_y: f32 = delta.y.into();
                    let zoom_factor = 1.0 + (delta_y / 100.0);
                    let new_zoom = (this.camera.zoom * zoom_factor).clamp(0.1, 10.0);

                    // Get mouse position in screen space
                    let mouse_pos = event.position;

                    // Convert mouse position to canvas space before zoom
                    let canvas_x = (mouse_pos.x - this.camera.offset.x) / this.camera.zoom;
                    let canvas_y = (mouse_pos.y - this.camera.offset.y) / this.camera.zoom;

                    // Apply new zoom
                    this.camera.zoom = new_zoom;

                    // Adjust offset so the canvas point under the mouse stays fixed
                    this.camera.offset.x = mouse_pos.x - canvas_x * new_zoom;
                    this.camera.offset.y = mouse_pos.y - canvas_y * new_zoom;

                    cx.notify();
                })
            })
            // Status bar
            .child(
                div()
                    .absolute()
                    .top_2()
                    .left_2()
                    .px_3()
                    .py_1()
                    .bg(rgb(0x333333))
                    .rounded_md()
                    .text_sm()
                    .text_color(white())
                    .child(format!(
                        "Items: {}/{} | Zoom: {:.0}% | Scale: {}x",
                        ready,
                        total,
                        self.camera.zoom * 100.0,
                        scale_factor
                    )),
            )
            // Downscale mode dropdown
            .child(
                div()
                    .absolute()
                    .top_2()
                    .right_2()
                    .flex()
                    .flex_col()
                    .gap_2()
                    // Mode dropdown
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0x444444))
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(white())
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_dropdown = !this.show_dropdown;
                                            this.show_blockiness_dropdown = false;
                                            cx.notify();
                                        }),
                                    )
                                    .child(format!("Mode: {} ▼", current_mode.display_name())),
                            )
                            // Mode dropdown options
                            .children(if show_dropdown {
                                Some(
                                    div()
                                        .bg(rgb(0x333333))
                                        .rounded_md()
                                        .overflow_hidden()
                                        .children(DownscaleMode::all().iter().map(|mode| {
                                            let mode = *mode;
                                            let is_selected = mode == current_mode;
                                            let text_color = if is_selected {
                                                rgb(0x50fa7b)
                                            } else {
                                                rgb(0xffffff)
                                            };
                                            let bg_color = if is_selected {
                                                rgb(0x444444)
                                            } else {
                                                rgb(0x333333)
                                            };
                                            div()
                                                .px_3()
                                                .py_1()
                                                .text_sm()
                                                .text_color(text_color)
                                                .bg(bg_color)
                                                .hover(|s| s.bg(rgb(0x555555)))
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.downscale_mode = mode;
                                                        this.show_dropdown = false;
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(mode.display_name())
                                        })),
                                )
                            } else {
                                None
                            }),
                    )
                    // Blockiness dropdown
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0x444444))
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(white())
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _, _, cx| {
                                            this.show_blockiness_dropdown =
                                                !this.show_blockiness_dropdown;
                                            this.show_dropdown = false;
                                            cx.notify();
                                        }),
                                    )
                                    .child(format!(
                                        "Blocks: {} ▼",
                                        current_blockiness.display_name()
                                    )),
                            )
                            // Blockiness dropdown options
                            .children(if show_blockiness_dropdown {
                                Some(
                                    div()
                                        .bg(rgb(0x333333))
                                        .rounded_md()
                                        .overflow_hidden()
                                        .children(Blockiness::all().iter().map(|b| {
                                            let b = *b;
                                            let is_selected = b == current_blockiness;
                                            let text_color = if is_selected {
                                                rgb(0x50fa7b)
                                            } else {
                                                rgb(0xffffff)
                                            };
                                            let bg_color = if is_selected {
                                                rgb(0x444444)
                                            } else {
                                                rgb(0x333333)
                                            };
                                            div()
                                                .px_3()
                                                .py_1()
                                                .text_sm()
                                                .text_color(text_color)
                                                .bg(bg_color)
                                                .hover(|s| s.bg(rgb(0x555555)))
                                                .cursor_pointer()
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        this.blockiness = b;
                                                        this.show_blockiness_dropdown = false;
                                                        cx.notify();
                                                    }),
                                                )
                                                .child(b.display_name())
                                        })),
                                )
                            } else {
                                None
                            }),
                    ),
            )
            // Help text
            .child(
                div()
                    .absolute()
                    .bottom_2()
                    .left_2()
                    .px_3()
                    .py_1()
                    .bg(rgb(0x333333))
                    .rounded_md()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(
                        "Pan: middle-drag | Zoom: scroll (try zooming out to see downscale modes)",
                    ),
            )
            // Canvas items
            .children(rendered_items)
    }
}

/// Camera for pan/zoom
#[derive(Clone, Default)]
struct Camera {
    offset: Point<Pixels>,
    zoom: f32,
}

impl Camera {
    fn default() -> Self {
        Self {
            offset: point(px(0.0), px(0.0)),
            zoom: 1.0,
        }
    }
}

/// Transform bounds from world to screen space
fn transform_bounds(bounds: &Bounds<Pixels>, camera: &Camera) -> Bounds<Pixels> {
    Bounds::new(
        point(
            (bounds.origin.x + camera.offset.x) * camera.zoom,
            (bounds.origin.y + camera.offset.y) * camera.zoom,
        ),
        size(
            bounds.size.width * camera.zoom,
            bounds.size.height * camera.zoom,
        ),
    )
}

/// Check if two bounds intersect
fn bounds_intersect(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> bool {
    a.origin.x < b.origin.x + b.size.width
        && a.origin.x + a.size.width > b.origin.x
        && a.origin.y < b.origin.y + b.size.height
        && a.origin.y + a.size.height > b.origin.y
}
