//! Basic example of the infinite canvas component.
//!
//! This example demonstrates:
//! - Creating an infinite canvas
//! - Adding items with different positions and sizes
//! - Pan and zoom functionality
//! - Layout-based item sizing (measuring content height at a given width)
//!
//! Run with: cargo run -p infinite-canvas --example basic

use gpui::*;
use infinite_canvas::prelude::*;
use rand::Rng;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1024.0), px(768.0)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Infinite Canvas - Basic Example".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| ExampleView::new()),
        )
        .unwrap();
    });
}

/// Canvas item with measured layout
struct MeasuredItem {
    id: String,
    position: Point<Pixels>,
    /// The desired width for layout
    layout_width: Pixels,
    /// The measured height (computed during render)
    measured_size: Option<Size<Pixels>>,
    #[allow(dead_code)]
    z_index: i32,
    /// Number of Lorem Ipsum paragraphs (randomized 1-10)
    paragraph_count: usize,
}

struct ExampleView {
    camera: Camera,
    items: Vec<MeasuredItem>,
    is_panning: bool,
    last_mouse_pos: Point<Pixels>,
    /// Whether we need to re-measure items
    needs_measure: bool,
}

impl ExampleView {
    fn new() -> Self {
        let mut rng = rand::thread_rng();

        let range = 1..=5;
        // Create items with positions and desired widths
        // Heights will be measured based on content
        // Each item gets 1-10 random paragraphs
        let items = vec![
            MeasuredItem {
                id: "item-1".to_string(),
                position: point(px(50.0), px(50.0)),
                layout_width: px(300.0),
                measured_size: None,
                z_index: 1,
                paragraph_count: rng.gen_range(range.clone()),
            },
            MeasuredItem {
                id: "item-2".to_string(),
                position: point(px(400.0), px(80.0)),
                layout_width: px(280.0),
                measured_size: None,
                z_index: 2,
                paragraph_count: rng.gen_range(range.clone()),
            },
            MeasuredItem {
                id: "item-3".to_string(),
                position: point(px(100.0), px(300.0)),
                layout_width: px(320.0),
                measured_size: None,
                z_index: 1,
                paragraph_count: rng.gen_range(range.clone()),
            },
            MeasuredItem {
                id: "item-4".to_string(),
                position: point(px(500.0), px(320.0)),
                layout_width: px(280.0),
                measured_size: None,
                z_index: 3,
                paragraph_count: rng.gen_range(range.clone()),
            },
            MeasuredItem {
                id: "item-5".to_string(),
                position: point(px(50.0), px(580.0)),
                layout_width: px(350.0),
                measured_size: None,
                z_index: 2,
                paragraph_count: rng.gen_range(range.clone()),
            },
        ];

        Self {
            camera: Camera::default(),
            items,
            is_panning: false,
            last_mouse_pos: point(px(0.0), px(0.0)),
            needs_measure: true,
        }
    }

    /// Create the content element for an item (used for both measurement and rendering)
    fn create_item_content(&self, item_id: &str, zoom: f32, paragraph_count: usize) -> AnyElement {
        // Calculate zoom-aware font sizes (clamped to reasonable bounds)
        let title_size = (18.0 * zoom).clamp(10.0, 72.0);
        let text_size = (14.0 * zoom).clamp(8.0, 48.0);
        let padding = (16.0 * zoom).clamp(8.0, 64.0);
        let gap = (8.0 * zoom).clamp(4.0, 32.0);

        // Background color based on item ID
        let bg_color = match item_id {
            "item-1" => rgb(0xffffff),
            "item-2" => rgb(0xfff9e6),
            "item-3" => rgb(0xe6f7ff),
            "item-4" => rgb(0xffeef0),
            "item-5" => rgb(0xf3e6ff),
            _ => rgb(0xf5f5f5),
        };

        let title = format!("Lorem Ipsum - {} paragraph{}", paragraph_count, if paragraph_count == 1 { "" } else { "s" });

        // Lorem Ipsum paragraph pool
        let paragraphs = [
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
            "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
            "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.",
            "Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet.",
            "At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti atque corrupti quos dolores et quas molestias excepturi sint occaecati cupiditate non provident.",
            "Temporibus autem quibusdam et aut officiis debitis aut rerum necessitatibus saepe eveniet ut et voluptates repudiandae sint et molestiae non recusandae. Itaque earum rerum hic tenetur a sapiente delectus.",
            "Nam libero tempore, cum soluta nobis est eligendi optio cumque nihil impedit quo minus id quod maxime placeat facere possimus, omnis voluptas assumenda est, omnis dolor repellendus.",
            "Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur.",
            "Et harum quidem rerum facilis est et expedita distinctio. Nam libero tempore, cum soluta nobis est eligendi optio cumque nihil impedit quo minus id quod maxime placeat facere possimus.",
            "Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem.",
        ];

        // Take the requested number of paragraphs, cycling if needed
        let text_paragraphs: Vec<_> = (0..paragraph_count)
            .map(|i| paragraphs[i % paragraphs.len()])
            .collect();

        div()
            .size_full()
            .bg(bg_color)
            .rounded(px(8.0 * zoom))
            .border_1()
            .border_color(rgb(0xdddddd))
            .shadow_lg()
            .p(px(padding))
            .flex()
            .flex_col()
            .gap(px(gap))
            .child(
                div()
                    .text_size(px(title_size))
                    .text_color(rgb(0x333333))
                    .font_weight(FontWeight::BOLD)
                    .child(title),
            )
            .children(text_paragraphs.iter().map(|para| {
                div()
                    .text_size(px(text_size))
                    .text_color(rgb(0x666666))
                    .line_height(rems(1.5))
                    .mb(px(gap / 2.0))
                    .child(*para)
            }))
            .into_any_element()
    }
}

impl Render for ExampleView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport_size = window.viewport_size();
        let viewport_bounds = Bounds::new(point(px(0.0), px(0.0)), viewport_size);

        // Measure items if needed (at zoom=1.0 for base measurements)
        if self.needs_measure {
            // Collect items that need measuring (to avoid borrow issues)
            let items_to_measure: Vec<(usize, String, Pixels, usize)> = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.measured_size.is_none())
                .map(|(i, item)| (i, item.id.clone(), item.layout_width, item.paragraph_count))
                .collect();

            for (idx, id, layout_width, paragraph_count) in items_to_measure {
                let mut element = self.create_item_content(&id, 1.0, paragraph_count);
                let available_space = size(
                    AvailableSpace::Definite(layout_width),
                    AvailableSpace::MinContent,
                );
                let measured = element.layout_as_root(available_space, window, cx);
                self.items[idx].measured_size = Some(measured);
            }
            self.needs_measure = false;
        }

        // Render visible items
        let mut rendered_items: Vec<AnyElement> = Vec::new();

        for item in &self.items {
            // Get the measured size (or use layout_width with a default height)
            let base_size = item.measured_size.unwrap_or(size(item.layout_width, px(100.0)));

            // Transform item bounds by camera
            let screen_origin = self.camera.canvas_to_screen(item.position);
            let screen_size = Size::new(
                base_size.width * self.camera.zoom,
                base_size.height * self.camera.zoom,
            );
            let screen_bounds = Bounds::new(screen_origin, screen_size);

            // Cull items outside viewport
            if !bounds_intersect(&screen_bounds, &viewport_bounds) {
                continue;
            }

            let content = self.create_item_content(&item.id, self.camera.zoom, item.paragraph_count);

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

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // Header
                div()
                    .w_full()
                    .h(px(48.0))
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x252525))
                    .border_b_1()
                    .border_color(rgb(0x3d3d3d))
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0xcccccc))
                            .child("Infinite Canvas - Basic Example"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child("Middle-drag to pan • Scroll to zoom"),
                    ),
            )
            .child(
                // Canvas area
                div()
                    .flex_1()
                    .w_full()
                    .bg(rgb(0xf5f5f5))
                    .overflow_hidden()
                    .relative()
                    // Pan with middle mouse button
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
                                this.camera.pan(delta);
                                this.last_mouse_pos = event.position;
                                cx.notify();
                            }
                        })
                    })
                    // Zoom with scroll wheel
                    .on_scroll_wheel({
                        cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                            let delta = event.delta.pixel_delta(px(1.0));
                            let zoom_delta: f32 = delta.y.into();

                            // Zoom sensitivity: 2x normal, 4x with Ctrl
                            let divisor = if event.modifiers.control {
                                60.0 // ~8x sensitivity with Ctrl
                            } else {
                                125.0 // 4x sensitivity
                            };
                            let zoom_factor = 1.0 - (zoom_delta / divisor);
                            let new_zoom = (this.camera.zoom * zoom_factor).clamp(0.1, 5.0);

                            // Zoom centered on mouse position
                            // Convert mouse position to canvas space before zoom
                            let mouse_canvas_before = this.camera.screen_to_canvas(event.position);

                            // Apply zoom
                            this.camera.zoom = new_zoom;

                            // Convert mouse position to canvas space after zoom
                            let mouse_canvas_after = this.camera.screen_to_canvas(event.position);

                            // Adjust offset to keep mouse position fixed in canvas space
                            let canvas_delta = Point::new(
                                mouse_canvas_after.x - mouse_canvas_before.x,
                                mouse_canvas_after.y - mouse_canvas_before.y,
                            );
                            this.camera.offset.x += canvas_delta.x * this.camera.zoom;
                            this.camera.offset.y += canvas_delta.y * this.camera.zoom;

                            cx.notify();
                        })
                    })
                    // Grid background (simple colored div)
                    .child(
                        div()
                            .absolute()
                            .size_full()
                            .bg(rgb(0xfafafa)),
                    )
                    // Canvas items
                    .children(rendered_items),
            )
            .child(
                // Footer with stats
                div()
                    .w_full()
                    .h(px(32.0))
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x252525))
                    .border_t_1()
                    .border_color(rgb(0x3d3d3d))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .child(format!(
                                "Items: {} | Zoom: {:.0}%",
                                self.items.len(),
                                self.camera.zoom * 100.0
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x666666))
                            .child(format!(
                                "Offset: ({:.0}, {:.0})",
                                Into::<f32>::into(self.camera.offset.x),
                                Into::<f32>::into(self.camera.offset.y)
                            )),
                    ),
            )
    }
}

/// Check if two bounds intersect (simple AABB collision)
fn bounds_intersect(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> bool {
    a.origin.x < b.origin.x + b.size.width
        && a.origin.x + a.size.width > b.origin.x
        && a.origin.y < b.origin.y + b.size.height
        && a.origin.y + a.size.height > b.origin.y
}
