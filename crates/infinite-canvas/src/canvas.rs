//! The main infinite canvas component.
//!
//! This module provides the `InfiniteCanvas` component which renders
//! a pannable, zoomable canvas with items.

use gpui::{
    div, point, prelude::*, px, AnyElement, App, Bounds, DefiniteLength, Element, ElementId,
    GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId, IntoElement, LayoutId, Length,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point,
    ScrollWheelEvent, Size, Style, Styled, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::camera::Camera;
use crate::item::{CanvasItem, CanvasItems};
use crate::options::CanvasOptions;

/// Persistent state for the canvas element, stored in GPUI's element state system.
/// This persists across re-renders when the element has the same ID.
#[derive(Default)]
struct CanvasElementState {
    /// Current camera state (persists across renders).
    camera: Option<Rc<RefCell<Camera>>>,

    /// Whether we're currently panning with middle mouse.
    is_panning: Option<Rc<RefCell<bool>>>,

    /// The last mouse position during a pan operation.
    last_pan_position: Option<Rc<RefCell<Point<Pixels>>>>,
}

/// The infinite canvas component.
///
/// An infinite canvas provides a pannable, zoomable viewport into an unbounded
/// 2D space where items can be placed and arranged.
///
/// # Features
///
/// - Pan with middle mouse button or space+drag
/// - Zoom with scroll wheel (centered on cursor)
/// - Keyboard shortcuts for navigation
/// - Background grid display
/// - Viewport culling for performance
///
/// # Example
///
/// ```no_run
/// use infinite_canvas::{InfiniteCanvas, CanvasItem, Camera};
/// use gpui::{point, px, size, Bounds};
///
/// let canvas = InfiniteCanvas::new("my-canvas")
///     .camera(Camera::default())
///     .items(vec![
///         CanvasItem::new("item-1", Bounds::new(point(px(0.), px(0.)), size(px(100.), px(80.)))),
///     ]);
/// ```
#[derive(IntoElement)]
pub struct InfiniteCanvas<D: Clone + 'static = ()> {
    /// Unique identifier for this canvas.
    id: ElementId,

    /// Initial camera state (used on first render only).
    initial_camera: Camera,

    /// Canvas options.
    options: CanvasOptions,

    /// Items to display on the canvas.
    items: CanvasItems<D>,

    /// Optional callback when camera changes.
    on_camera_change: Option<Rc<dyn Fn(Camera) + 'static>>,

    /// Optional callback when an item is clicked.
    on_item_click: Option<Box<dyn Fn(&CanvasItem<D>) + 'static>>,

    /// Custom item renderer.
    render_item: Option<Box<dyn Fn(&CanvasItem<D>, &Camera, &mut Window, &mut App) -> AnyElement>>,
}

impl<D: Clone + 'static> InfiniteCanvas<D> {
    /// Create a new infinite canvas with the given ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            initial_camera: Camera::default(),
            options: CanvasOptions::default(),
            items: CanvasItems::new(),
            on_camera_change: None,
            on_item_click: None,
            render_item: None,
        }
    }

    /// Set the initial camera state.
    /// Note: This only affects the first render. After that, the canvas maintains its own state.
    pub fn camera(mut self, camera: Camera) -> Self {
        self.initial_camera = camera;
        self
    }

    /// Set the canvas options.
    pub fn options(mut self, options: CanvasOptions) -> Self {
        self.options = options;
        self
    }

    /// Set the items to display.
    pub fn items(mut self, items: impl Into<CanvasItems<D>>) -> Self {
        self.items = items.into();
        self
    }

    /// Add a single item.
    pub fn item(mut self, item: CanvasItem<D>) -> Self {
        self.items.push(item);
        self
    }

    /// Set the camera change callback.
    pub fn on_camera_change(mut self, callback: impl Fn(Camera) + 'static) -> Self {
        self.on_camera_change = Some(Rc::new(callback));
        self
    }

    /// Set the item click callback.
    pub fn on_item_click(mut self, callback: impl Fn(&CanvasItem<D>) + 'static) -> Self {
        self.on_item_click = Some(Box::new(callback));
        self
    }

    /// Set a custom item renderer.
    pub fn render_item_with(
        mut self,
        renderer: impl Fn(&CanvasItem<D>, &Camera, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.render_item = Some(Box::new(renderer));
        self
    }
}

impl<D: Clone + 'static> RenderOnce for InfiniteCanvas<D> {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial_camera = self.initial_camera;
        let options = self.options.clone();
        let items = self.items.clone();

        div()
            .id(id)
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(gpui::rgb(0x1e1e1e)) // Dark background
            .child(CanvasElement {
                initial_camera,
                options,
                items,
                render_item: self.render_item,
                on_camera_change: self.on_camera_change,
                on_item_click: self.on_item_click,
            })
    }
}

impl<D: Clone + 'static> From<Vec<CanvasItem<D>>> for CanvasItems<D> {
    fn from(items: Vec<CanvasItem<D>>) -> Self {
        CanvasItems::from_vec(items)
    }
}

// ============================================================================
// Canvas Element (handles rendering and interaction)
// ============================================================================

/// Internal element that handles canvas rendering and interaction.
struct CanvasElement<D: Clone + 'static> {
    initial_camera: Camera,
    options: CanvasOptions,
    items: CanvasItems<D>,
    #[allow(dead_code)]
    render_item: Option<Box<dyn Fn(&CanvasItem<D>, &Camera, &mut Window, &mut App) -> AnyElement>>,
    on_camera_change: Option<Rc<dyn Fn(Camera) + 'static>>,
    #[allow(dead_code)]
    on_item_click: Option<Box<dyn Fn(&CanvasItem<D>) + 'static>>,
}

/// State needed after layout for painting.
struct CanvasElementPrepaintState {
    hitbox: Hitbox,
    camera: Rc<RefCell<Camera>>,
    is_panning: Rc<RefCell<bool>>,
    last_pan_position: Rc<RefCell<Point<Pixels>>>,
}

impl<D: Clone + 'static> IntoElement for CanvasElement<D> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<D: Clone + 'static> Element for CanvasElement<D> {
    type RequestLayoutState = ();
    type PrepaintState = CanvasElementPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some("canvas-element".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // Make the element fill its parent
        let mut style = Style::default();
        style.size.width = Length::Definite(DefiniteLength::Fraction(1.0));
        style.size.height = Length::Definite(DefiniteLength::Fraction(1.0));

        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        // Get or initialize persistent element state
        let initial_camera = self.initial_camera;
        let (camera, is_panning, last_pan_position) = window
            .with_optional_element_state::<CanvasElementState, _>(
                global_id,
                |element_state, _window| {
                    let mut state = element_state
                        .map(|s| s.unwrap_or_default())
                        .unwrap_or_default();

                    // Initialize camera if not present
                    let camera = state
                        .camera
                        .get_or_insert_with(|| Rc::new(RefCell::new(initial_camera)))
                        .clone();

                    // Initialize panning state
                    let is_panning = state
                        .is_panning
                        .get_or_insert_with(|| Rc::new(RefCell::new(false)))
                        .clone();

                    // Initialize last pan position
                    let last_pan_position = state
                        .last_pan_position
                        .get_or_insert_with(|| Rc::new(RefCell::new(point(px(0.), px(0.)))))
                        .clone();

                    ((camera, is_panning, last_pan_position), Some(state))
                },
            );

        CanvasElementPrepaintState {
            hitbox,
            camera,
            is_panning,
            last_pan_position,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let camera = *prepaint.camera.borrow();
        let options = &self.options;
        let hitbox = &prepaint.hitbox;

        // Draw background grid if enabled
        if options.show_grid {
            self.paint_grid(bounds, &camera, options, window, cx);
        }

        // Draw items
        let viewport_size = bounds.size;
        let visible_bounds = camera.visible_canvas_bounds(viewport_size);

        // Sort items by z-index for proper layering
        let mut items_to_render: Vec<_> = self
            .items
            .iter()
            .filter(|item| item.visible && item.intersects(&visible_bounds))
            .collect();
        items_to_render.sort_by_key(|item| item.z_index);

        for item in items_to_render {
            // Convert item bounds to screen space
            let screen_bounds = camera.canvas_to_screen_bounds(item.bounds);

            // Adjust for canvas position within window
            let adjusted_bounds = Bounds::new(
                point(
                    bounds.origin.x + screen_bounds.origin.x,
                    bounds.origin.y + screen_bounds.origin.y,
                ),
                screen_bounds.size,
            );

            // Skip items that are completely outside the viewport
            if adjusted_bounds.origin.x + adjusted_bounds.size.width < bounds.origin.x
                || adjusted_bounds.origin.y + adjusted_bounds.size.height < bounds.origin.y
                || adjusted_bounds.origin.x > bounds.origin.x + bounds.size.width
                || adjusted_bounds.origin.y > bounds.origin.y + bounds.size.height
            {
                continue;
            }

            // Render the item (background)
            self.paint_default_item(adjusted_bounds, item, window, cx);
        }

        // Set up mouse event handlers
        let view_id = window.current_view();
        let hitbox_id = hitbox.id;

        // Handle scroll wheel for zooming
        if !options.locked {
            let camera_rc = prepaint.camera.clone();
            let options_clone = options.clone();
            let on_camera_change = self.on_camera_change.clone();

            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase.bubble() && hitbox_id.is_hovered(window)
                    && options_clone.wheel_behavior.is_zoom() {
                        let mut camera = camera_rc.borrow_mut();
                        let delta = event.delta.pixel_delta(px(20.));
                        let zoom_factor =
                            1.0 - f32::from(delta.y) * options_clone.zoom_speed * 0.001;

                        camera.zoom_around(
                            zoom_factor,
                            event.position,
                            options_clone.min_zoom,
                            options_clone.max_zoom,
                        );

                        let new_camera = *camera;
                        drop(camera); // Release borrow before callback

                        if let Some(ref callback) = on_camera_change {
                            callback(new_camera);
                        }

                        window.refresh();
                        cx.notify(view_id);
                    }
            });
        }

        // Handle mouse down for starting pan (middle mouse button)
        if !options.locked {
            let is_panning = prepaint.is_panning.clone();
            let last_pan_position = prepaint.last_pan_position.clone();

            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _cx| {
                if phase.bubble() && hitbox_id.is_hovered(window)
                    && event.button == MouseButton::Middle {
                        *is_panning.borrow_mut() = true;
                        *last_pan_position.borrow_mut() = event.position;
                    }
            });
        }

        // Handle mouse move for panning
        if !options.locked {
            let camera_rc = prepaint.camera.clone();
            let is_panning = prepaint.is_panning.clone();
            let last_pan_position = prepaint.last_pan_position.clone();
            let on_camera_change = self.on_camera_change.clone();

            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase.bubble() {
                    let panning = *is_panning.borrow();
                    // Check if we're panning with middle mouse button held
                    if panning && event.pressed_button == Some(MouseButton::Middle) {
                        let last_pos = *last_pan_position.borrow();
                        let delta =
                            point(event.position.x - last_pos.x, event.position.y - last_pos.y);

                        let mut camera = camera_rc.borrow_mut();
                        camera.pan(delta);
                        let new_camera = *camera;
                        drop(camera); // Release borrow

                        *last_pan_position.borrow_mut() = event.position;

                        if let Some(ref callback) = on_camera_change {
                            callback(new_camera);
                        }

                        window.refresh();
                        cx.notify(view_id);
                    }
                }
            });
        }

        // Handle mouse up for ending pan
        if !options.locked {
            let is_panning = prepaint.is_panning.clone();

            window.on_mouse_event(move |event: &MouseUpEvent, phase, _window, _cx| {
                if phase.bubble()
                    && event.button == MouseButton::Middle {
                        *is_panning.borrow_mut() = false;
                    }
            });
        }
    }
}

impl<D: Clone + 'static> CanvasElement<D> {
    /// Paint the background grid.
    fn paint_grid(
        &self,
        bounds: Bounds<Pixels>,
        camera: &Camera,
        options: &CanvasOptions,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let grid_size = options.grid_size * camera.zoom;

        // Don't draw grid if cells are too small
        if f32::from(grid_size) < 5.0 {
            return;
        }

        let grid_color = gpui::rgba(0xffffff20); // Subtle white grid lines

        // Calculate grid offset based on camera position
        let offset_x_f32: f32 = camera.offset.x.into();
        let offset_y_f32: f32 = camera.offset.y.into();
        let grid_size_f32: f32 = grid_size.into();

        let offset_x = px(offset_x_f32.rem_euclid(grid_size_f32));
        let offset_y = px(offset_y_f32.rem_euclid(grid_size_f32));

        // Draw vertical lines
        let mut x = bounds.origin.x + offset_x;
        while x < bounds.origin.x + bounds.size.width + grid_size {
            window.paint_quad(gpui::fill(
                Bounds::new(
                    point(x, bounds.origin.y),
                    Size::new(px(1.), bounds.size.height),
                ),
                grid_color,
            ));
            x += grid_size;
        }

        // Draw horizontal lines
        let mut y = bounds.origin.y + offset_y;
        while y < bounds.origin.y + bounds.size.height + grid_size {
            window.paint_quad(gpui::fill(
                Bounds::new(
                    point(bounds.origin.x, y),
                    Size::new(bounds.size.width, px(1.)),
                ),
                grid_color,
            ));
            y += grid_size;
        }
    }

    /// Paint a default item representation.
    fn paint_default_item(
        &self,
        bounds: Bounds<Pixels>,
        item: &CanvasItem<D>,
        window: &mut Window,
        _cx: &mut App,
    ) {
        // Background
        let bg_color = if item.selected {
            gpui::rgb(0x3a5f8a) // Highlighted blue
        } else {
            gpui::rgb(0x2d2d2d) // Dark gray
        };

        let border_color = if item.selected {
            gpui::rgb(0x5a9fd4)
        } else {
            gpui::rgb(0x3d3d3d)
        };

        // Draw background with rounded corners
        window.paint_quad(gpui::PaintQuad {
            bounds,
            corner_radii: gpui::Corners::all(px(4.)),
            background: bg_color.into(),
            border_widths: gpui::Edges::all(px(1.)),
            border_color: border_color.into(),
            border_style: gpui::BorderStyle::Solid,
        });
    }
}

// ============================================================================
// Canvas Handle (for external control)
// ============================================================================

/// A handle for controlling an infinite canvas from outside.
///
/// This can be used to programmatically pan, zoom, or manipulate
/// the canvas state.
#[derive(Clone, Debug)]
pub struct CanvasHandle {
    camera: Camera,
    options: CanvasOptions,
}

impl CanvasHandle {
    /// Create a new canvas handle.
    pub fn new() -> Self {
        Self {
            camera: Camera::default(),
            options: CanvasOptions::default(),
        }
    }

    /// Get the current camera state.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Set the camera.
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    /// Get the options.
    pub fn options(&self) -> &CanvasOptions {
        &self.options
    }

    /// Set the options.
    pub fn set_options(&mut self, options: CanvasOptions) {
        self.options = options;
    }

    /// Reset the camera to default state.
    pub fn reset(&mut self) {
        self.camera.reset();
    }

    /// Zoom to fit the given bounds.
    pub fn zoom_to_fit(&mut self, bounds: Bounds<Pixels>, viewport_size: Size<Pixels>) {
        self.camera.zoom_to_fit(
            bounds,
            viewport_size,
            px(20.),
            self.options.min_zoom,
            self.options.max_zoom,
        );
    }

    /// Step zoom in.
    pub fn zoom_in(&mut self, viewport_size: Size<Pixels>) {
        let new_zoom = self.camera.next_zoom_step(&self.options.zoom_steps);
        self.camera.set_zoom(
            new_zoom,
            viewport_size,
            self.options.min_zoom,
            self.options.max_zoom,
        );
    }

    /// Step zoom out.
    pub fn zoom_out(&mut self, viewport_size: Size<Pixels>) {
        let new_zoom = self.camera.prev_zoom_step(&self.options.zoom_steps);
        self.camera.set_zoom(
            new_zoom,
            viewport_size,
            self.options.min_zoom,
            self.options.max_zoom,
        );
    }

    /// Reset zoom to 100%.
    pub fn reset_zoom(&mut self, viewport_size: Size<Pixels>) {
        self.camera.set_zoom(
            1.0,
            viewport_size,
            self.options.min_zoom,
            self.options.max_zoom,
        );
    }

    /// Center on a canvas point.
    pub fn center_on(&mut self, point: Point<Pixels>, viewport_size: Size<Pixels>) {
        self.camera.center_on(point, viewport_size);
    }
}

impl Default for CanvasHandle {
    fn default() -> Self {
        Self::new()
    }
}
