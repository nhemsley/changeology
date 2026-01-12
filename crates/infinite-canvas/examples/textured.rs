//! Example demonstrating InfiniteCanvas with TexturedCanvasItemsProvider
//!
//! This example shows how to use `InfiniteCanvas` with `TexturedCanvasItemsProvider`
//! which renders items as textures for smooth zooming.
//!
//! Features:
//! - Pan with middle mouse button
//! - Zoom with scroll wheel (centered on cursor)
//! - Items rendered as textures (zoomable)
//!
//! Run with: RUST_LOG=info cargo run -p infinite-canvas --example textured

use gpui::*;
use infinite_canvas::prelude::*;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

/// Background color for the canvas
const BG_COLOR: u32 = 0x1e1e1e;

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
    provider: Rc<RefCell<TexturedCanvasItemsProvider>>,
}

impl TexturedCanvasView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        info!("[textured_example] Creating TexturedCanvasView");

        // Create provider with FixedWidth sizing (height measured from content)
        let provider = Rc::new(RefCell::new(TexturedCanvasItemsProvider::with_sizing(
            ItemSizing::FixedWidth {
                width: px(280.0),
                estimated_height: px(150.0),
            },
        )));

        // Add code file cards - each gets its own TexturedView for async rendering

        // Row 1
        provider
            .borrow_mut()
            .add_item("main.rs", point(px(50.0), px(50.0)), window, cx, || {
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

        provider
            .borrow_mut()
            .add_item("lib.rs", point(px(370.0), px(50.0)), window, cx, || {
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
        provider
            .borrow_mut()
            .add_item("canvas.rs", point(px(50.0), px(250.0)), window, cx, || {
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

        provider.borrow_mut().add_item(
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
        provider
            .borrow_mut()
            .add_item("item.rs", point(px(50.0), px(450.0)), window, cx, || {
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

        provider
            .borrow_mut()
            .add_item("tests.rs", point(px(370.0), px(450.0)), window, cx, || {
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
            provider.borrow().item_count()
        );

        Self { provider }
    }
}

impl Render for TexturedCanvasView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let item_count = self.provider.borrow().item_count();

        div()
            .size_full()
            .bg(rgb(BG_COLOR))
            .child(
                InfiniteCanvas::new("textured-canvas", self.provider.clone()).options(
                    CanvasOptions::new()
                        .show_grid(true)
                        .min_zoom(0.1)
                        .max_zoom(10.0),
                ),
            )
            // Status bar overlay
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
                    .child(format!("Items: {}", item_count)),
            )
            // Help text overlay
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
    }
}
