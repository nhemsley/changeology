//! TexturedCanvasItemsProvider2 - Async texture rendering using GPUI's TexturedView
//!
//! This is a redesigned canvas items provider that leverages GPUI's `TexturedView`
//! for each item, rather than managing background threads and channels manually.
//!
//! # Architecture
//!
//! Unlike the original `TexturedCanvasItemsProvider` which used:
//! - Manual thread spawning per item
//! - Synchronous `tick()` polling with `try_recv()`
//! - Timer-based refresh workarounds
//!
//! This provider uses:
//! - GPUI's `TexturedView` for each item (handles async rendering internally)
//! - Proper wake mechanism via `recv_async().await`
//! - No polling, no timers - UI updates automatically when textures are ready
//!
//! # Design
//!
//! Since `TexturedView<F>` is generic over the render function type, we can't store
//! heterogeneous views directly. Instead, we store `AnyView` (type-erased view handle)
//! which allows us to render any TexturedView regardless of its concrete type.
//!
//! # Example
//!
//! ```ignore
//! use infinite_canvas::textured_provider2::TexturedCanvasProvider;
//! use gpui::*;
//!
//! struct MyCanvasView {
//!     provider: TexturedCanvasItemsProvider,
//!     camera: Camera,
//! }
//!
//! impl MyCanvasView {
//!     fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
//!         let mut provider = TexturedCanvasItemsProvider::new();
//!
//!         // Add items - each gets its own TexturedView
//!         provider.add_item("card-1", point(px(0.0), px(0.0)), window, cx, || {
//!             div().p_4().bg(rgb(0x3498db)).child("Hello!")
//!         });
//!
//!         Self { provider, camera: Camera::default() }
//!     }
//! }
//! ```
//!
//! # Platform Support
//!
//! Requires Linux/FreeBSD where `Application::textured()` is available.
//! On other platforms, items will show error placeholders.

use gpui::{
    div, img, point, px, size, AnyElement, AnyView, App, AppContext as _, Bounds, Context,
    IntoElement, ObjectFit, ParentElement, Pixels, Point, RenderImage, RenderOnce, Size, Styled,
    StyledImage, Window,
};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use gpui::TexturedView;

// Re-export ItemSizing from gpui
pub use gpui::ItemSizing;

// ============================================================================
// Types
// ============================================================================

/// Unique identifier for a canvas item.
pub type CanvasItemId = String;

/// Describes a canvas item's position and bounds for rendering.
#[derive(Clone, Debug)]
pub struct CanvasItemDescriptor {
    /// Unique identifier
    pub id: CanvasItemId,
    /// Position and size on the canvas
    pub bounds: Bounds<Pixels>,
    /// Z-index for rendering order
    pub z_index: u32,
}

/// State of an item's texture.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureState {
    /// Texture is being rendered in background
    Rendering,
    /// Texture is ready to display
    Ready,
    /// Rendering failed
    Failed(String),
    /// Platform not supported
    Unsupported,
}

// ============================================================================
// Internal Item Storage
// ============================================================================

/// Type alias for the texture getter closure
type TextureGetter = Box<dyn Fn(&App) -> Option<Arc<RenderImage>> + Send + Sync>;

/// Internal storage for a canvas item.
struct CanvasItemEntry {
    /// Position on canvas
    origin: Point<Pixels>,
    /// Size (may be measured)
    size: Size<Pixels>,
    /// Z-index for ordering
    z_index: u32,
    /// The TexturedView as a type-erased AnyView (for default rendering)
    view: AnyView,
    /// Closure to get the texture from the TexturedView entity (for zoom support)
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    texture_getter: TextureGetter,
}

// ============================================================================
// TexturedCanvasItemsProvider
// ============================================================================

/// A canvas items provider that uses GPUI's TexturedView for async rendering.
///
/// Each item is backed by a `TexturedView` entity which handles:
/// - Background thread rendering via `Application::textured()`
/// - Async frame receiving with proper event loop wake
/// - Texture caching
///
/// This provider just tracks positions and provides a unified API for the canvas.
pub struct TexturedCanvasItemsProvider {
    /// Items by ID
    items: HashMap<CanvasItemId, CanvasItemEntry>,
    /// Default sizing for new items
    default_sizing: ItemSizing,
}

impl TexturedCanvasItemsProvider {
    /// Create a new provider with default sizing.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            default_sizing: ItemSizing::Fixed {
                size: size(px(300.0), px(200.0)),
            },
        }
    }

    /// Create a new provider with specific default sizing.
    pub fn with_sizing(sizing: ItemSizing) -> Self {
        Self {
            items: HashMap::new(),
            default_sizing: sizing,
        }
    }

    /// Set the default sizing for new items.
    pub fn set_default_sizing(&mut self, sizing: ItemSizing) {
        self.default_sizing = sizing;
    }

    /// Get the default sizing.
    pub fn default_sizing(&self) -> &ItemSizing {
        &self.default_sizing
    }

    /// Add an item at a specific position.
    ///
    /// The `render_fn` creates the GPUI element to render as a texture.
    /// Rendering happens asynchronously in a background thread.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn add_item<V: 'static, F, E>(
        &mut self,
        id: impl Into<String>,
        origin: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<V>,
        render_fn: F,
    ) where
        F: Fn() -> E + Send + Clone + 'static,
        E: IntoElement + 'static,
    {
        let id = id.into();
        let sizing = self.default_sizing.clone();
        let initial_size = sizing.initial_size();

        // Create TexturedView for this item
        let entity = cx.new(|cx| {
            TexturedView::with_options(sizing, gpui::RenderMode::Once, window, cx, render_fn)
        });

        // Create a closure to get the texture from this entity
        let entity_clone = entity.clone();
        let texture_getter: TextureGetter =
            Box::new(move |cx: &App| entity_clone.read(cx).texture());

        self.items.insert(
            id,
            CanvasItemEntry {
                origin,
                size: initial_size,
                z_index: 0,
                view: entity.into(),
                texture_getter,
            },
        );
    }

    /// Add an item at a specific position (unsupported platform stub).
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    pub fn add_item<V: 'static, F, E>(
        &mut self,
        id: impl Into<String>,
        origin: Point<Pixels>,
        _window: &mut Window,
        cx: &mut Context<V>,
        _render_fn: F,
    ) where
        F: Fn() -> E + Send + Clone + 'static,
        E: IntoElement + 'static,
    {
        let id_str = id.into();
        let initial_size = self.default_sizing.initial_size();

        // Create a placeholder view for unsupported platforms
        let view = cx
            .new(|_| UnsupportedPlatformView { size: initial_size })
            .into();

        self.items.insert(
            id_str,
            CanvasItemEntry {
                origin,
                size: initial_size,
                z_index: 0,
                view,
            },
        );
    }

    /// Add an item at the origin (0, 0).
    pub fn add_item_at_origin<V: 'static, F, E>(
        &mut self,
        id: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<V>,
        render_fn: F,
    ) where
        F: Fn() -> E + Send + Clone + 'static,
        E: IntoElement + 'static,
    {
        self.add_item(id, point(px(0.0), px(0.0)), window, cx, render_fn);
    }

    /// Remove an item.
    pub fn remove_item(&mut self, id: &str) -> bool {
        self.items.remove(id).is_some()
    }

    /// Set an item's position.
    pub fn set_position(&mut self, id: &str, origin: Point<Pixels>) {
        if let Some(item) = self.items.get_mut(id) {
            item.origin = origin;
        }
    }

    /// Set an item's z-index.
    pub fn set_z_index(&mut self, id: &str, z_index: u32) {
        if let Some(item) = self.items.get_mut(id) {
            item.z_index = z_index;
        }
    }

    /// Get an item's bounds.
    pub fn bounds(&self, id: &str) -> Option<Bounds<Pixels>> {
        self.items
            .get(id)
            .map(|item| Bounds::new(item.origin, item.size))
    }

    /// Get all item IDs.
    pub fn item_ids(&self) -> Vec<&str> {
        self.items.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of items.
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if an item exists.
    pub fn contains(&self, id: &str) -> bool {
        self.items.contains_key(id)
    }

    /// Get all item descriptors (for rendering).
    pub fn items(&self) -> Vec<CanvasItemDescriptor> {
        self.items
            .iter()
            .map(|(id, item)| CanvasItemDescriptor {
                id: id.clone(),
                bounds: Bounds::new(item.origin, item.size),
                z_index: item.z_index,
            })
            .collect()
    }

    /// Get the AnyView for an item.
    ///
    /// Use this to render the item's texture in your view.
    pub fn get_view(&self, id: &str) -> Option<&AnyView> {
        self.items.get(id).map(|item| &item.view)
    }

    /// Render an item as an element.
    ///
    /// This returns an element that can be added as a child.
    /// Position it using absolute positioning in the parent.
    pub fn render_item(&self, id: &str) -> Option<AnyElement> {
        self.items.get(id).map(|item| {
            div()
                .size_full()
                .child(item.view.clone())
                .into_any_element()
        })
    }

    /// Render an item at its canvas position (for use in canvas rendering).
    ///
    /// Returns an absolutely positioned element at the item's origin.
    pub fn render_item_positioned(&self, id: &str) -> Option<AnyElement> {
        self.items.get(id).map(|item| {
            div()
                .absolute()
                .left(item.origin.x)
                .top(item.origin.y)
                .w(item.size.width)
                .h(item.size.height)
                .child(item.view.clone())
                .into_any_element()
        })
    }

    /// Render an item at a transformed screen position with proper zoom scaling.
    ///
    /// Use this when applying camera transforms. The texture will be scaled
    /// to fit the screen bounds using `ObjectFit::Fill`.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn render_item_at(
        &self,
        id: &str,
        screen_bounds: Bounds<Pixels>,
        cx: &App,
    ) -> Option<AnyElement> {
        self.items.get(id).map(|item| {
            // Try to get the texture for proper scaling
            if let Some(texture) = (item.texture_getter)(cx) {
                // Render with proper scaling using object_fit
                div()
                    .absolute()
                    .left(screen_bounds.origin.x)
                    .top(screen_bounds.origin.y)
                    .w(screen_bounds.size.width)
                    .h(screen_bounds.size.height)
                    .child(img(texture).size_full().object_fit(ObjectFit::Fill))
                    .into_any_element()
            } else {
                // Texture not ready yet, show the view (which has loading placeholder)
                div()
                    .absolute()
                    .left(screen_bounds.origin.x)
                    .top(screen_bounds.origin.y)
                    .w(screen_bounds.size.width)
                    .h(screen_bounds.size.height)
                    .overflow_hidden()
                    .child(item.view.clone())
                    .into_any_element()
            }
        })
    }

    /// Render an item at a transformed screen position (unsupported platform).
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    pub fn render_item_at(
        &self,
        id: &str,
        screen_bounds: Bounds<Pixels>,
        _cx: &App,
    ) -> Option<AnyElement> {
        self.items.get(id).map(|item| {
            div()
                .absolute()
                .left(screen_bounds.origin.x)
                .top(screen_bounds.origin.y)
                .w(screen_bounds.size.width)
                .h(screen_bounds.size.height)
                .overflow_hidden()
                .child(item.view.clone())
                .into_any_element()
        })
    }

    /// Invalidate an item's texture (force re-render).
    ///
    /// This creates a new TexturedView for the item.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    pub fn invalidate<V: 'static, F, E>(
        &mut self,
        id: &str,
        window: &mut Window,
        cx: &mut Context<V>,
        render_fn: F,
    ) where
        F: Fn() -> E + Send + Clone + 'static,
        E: IntoElement + 'static,
    {
        if let Some(item) = self.items.get_mut(id) {
            let sizing = self.default_sizing.clone();

            let view = cx
                .new(|cx| {
                    TexturedView::with_options(
                        sizing,
                        gpui::RenderMode::Once,
                        window,
                        cx,
                        render_fn,
                    )
                })
                .into();

            item.view = view;
        }
    }

    /// Invalidate (unsupported platform stub).
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    pub fn invalidate<V: 'static, F, E>(
        &mut self,
        _id: &str,
        _window: &mut Window,
        _cx: &mut Context<V>,
        _render_fn: F,
    ) where
        F: Fn() -> E + Send + Clone + 'static,
        E: IntoElement + 'static,
    {
        // No-op on unsupported platforms
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for TexturedCanvasItemsProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Unsupported Platform Placeholder View
// ============================================================================

/// A placeholder view shown on unsupported platforms.
#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
struct UnsupportedPlatformView {
    size: Size<Pixels>,
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
impl gpui::Render for UnsupportedPlatformView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(self.size.width)
            .h(self.size.height)
            .bg(gpui::rgb(0xffcccc))
            .flex()
            .justify_center()
            .items_center()
            .text_color(gpui::rgb(0xcc0000))
            .child("Unsupported platform")
    }
}

// ============================================================================
// CanvasItemElement - Element wrapper for positioned items
// ============================================================================

/// An element that renders a canvas item at a specific screen position.
///
/// Use this when rendering items in a canvas view with camera transforms applied.
pub struct CanvasItemElement {
    /// Screen bounds (after camera transform)
    screen_bounds: Bounds<Pixels>,
    /// The item's content element
    content: AnyElement,
}

impl CanvasItemElement {
    /// Create a new canvas item element.
    pub fn new(screen_bounds: Bounds<Pixels>, content: AnyElement) -> Self {
        Self {
            screen_bounds,
            content,
        }
    }
}

impl RenderOnce for CanvasItemElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .absolute()
            .left(self.screen_bounds.origin.x)
            .top(self.screen_bounds.origin.y)
            .w(self.screen_bounds.size.width)
            .h(self.screen_bounds.size.height)
            .child(self.content)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new() {
        let provider = TexturedCanvasItemsProvider::new();
        assert_eq!(provider.item_count(), 0);
    }

    #[test]
    fn test_provider_with_sizing() {
        let provider = TexturedCanvasItemsProvider::with_sizing(ItemSizing::Fixed {
            size: size(px(400.0), px(300.0)),
        });
        match provider.default_sizing() {
            ItemSizing::Fixed { size } => {
                assert_eq!(size.width, px(400.0));
                assert_eq!(size.height, px(300.0));
            }
            _ => panic!("Expected Fixed sizing"),
        }
    }

    #[test]
    fn test_set_position() {
        let mut provider = TexturedCanvasItemsProvider::new();
        // Note: Can't fully test add_item without GPUI context
        // Just test the position setting logic exists
        provider.set_position("nonexistent", point(px(100.0), px(200.0)));
        assert!(!provider.contains("nonexistent"));
    }

    #[test]
    fn test_item_ids() {
        let provider = TexturedCanvasItemsProvider::new();
        assert!(provider.item_ids().is_empty());
    }

    #[test]
    fn test_default() {
        let provider = TexturedCanvasItemsProvider::default();
        assert_eq!(provider.item_count(), 0);
    }
}
