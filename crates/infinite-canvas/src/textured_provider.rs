//! Textured Canvas Items Provider
//!
//! This module provides a canvas items provider that renders items to textures
//! using GPUI's `Application::textured()` feature. Items are rendered once and
//! cached, allowing efficient pan/zoom without re-rendering.
//!
//! # Architecture
//!
//! - Items are defined with closures that return GPUI elements
//! - When a texture is requested, the element is rendered to a texture
//! - Textures are cached and displayed as images
//! - Layout-based sizing: elements size themselves via GPUI's layout system
//!
//! # Example
//!
//! ```ignore
//! use infinite_canvas::textured_provider::TexturedCanvasItemsProvider;
//! use gpui::*;
//!
//! let mut provider = TexturedCanvasItemsProvider::new();
//!
//! provider.add_item("card-1", || {
//!     div()
//!         .p_4()
//!         .bg(rgb(0x3498db))
//!         .child("Hello World")
//!         .into_any_element()
//! });
//! ```
//!
//! # Platform Support
//!
//! Texture rendering requires the vendored GPUI with `Application::textured()` support,
//! which is only available on Linux/FreeBSD. On other platforms, `tick()` will return
//! errors for render requests.

use gpui::{
    div, img, point, px, size, AnyElement, App, Bounds, IntoElement, ObjectFit, ParentElement,
    Pixels, Point, RenderImage, RenderOnce, Size, Styled, StyledImage, Window,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::cell::RefCell;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::rc::Rc;

/// String-based identifier for canvas items
pub type CanvasItemId = String;

/// Description of a canvas item for display
#[derive(Clone, Debug)]
pub struct CanvasItemDescriptor {
    /// Unique identifier
    pub id: CanvasItemId,
    /// Position and size in canvas world coordinates
    pub bounds: Bounds<Pixels>,
    /// Z-order for layering
    pub z_index: u32,
}

/// State of a texture
#[derive(Clone)]
pub enum TextureState {
    /// Not yet requested
    NotRequested,
    /// Queued for rendering
    Queued,
    /// Ready to display
    Ready {
        image: Arc<RenderImage>,
        size: Size<Pixels>,
    },
    /// Rendering failed
    Failed(String),
}

impl std::fmt::Debug for TextureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureState::NotRequested => write!(f, "NotRequested"),
            TextureState::Queued => write!(f, "Queued"),
            TextureState::Ready { size, .. } => write!(f, "Ready({:?})", size),
            TextureState::Failed(e) => write!(f, "Failed({})", e),
        }
    }
}

/// Internal item definition
struct ItemDefinition {
    id: CanvasItemId,
    origin: Point<Pixels>,
    size: Option<Size<Pixels>>,
    z_index: u32,
    factory: Box<dyn Fn() -> AnyElement>,
}

/// Trait for providing canvas items and their textures
pub trait CanvasItemsProvider {
    /// Get all items to display
    fn items(&self) -> Vec<CanvasItemDescriptor>;

    /// Get texture state for an item
    fn texture_state(&self, id: &str) -> TextureState;

    /// Request texture rendering (queues it)
    fn request_texture(&mut self, id: &str);

    /// Process render queue (call once per frame)
    /// Returns true if any work was done
    fn tick(&mut self) -> bool;
}

/// A canvas items provider that renders items to textures
pub struct TexturedCanvasItemsProvider {
    items: HashMap<CanvasItemId, ItemDefinition>,
    textures: HashMap<CanvasItemId, TextureState>,
    render_queue: VecDeque<CanvasItemId>,
    /// Max textures to render per tick (throttling)
    renders_per_tick: usize,
}

impl TexturedCanvasItemsProvider {
    /// Create a new textured canvas items provider
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            textures: HashMap::new(),
            render_queue: VecDeque::new(),
            renders_per_tick: 1,
        }
    }

    /// Add an item (auto-positioned at origin)
    pub fn add_item<F>(&mut self, id: impl Into<String>, factory: F)
    where
        F: Fn() -> AnyElement + 'static,
    {
        self.add_item_at(id, point(px(0.0), px(0.0)), factory);
    }

    /// Add an item at a specific position
    pub fn add_item_at<F>(&mut self, id: impl Into<String>, origin: Point<Pixels>, factory: F)
    where
        F: Fn() -> AnyElement + 'static,
    {
        let id = id.into();

        self.items.insert(
            id.clone(),
            ItemDefinition {
                id: id.clone(),
                origin,
                size: None,
                z_index: 0,
                factory: Box::new(factory),
            },
        );

        self.textures.insert(id, TextureState::NotRequested);
    }

    /// Set item position
    pub fn set_position(&mut self, id: &str, origin: Point<Pixels>) {
        if let Some(item) = self.items.get_mut(id) {
            item.origin = origin;
        }
    }

    /// Set item z-index
    pub fn set_z_index(&mut self, id: &str, z_index: u32) {
        if let Some(item) = self.items.get_mut(id) {
            item.z_index = z_index;
        }
    }

    /// Get item bounds (None if not yet measured)
    pub fn bounds(&self, id: &str) -> Option<Bounds<Pixels>> {
        self.items
            .get(id)
            .and_then(|item| item.size.map(|size| Bounds::new(item.origin, size)))
    }

    /// Remove an item
    pub fn remove_item(&mut self, id: &str) {
        self.items.remove(id);
        self.textures.remove(id);
        self.render_queue.retain(|i| i != id);
    }

    /// Invalidate an item (force re-render)
    pub fn invalidate(&mut self, id: &str) {
        if self.items.contains_key(id) {
            self.textures
                .insert(id.to_string(), TextureState::NotRequested);
        }
    }

    /// Invalidate all items
    pub fn invalidate_all(&mut self) {
        for id in self.items.keys() {
            self.textures.insert(id.clone(), TextureState::NotRequested);
        }
    }

    /// Set throttling (textures per frame)
    pub fn set_renders_per_tick(&mut self, count: usize) {
        self.renders_per_tick = count.max(1);
    }

    /// Get number of pending renders
    pub fn pending_count(&self) -> usize {
        self.render_queue.len()
    }

    /// Check if all textures are ready
    pub fn all_ready(&self) -> bool {
        self.textures
            .values()
            .all(|state| matches!(state, TextureState::Ready { .. }))
    }

    /// Get item IDs
    pub fn item_ids(&self) -> impl Iterator<Item = &str> {
        self.items.keys().map(|s| s.as_str())
    }
}

impl Default for TexturedCanvasItemsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasItemsProvider for TexturedCanvasItemsProvider {
    fn items(&self) -> Vec<CanvasItemDescriptor> {
        self.items
            .values()
            .map(|item| {
                // Use measured size or placeholder
                let item_size = item.size.unwrap_or(size(px(100.0), px(100.0)));
                CanvasItemDescriptor {
                    id: item.id.clone(),
                    bounds: Bounds::new(item.origin, item_size),
                    z_index: item.z_index,
                }
            })
            .collect()
    }

    fn texture_state(&self, id: &str) -> TextureState {
        self.textures
            .get(id)
            .cloned()
            .unwrap_or(TextureState::NotRequested)
    }

    fn request_texture(&mut self, id: &str) {
        if self.items.contains_key(id) {
            if let Some(TextureState::NotRequested) = self.textures.get(id) {
                self.textures.insert(id.to_string(), TextureState::Queued);
                self.render_queue.push_back(id.to_string());
            }
        }
    }

    fn tick(&mut self) -> bool {
        let mut work_done = false;

        for _ in 0..self.renders_per_tick {
            if let Some(id) = self.render_queue.pop_front() {
                if let Some(item) = self.items.get(&id) {
                    // Call the factory to get the element
                    let element = (item.factory)();

                    match render_element_to_texture(element) {
                        Ok((image, measured_size)) => {
                            // Update measured size
                            if let Some(item) = self.items.get_mut(&id) {
                                item.size = Some(measured_size);
                            }

                            self.textures.insert(
                                id,
                                TextureState::Ready {
                                    image,
                                    size: measured_size,
                                },
                            );
                        }
                        Err(e) => {
                            self.textures.insert(id, TextureState::Failed(e));
                        }
                    }
                    work_done = true;
                }
            }
        }

        work_done
    }
}

/// Render an element to a texture
///
/// This requires the vendored GPUI with `Application::textured()` support.
/// On platforms without this support, returns an error.
fn render_element_to_texture(
    element: AnyElement,
) -> Result<(Arc<RenderImage>, Size<Pixels>), String> {
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        let _ = element;
        return Err("Textured rendering only supported on Linux/FreeBSD".into());
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        render_element_to_texture_impl(element)
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn render_element_to_texture_impl(
    element: AnyElement,
) -> Result<(Arc<RenderImage>, Size<Pixels>), String> {
    use gpui::{prelude::*, Application, WindowBounds, WindowOptions};

    // Store element in a way that can be accessed by closures
    let element_cell: Rc<RefCell<Option<AnyElement>>> = Rc::new(RefCell::new(Some(element)));
    let result: Rc<RefCell<Option<Result<(Arc<RenderImage>, Size<Pixels>), String>>>> =
        Rc::new(RefCell::new(None));

    let element_for_app = element_cell.clone();
    let result_for_app = result.clone();

    // Use a fixed size for now - we can't easily measure across App boundaries
    // due to arena lifecycle issues. Elements will be rendered at this size.
    let render_width = 800.0_f32;
    let render_height = 600.0_f32;
    let render_size = size(px(render_width), px(render_height));

    let app = Application::textured();

    app.run(move |cx: &mut App| {
        let element_cell = element_for_app.clone();
        let result_cell = result_for_app.clone();

        let bounds = Bounds::centered(None, render_size, cx);

        let window_result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| RenderView::new(element_cell.clone())),
        );

        match window_result {
            Ok(window) => {
                let handle: gpui::AnyWindowHandle = window.into();
                let result_for_spawn = result_cell.clone();
                let width_copy = render_width;
                let height_copy = render_height;

                cx.spawn(async move |cx| {
                    // Draw to render the content
                    let draw_result =
                        cx.update_window(handle, |_, window, cx| window.draw_and_present(cx));

                    match draw_result {
                        Ok(true) => {
                            // Read pixels
                            let read_result =
                                cx.update_window(handle, |_, window, _cx| window.read_pixels());

                            match read_result {
                                Ok(Some(pixels)) => {
                                    let width = width_copy as u32;
                                    let height = height_copy as u32;

                                    if let Some(image) =
                                        pixels_to_render_image(&pixels, width, height)
                                    {
                                        *result_for_spawn.borrow_mut() = Some(Ok((
                                            Arc::new(image),
                                            size(px(width_copy), px(height_copy)),
                                        )));
                                    } else {
                                        *result_for_spawn.borrow_mut() =
                                            Some(Err("Failed to convert pixels".into()));
                                    }
                                }
                                Ok(None) => {
                                    *result_for_spawn.borrow_mut() =
                                        Some(Err("read_pixels returned None".into()));
                                }
                                Err(e) => {
                                    *result_for_spawn.borrow_mut() =
                                        Some(Err(format!("Failed to read pixels: {}", e)));
                                }
                            }
                        }
                        Ok(false) => {
                            *result_for_spawn.borrow_mut() =
                                Some(Err("Window not ready for drawing".into()));
                        }
                        Err(e) => {
                            *result_for_spawn.borrow_mut() =
                                Some(Err(format!("update_window failed: {}", e)));
                        }
                    }

                    // Quit after we're done
                    let _ = cx.update(|cx| cx.quit());
                })
                .detach();
            }
            Err(e) => {
                *result_cell.borrow_mut() = Some(Err(format!("Failed to open window: {}", e)));
                cx.quit();
            }
        }
    });

    // Extract result after app.run() completes
    let final_result = result
        .borrow_mut()
        .take()
        .unwrap_or(Err("No result returned".into()));
    final_result
}

/// Convert BGRA pixels to RenderImage
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn pixels_to_render_image(pixels: &[u8], width: u32, height: u32) -> Option<RenderImage> {
    use image::{Frame, RgbaImage};
    use smallvec::smallvec;

    // Convert BGRA to RGBA
    let mut rgba = Vec::with_capacity(pixels.len());
    for chunk in pixels.chunks(4) {
        if chunk.len() == 4 {
            rgba.push(chunk[2]); // R (was B)
            rgba.push(chunk[1]); // G
            rgba.push(chunk[0]); // B (was R)
            rgba.push(chunk[3]); // A
        }
    }

    let image = RgbaImage::from_raw(width, height, rgba)?;
    Some(RenderImage::new(smallvec![Frame::new(image)]))
}

/// View used for rendering
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
struct RenderView {
    element_cell: Rc<RefCell<Option<AnyElement>>>,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl RenderView {
    fn new(element_cell: Rc<RefCell<Option<AnyElement>>>) -> Self {
        Self { element_cell }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
impl gpui::Render for RenderView {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        // Take and render the element
        if let Some(element) = self.element_cell.borrow_mut().take() {
            element
        } else {
            div().size_full().into_any_element()
        }
    }
}

/// A GPUI element that displays a textured canvas item
///
/// Use this in your canvas render loop to display cached textures
pub struct TexturedItemElement {
    image: Arc<RenderImage>,
    bounds: Bounds<Pixels>,
}

impl TexturedItemElement {
    pub fn new(image: Arc<RenderImage>, bounds: Bounds<Pixels>) -> Self {
        Self { image, bounds }
    }
}

impl RenderOnce for TexturedItemElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .absolute()
            .left(self.bounds.origin.x)
            .top(self.bounds.origin.y)
            .w(self.bounds.size.width)
            .h(self.bounds.size.height)
            .child(img(self.image).size_full().object_fit(ObjectFit::Fill))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_add_item() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());

        assert_eq!(provider.items().len(), 1);
        assert!(matches!(
            provider.texture_state("test-1"),
            TextureState::NotRequested
        ));
    }

    #[test]
    fn test_provider_request_texture() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.request_texture("test-1");

        assert!(matches!(
            provider.texture_state("test-1"),
            TextureState::Queued
        ));
        assert_eq!(provider.pending_count(), 1);
    }

    #[test]
    fn test_provider_remove_item() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.remove_item("test-1");

        assert_eq!(provider.items().len(), 0);
    }

    #[test]
    fn test_provider_invalidate() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.request_texture("test-1");

        assert!(matches!(
            provider.texture_state("test-1"),
            TextureState::Queued
        ));

        provider.invalidate("test-1");

        assert!(matches!(
            provider.texture_state("test-1"),
            TextureState::NotRequested
        ));
    }

    #[test]
    fn test_provider_set_position() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.set_position("test-1", point(px(100.0), px(200.0)));

        let items = provider.items();
        assert_eq!(items[0].bounds.origin.x, px(100.0));
        assert_eq!(items[0].bounds.origin.y, px(200.0));
    }

    #[test]
    fn test_provider_tick_renders() {
        let mut provider = TexturedCanvasItemsProvider::new();

        provider.add_item("test-1", || div().into_any_element());
        provider.request_texture("test-1");

        // tick() should process the queue
        let work_done = provider.tick();
        assert!(work_done);

        // On Linux/FreeBSD with vendored GPUI, this might succeed
        // On other platforms or without vendored GPUI, it will fail
        let state = provider.texture_state("test-1");
        assert!(
            matches!(state, TextureState::Ready { .. }) || matches!(state, TextureState::Failed(_))
        );
    }
}
