//! The main infinite canvas component.
//!
//! This module provides the `InfiniteCanvas` component which renders
//! a pannable, zoomable canvas with items from a `CanvasItemsProvider`.

use gpui::{
    point, px, AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId,
    Hitbox, HitboxBehavior, InspectorElementId, IntoElement, LayoutId, Length, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollWheelEvent, Size, Style,
    Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::camera::Camera;
use crate::options::CanvasOptions;
use crate::provider::{CanvasItemsProvider, ItemDescriptor};

/// A shared reference to a canvas items provider.
pub type SharedProvider<P> = Rc<RefCell<P>>;

/// Persistent state for the canvas element, stored in GPUI's element state system.
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
/// # Type Parameters
///
/// * `P` - The items provider type (implements `CanvasItemsProvider + Clone`)
///
/// # Features
///
/// - Pan with middle mouse button
/// - Zoom with scroll wheel (centered on cursor)
/// - Background grid display
/// - Viewport culling for performance
///
/// # Example
///
/// ```ignore
/// use infinite_canvas::{InfiniteCanvas, TexturedCanvasItemsProvider};
///
/// let provider = Rc::new(RefCell::new(TexturedCanvasItemsProvider::new()));
/// // ... add items to provider ...
///
/// let canvas = InfiniteCanvas::new("my-canvas", provider.clone())
///     .options(CanvasOptions::new().show_grid(true));
/// ```
pub struct InfiniteCanvas<P: CanvasItemsProvider + 'static> {
    /// Unique identifier for this canvas.
    id: ElementId,
    /// The items provider (shared via Rc<RefCell<>>).
    provider: SharedProvider<P>,
    /// Initial camera state (used on first render only).
    initial_camera: Camera,
    /// Canvas options.
    options: CanvasOptions,
    /// Optional callback when camera changes.
    on_camera_change: Option<Rc<dyn Fn(Camera) + 'static>>,
}

impl<P: CanvasItemsProvider + 'static> InfiniteCanvas<P> {
    /// Create a new infinite canvas with the given ID and provider.
    pub fn new(id: impl Into<ElementId>, provider: SharedProvider<P>) -> Self {
        Self {
            id: id.into(),
            provider,
            initial_camera: Camera::default(),
            options: CanvasOptions::default(),
            on_camera_change: None,
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

    /// Set the camera change callback.
    pub fn on_camera_change(mut self, callback: impl Fn(Camera) + 'static) -> Self {
        self.on_camera_change = Some(Rc::new(callback));
        self
    }
}

impl<P: CanvasItemsProvider + 'static> IntoElement for InfiniteCanvas<P> {
    type Element = CanvasElement<P>;

    fn into_element(self) -> Self::Element {
        CanvasElement {
            id: self.id,
            provider: self.provider,
            initial_camera: self.initial_camera,
            options: self.options,
            on_camera_change: self.on_camera_change,
        }
    }
}

/// The element that implements the canvas rendering and interaction.
pub struct CanvasElement<P: CanvasItemsProvider + 'static> {
    id: ElementId,
    provider: SharedProvider<P>,
    initial_camera: Camera,
    options: CanvasOptions,
    on_camera_change: Option<Rc<dyn Fn(Camera) + 'static>>,
}

impl<P: CanvasItemsProvider + 'static> IntoElement for CanvasElement<P> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// State needed after layout for painting.
pub struct CanvasElementPrepaintState {
    hitbox: Hitbox,
    camera: Rc<RefCell<Camera>>,
    is_panning: Rc<RefCell<bool>>,
    last_pan_position: Rc<RefCell<Point<Pixels>>>,
    /// Elements to paint (prepared during prepaint)
    item_elements: Vec<AnyElement>,
}

impl<P: CanvasItemsProvider + 'static> Element for CanvasElement<P> {
    type RequestLayoutState = ();
    type PrepaintState = CanvasElementPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
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
        let mut style = Style::default();
        style.size.width = Length::Definite(gpui::DefiniteLength::Fraction(1.0));
        style.size.height = Length::Definite(gpui::DefiniteLength::Fraction(1.0));

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
        cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        let initial_camera = self.initial_camera;
        let (camera, is_panning, last_pan_position) = window
            .with_optional_element_state::<CanvasElementState, _>(
                global_id,
                |element_state, _window| {
                    let mut state = element_state
                        .map(|s| s.unwrap_or_default())
                        .unwrap_or_default();

                    let camera = state
                        .camera
                        .get_or_insert_with(|| Rc::new(RefCell::new(initial_camera)))
                        .clone();

                    let is_panning = state
                        .is_panning
                        .get_or_insert_with(|| Rc::new(RefCell::new(false)))
                        .clone();

                    let last_pan_position = state
                        .last_pan_position
                        .get_or_insert_with(|| Rc::new(RefCell::new(point(px(0.), px(0.)))))
                        .clone();

                    ((camera, is_panning, last_pan_position), Some(state))
                },
            );

        // Prepare item elements during prepaint phase
        let camera_val = *camera.borrow();
        let viewport_size = bounds.size;
        let visible_canvas_bounds = camera_val.visible_canvas_bounds(viewport_size);

        // Use items_with_context to get measured sizes (e.g., for FixedWidth mode)
        let mut items: Vec<ItemDescriptor> = self.provider.borrow().items_with_context(cx);
        items.sort_by_key(|item| item.z_index);

        for item in &items {
            log::debug!(
                "[Canvas] Item '{}': canvas_bounds={:?}",
                item.id,
                item.bounds
            );
        }

        let mut item_elements: Vec<AnyElement> = Vec::new();

        for item in items {
            // Check if item intersects visible area
            if !item.bounds.intersects(&visible_canvas_bounds) {
                continue;
            }

            // Transform item bounds to screen space
            let screen_bounds = camera_val.canvas_to_screen_bounds(item.bounds);
            log::debug!(
                "[Canvas] Item '{}': screen_bounds={:?}",
                item.id,
                screen_bounds
            );

            // Adjust for canvas position within window
            let adjusted_bounds = Bounds::new(
                point(
                    bounds.origin.x + screen_bounds.origin.x,
                    bounds.origin.y + screen_bounds.origin.y,
                ),
                screen_bounds.size,
            );

            // Skip items completely outside the canvas bounds
            if adjusted_bounds.origin.x + adjusted_bounds.size.width < bounds.origin.x
                || adjusted_bounds.origin.y + adjusted_bounds.size.height < bounds.origin.y
                || adjusted_bounds.origin.x > bounds.origin.x + bounds.size.width
                || adjusted_bounds.origin.y > bounds.origin.y + bounds.size.height
            {
                continue;
            }

            // Get element from provider and prepare it
            if let Some(mut element) =
                self.provider
                    .borrow()
                    .render_item(&item.id, adjusted_bounds, cx)
            {
                element.prepaint_as_root(
                    adjusted_bounds.origin,
                    Size {
                        width: AvailableSpace::Definite(adjusted_bounds.size.width),
                        height: AvailableSpace::Definite(adjusted_bounds.size.height),
                    },
                    window,
                    cx,
                );
                item_elements.push(element);
            }
        }

        CanvasElementPrepaintState {
            hitbox,
            camera,
            is_panning,
            last_pan_position,
            item_elements,
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

        // Draw background
        window.paint_quad(gpui::fill(bounds, gpui::rgb(0x1e1e1e)));

        // Draw background grid if enabled
        if options.show_grid {
            self.paint_grid(bounds, &camera, options, window);
        }

        // Paint all the item elements that were prepared during prepaint
        for element in &mut prepaint.item_elements {
            element.paint(window, cx);
        }

        // Set up mouse event handlers
        self.setup_event_handlers(prepaint, hitbox.id, window);
    }
}

impl<P: CanvasItemsProvider + 'static> CanvasElement<P> {
    /// Paint the background grid.
    fn paint_grid(
        &self,
        bounds: Bounds<Pixels>,
        camera: &Camera,
        options: &CanvasOptions,
        window: &mut Window,
    ) {
        let grid_size = options.grid_size * camera.zoom;

        // Don't draw grid if cells are too small
        if f32::from(grid_size) < 5.0 {
            return;
        }

        let grid_color = gpui::rgba(0xffffff20);

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

    /// Set up mouse event handlers for pan and zoom.
    fn setup_event_handlers(
        &self,
        prepaint: &CanvasElementPrepaintState,
        hitbox_id: gpui::HitboxId,
        window: &mut Window,
    ) {
        let options = &self.options;
        let view_id = window.current_view();

        // Handle scroll wheel for zooming
        if !options.locked {
            let camera_rc = prepaint.camera.clone();
            let options_clone = options.clone();
            let on_camera_change = self.on_camera_change.clone();

            window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                if phase.bubble()
                    && hitbox_id.is_hovered(window)
                    && options_clone.wheel_behavior.is_zoom()
                {
                    let mut camera = camera_rc.borrow_mut();
                    let delta = event.delta.pixel_delta(px(20.));
                    let zoom_factor = 1.0 - f32::from(delta.y) * options_clone.zoom_speed * 0.001;

                    camera.zoom_around(
                        zoom_factor,
                        event.position,
                        options_clone.min_zoom,
                        options_clone.max_zoom,
                    );

                    let new_camera = *camera;
                    drop(camera);

                    if let Some(ref callback) = on_camera_change {
                        callback(new_camera);
                    }

                    window.refresh();
                    cx.notify(view_id);
                }
            });
        }

        // Handle mouse down for starting pan
        if !options.locked {
            let is_panning = prepaint.is_panning.clone();
            let last_pan_position = prepaint.last_pan_position.clone();

            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _cx| {
                if phase.bubble()
                    && hitbox_id.is_hovered(window)
                    && event.button == MouseButton::Middle
                {
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
                    if panning && event.pressed_button == Some(MouseButton::Middle) {
                        let last_pos = *last_pan_position.borrow();
                        let delta =
                            point(event.position.x - last_pos.x, event.position.y - last_pos.y);

                        let mut camera = camera_rc.borrow_mut();
                        camera.pan(delta);
                        let new_camera = *camera;
                        drop(camera);

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
                if phase.bubble() && event.button == MouseButton::Middle {
                    *is_panning.borrow_mut() = false;
                }
            });
        }
    }
}
