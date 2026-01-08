//! Example demonstrating textured canvas items
//!
//! This example shows how to use `TexturedCanvasItemsProvider` to render
//! canvas items to textures for efficient pan/zoom.
//!
//! Run with: cargo run -p infinite-canvas --example textured

use gpui::*;
use infinite_canvas::prelude::*;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Create the provider
        let mut provider = TexturedCanvasItemsProvider::new();

        // Add items with closures - they size themselves via layout!
        provider.add_item("hello", || {
            div()
                .p_4()
                .bg(rgb(0x3498db))
                .rounded_lg()
                .child(div().text_xl().text_color(white()).child("Hello World"))
                .into_any_element()
        });

        provider.add_item_at("card-1", point(px(250.0), px(50.0)), || {
            div()
                .p_4()
                .bg(rgb(0xe74c3c))
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_2xl().text_color(white()).child("Card Title"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffcccc))
                        .child("This is a description with some text"),
                )
                .into_any_element()
        });

        provider.add_item_at("card-2", point(px(50.0), px(200.0)), || {
            div()
                .p_4()
                .bg(rgb(0x2ecc71))
                .rounded_lg()
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_xl().text_color(white()).child("Green Card"))
                .child(div().flex().gap_1().children((1..=3).map(|i| {
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(0x27ae60))
                        .rounded_md()
                        .text_xs()
                        .text_color(white())
                        .child(format!("Tag {}", i))
                })))
                .into_any_element()
        });

        provider.add_item_at("card-3", point(px(300.0), px(250.0)), || {
            div()
                .p_6()
                .bg(rgb(0x9b59b6))
                .rounded_xl()
                .flex()
                .flex_col()
                .gap_3()
                .child(div().text_3xl().text_color(white()).child("Purple Box"))
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(0xd6b4fc))
                        .child("A larger card with more padding"),
                )
                .child(
                    div()
                        .mt_2()
                        .p_3()
                        .bg(rgb(0x8e44ad))
                        .rounded_lg()
                        .text_sm()
                        .text_color(white())
                        .child("Nested content box"),
                )
                .into_any_element()
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
                    title: Some("Textured Canvas Example".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| canvas_view),
        )
        .unwrap();
    });
}

/// A view that displays a textured canvas
struct TexturedCanvasView {
    provider: TexturedCanvasItemsProvider,
    camera: Camera,
    is_panning: bool,
    last_mouse_pos: Point<Pixels>,
}

impl TexturedCanvasView {
    fn new(provider: TexturedCanvasItemsProvider) -> Self {
        Self {
            provider,
            camera: Camera::default(),
            is_panning: false,
            last_mouse_pos: point(px(0.0), px(0.0)),
        }
    }
}

impl Render for TexturedCanvasView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process render queue (throttled)
        if self.provider.tick() {
            // More work pending, request another frame
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

            // Create element based on texture state
            let content: AnyElement = match self.provider.texture_state(&item.id) {
                TextureState::Ready { ref image, .. } => img(image.clone())
                    .size_full()
                    .object_fit(ObjectFit::Fill)
                    .into_any_element(),
                TextureState::Queued | TextureState::NotRequested => div()
                    .size_full()
                    .bg(rgb(0xe0e0e0))
                    .flex()
                    .justify_center()
                    .items_center()
                    .text_color(rgb(0x666666))
                    .child("Loading...")
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
            // Scroll for zoom
            .on_scroll_wheel({
                cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                    let delta = event.delta.pixel_delta(px(1.0));
                    let delta_y: f32 = delta.y.into();
                    let zoom_factor = 1.0 + (delta_y / 500.0);
                    this.camera.zoom = (this.camera.zoom * zoom_factor).clamp(0.1, 10.0);
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
                        "Items: {}/{} ready | Pending: {} | Zoom: {:.0}% | Pan: middle-drag | Zoom: scroll",
                        ready, total, pending, self.camera.zoom * 100.0
                    )),
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
