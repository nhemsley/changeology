//! Basic example of the infinite canvas component.
//!
//! This example demonstrates:
//! - Creating an infinite canvas
//! - Adding items with different positions and sizes
//! - Pan and zoom functionality
//!
//! Run with: cargo run --example basic

use gpui::*;
use gpui_component::Root;
use infinite_canvas::{Camera, CanvasItem, CanvasOptions, InfiniteCanvas};

fn main() {
    let app = Application::new();

    app.run(|cx| {
        // Initialize gpui-component
        gpui_component::init(cx);

        // Initialize infinite-canvas
        infinite_canvas::init(cx);

        // Create the main window
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1024.), px(768.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Infinite Canvas Example".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|_cx| ExampleView::new());
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}

struct ExampleView {
    camera: Camera,
    items: Vec<CanvasItem>,
}

impl ExampleView {
    fn new() -> Self {
        // Create some sample items
        let items = vec![
            CanvasItem::new(
                "item-1",
                Bounds::new(point(px(50.), px(50.)), size(px(150.), px(100.))),
            ),
            CanvasItem::new(
                "item-2",
                Bounds::new(point(px(250.), px(80.)), size(px(120.), px(80.))),
            ),
            CanvasItem::new(
                "item-3",
                Bounds::new(point(px(100.), px(200.)), size(px(200.), px(120.))),
            ),
            CanvasItem::new(
                "item-4",
                Bounds::new(point(px(350.), px(220.)), size(px(100.), px(100.))),
            ),
            CanvasItem::new(
                "item-5",
                Bounds::new(point(px(500.), px(100.)), size(px(180.), px(150.))),
            ),
        ];

        Self {
            camera: Camera::default(),
            items,
        }
    }
}

impl Render for ExampleView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let camera = self.camera;
        let items = self.items.clone();

        let offset_x: f32 = camera.offset.x.into();
        let offset_y: f32 = camera.offset.y.into();

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // Header
                div()
                    .w_full()
                    .h(px(40.))
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x252525))
                    .border_b_1()
                    .border_color(rgb(0x3d3d3d))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .child("Infinite Canvas Example"),
                    )
                    .child(div().text_xs().text_color(rgb(0x888888)).child(format!(
                        "Zoom: {:.0}% | Offset: ({:.0}, {:.0})",
                        camera.zoom * 100.0,
                        offset_x,
                        offset_y
                    ))),
            )
            .child(
                // Canvas area
                div().flex_1().w_full().child(
                    InfiniteCanvas::new("main-canvas")
                        .camera(camera)
                        .options(
                            CanvasOptions::new()
                                .show_grid(true)
                                .min_zoom(0.1)
                                .max_zoom(5.0),
                        )
                        .items(items),
                ),
            )
            .child(
                // Footer with instructions
                div()
                    .w_full()
                    .h(px(32.))
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_4()
                    .bg(rgb(0x252525))
                    .border_t_1()
                    .border_color(rgb(0x3d3d3d))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child("Scroll to zoom • Middle-click drag to pan"),
                    ),
            )
    }
}
