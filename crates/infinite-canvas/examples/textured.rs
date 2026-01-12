//! Example demonstrating TexturedCanvasProvider with async rendering
//!
//! This example shows how to use `TexturedCanvasProvider` which leverages
//! GPUI's `TexturedView` for proper async wake mechanism.
//!
//! Features:
//! - No timer-based polling workarounds
//! - No `tick()` calls needed
//! - UI updates automatically when textures are ready
//! - Proper zoom scaling with `object_fit`
//!
//! Run with: RUST_LOG=info cargo run -p infinite-canvas --example textured

use gpui::*;
use infinite_canvas::prelude::*;
use log::info;

/// Background color for the canvas
const BG_COLOR: u32 = 0xf5f5f5;

/// Helper to create a code file card with syntax-highlighted content
fn code_file_card(filename: &str, code_parts: Vec<(&'static str, u32)>) -> AnyElement {
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
    env_logger::init();
    info!("[textured_example] Starting textured canvas example");

    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Textured Canvas Example".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.0), px(800.0)),
                    cx,
                ))),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| TexturedCanvasView::new(window, cx)),
        )
        .unwrap();
    });
}

// ============================================================================
// Canvas View
// ============================================================================

struct TexturedCanvasView {
    provider: TexturedCanvasItemsProvider,
    camera: Camera,
    is_panning: bool,
    last_mouse_pos: Point<Pixels>,
}

impl TexturedCanvasView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        info!("[textured_example] Creating TexturedCanvasView");

        // Create provider with FixedWidth sizing (height measured from content)
        let mut provider = TexturedCanvasItemsProvider::with_sizing(ItemSizing::FixedWidth {
            width: px(280.0),
            estimated_height: px(150.0),
        });

        // Add code file cards - each gets its own TexturedView for async rendering
        // The TexturedView handles all the async wake mechanism internally

        // Row 1
        provider.add_item("main.rs", point(px(50.0), px(50.0)), window, cx, || {
            code_file_card(
                "main.rs",
                vec![
                    ("fn ", 0x569cd6),
                    ("main", 0xdcdcaa),
                    ("() { ", 0xd4d4d4),
                    ("println!", 0xdcdcaa),
                    ("(", 0xd4d4d4),
                    ("\"Hello, world!\"", 0xce9178),
                    (");", 0xd4d4d4),
                    ("\n}", 0xd4d4d4),
                ],
            )
        });

        provider.add_item("lib.rs", point(px(370.0), px(50.0)), window, cx, || {
            code_file_card(
                "lib.rs",
                vec![
                    ("pub mod ", 0x569cd6),
                    ("canvas", 0x4ec9b0),
                    (";", 0xd4d4d4),
                    ("pub mod ", 0x569cd6),
                    ("renderer", 0x4ec9b0),
                    (";", 0xd4d4d4),
                    ("\n\npub use ", 0x569cd6),
                    ("canvas", 0x4ec9b0),
                    ("::*;", 0xd4d4d4),
                ],
            )
        });

        // Row 2
        provider.add_item("canvas.rs", point(px(50.0), px(250.0)), window, cx, || {
            code_file_card(
                "canvas.rs",
                vec![
                    ("pub struct ", 0x569cd6),
                    ("Canvas", 0x4ec9b0),
                    (" { ", 0xd4d4d4),
                    ("width", 0x9cdcfe),
                    (": ", 0xd4d4d4),
                    ("u32", 0x4ec9b0),
                    (",\n", 0xd4d4d4),
                    ("height", 0x9cdcfe),
                    (": ", 0xd4d4d4),
                    ("u32", 0x4ec9b0),
                    (", ", 0xd4d4d4),
                    ("items", 0x9cdcfe),
                    (": ", 0xd4d4d4),
                    ("Vec", 0x4ec9b0),
                    ("<Item>,", 0xd4d4d4),
                    ("\n}", 0xd4d4d4),
                ],
            )
        });

        provider.add_item(
            "renderer.rs",
            point(px(370.0), px(250.0)),
            window,
            cx,
            || {
                code_file_card(
                    "renderer.rs",
                    vec![
                        ("impl ", 0x569cd6),
                        ("Render", 0x4ec9b0),
                        (" for ", 0x569cd6),
                        ("Canvas", 0x4ec9b0),
                        (" { ", 0xd4d4d4),
                        ("fn ", 0x569cd6),
                        ("render", 0xdcdcaa),
                        ("(&", 0xd4d4d4),
                        ("self", 0x569cd6),
                        (") { ", 0xd4d4d4),
                        ("// Draw items", 0x6a9955),
                        ("\nfor ", 0x569cd6),
                        ("item", 0x9cdcfe),
                        (" in ", 0x569cd6),
                        ("&self", 0x569cd6),
                        (".items { ", 0xd4d4d4),
                        ("item", 0x9cdcfe),
                        (".draw();", 0xd4d4d4),
                        ("\n} }}", 0xd4d4d4),
                    ],
                )
            },
        );

        // Row 3
        provider.add_item("item.rs", point(px(50.0), px(450.0)), window, cx, || {
            code_file_card(
                "item.rs",
                vec![
                    ("#[derive(Clone, Debug)]", 0xd4d4d4),
                    ("pub struct ", 0x569cd6),
                    ("Item", 0x4ec9b0),
                    ("\n{ ", 0xd4d4d4),
                    ("pub ", 0x569cd6),
                    ("id", 0x9cdcfe),
                    (": ", 0xd4d4d4),
                    ("String", 0x4ec9b0),
                    (", ", 0xd4d4d4),
                    ("pub ", 0x569cd6),
                    ("bounds", 0x9cdcfe),
                    (": ", 0xd4d4d4),
                    ("Bounds", 0x4ec9b0),
                    (",\n}", 0xd4d4d4),
                ],
            )
        });

        provider.add_item("tests.rs", point(px(370.0), px(450.0)), window, cx, || {
            code_file_card(
                "tests.rs",
                vec![
                    ("#[test]", 0xd4d4d4),
                    ("fn ", 0x569cd6),
                    ("test_canvas", 0xdcdcaa),
                    ("() { ", 0xd4d4d4),
                    ("let ", 0x569cd6),
                    ("canvas", 0x9cdcfe),
                    (" = ", 0xd4d4d4),
                    ("Canvas", 0x4ec9b0),
                    ("::new(); ", 0xd4d4d4),
                    ("assert!", 0xdcdcaa),
                    ("(canvas.is_empty\n());", 0xd4d4d4),
                    ("\n}", 0xd4d4d4),
                ],
            )
        });

        info!(
            "[textured_example] Added {} items to provider",
            provider.item_count()
        );

        Self {
            provider,
            camera: Camera::default(),
            is_panning: false,
            last_mouse_pos: point(px(0.0), px(0.0)),
        }
    }
}

impl Render for TexturedCanvasView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport_size = size(px(1200.0), px(800.0)); // Approximate
        let viewport_bounds = Bounds::new(point(px(0.0), px(0.0)), viewport_size);
        let items = self.provider.items();

        // Collect rendered items
        let mut rendered_items: Vec<AnyElement> = Vec::new();

        for item in &items {
            // Transform bounds by camera using library method
            let screen_bounds = self.camera.canvas_to_screen_bounds(item.bounds);

            // Simple culling - skip items outside viewport
            if !screen_bounds.intersects(&viewport_bounds) {
                continue;
            }

            // Render the item at its transformed position (with proper zoom scaling)
            if let Some(element) = self.provider.render_item_at(&item.id, screen_bounds, cx) {
                rendered_items.push(element);
            }
        }

        // Status text
        let total = items.len();

        div()
            .size_full()
            .bg(rgb(BG_COLOR))
            .overflow_hidden()
            // Mouse handlers for pan
            .on_mouse_down(MouseButton::Middle, {
                cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                    info!("[textured_example] PAN START");
                    this.is_panning = true;
                    this.last_mouse_pos = event.position;
                })
            })
            .on_mouse_up(MouseButton::Middle, {
                cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                    info!("[textured_example] PAN END");
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
                        this.camera.pan(delta);
                        this.last_mouse_pos = event.position;
                        cx.notify();
                    }
                })
            })
            // Scroll for zoom
            .on_scroll_wheel({
                cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                    let delta = event.delta.pixel_delta(px(1.0));
                    let zoom_factor = 1.0 + f32::from(delta.y) / 100.0;
                    this.camera
                        .zoom_around(zoom_factor, event.position, 0.1, 10.0);
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
                        "Items: {} | Zoom: {:.0}%",
                        total,
                        self.camera.zoom * 100.0
                    )),
            )
            // Help text
            .child(
                div()
                    .absolute()
                    .bottom_2()
                    .left_2()
                    .px_3()
                    .py_1()
                    .bg(rgb(0x555555))
                    .rounded_md()
                    .text_sm()
                    .text_color(rgb(0xaaaaaa))
                    .child("Pan: middle-drag | Zoom: scroll"),
            )
            // Render all items
            .children(rendered_items)
    }
}
