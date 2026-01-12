//! Textured canvas items provider.
//!
//! This provider renders items as textures using GPUI's `TexturedView`,
//! allowing for smooth zooming and panning of pre-rendered content.
//!
//! # Platform Support
//!
//! Requires Linux/FreeBSD where `TexturedView` is available.
//! On other platforms, items will show placeholder content.

use gpui::{
    div, img, point, px, size, AnyElement, AnyView, App, AppContext as _, Bounds, Context,
    IntoElement, ObjectFit, ParentElement, Pixels, Point, RenderImage, Size, Styled, StyledImage,
    Window,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::provider::{CanvasItemsProvider, ItemDescriptor, ItemId};

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use gpui::TexturedView;

// Re-export ItemSizing from gpui for convenient API access
pub use gpui::ItemSizing;

// ============================================================================
// Types
// ============================================================================
/// Type alias for the texture getter closure.
type TextureGetter = Box<dyn Fn(&App) -> Option<Arc<RenderImage>> + Send + Sync>;

/// Type alias for the size getter closure (to query measured size from TexturedView).
type SizeGetter = Box<dyn Fn(&App) -> Option<Size<Pixels>> + Send + Sync>;

/// Internal storage for a canvas item.
struct CanvasItemEntry {
    /// Position on canvas (canvas space).
    origin: Point<Pixels>,
    /// Initial/estimated size of the item.
    size: Size<Pixels>,
    /// Z-index for ordering.
    z_index: i32,
    /// The view (TexturedView or placeholder).
    view: AnyView,
    /// Closure to get the texture (for zoom-scaled rendering).
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    texture_getter: TextureGetter,
    /// Closure to get the measured size from the TexturedView.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    size_getter: SizeGetter,
}

// ============================================================================
// TexturedCanvasItemsProvider
// ============================================================================

/// A canvas items provider that renders items as textures.
///
/// Each item is backed by a `TexturedView` which handles:
/// - Background thread rendering
/// - Async texture receiving with proper event loop wake
/// - Texture caching
///
/// # Example
///
/// ```ignore
/// use infinite_canvas::{InfiniteCanvas, TexturedCanvasItemsProvider, SharedProvider};
/// use gpui::*;
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// let provider = Rc::new(RefCell::new(TexturedCanvasItemsProvider::new()));
/// provider.borrow_mut().add_item("card-1", point(px(0.0), px(0.0)), window, cx, || {
///     div().p_4().bg(rgb(0x3498db)).child("Hello!")
/// });
///
/// let canvas = InfiniteCanvas::new("canvas", provider.clone());
/// ```
pub struct TexturedCanvasItemsProvider {
    /// Items by ID.
    items: HashMap<ItemId, CanvasItemEntry>,
    /// Default sizing for new items.
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
        let entity_for_texture = entity.clone();
        let texture_getter: TextureGetter =
            Box::new(move |cx: &App| entity_for_texture.read(cx).texture());

        // Create a closure to get the measured size from the entity
        let entity_for_size = entity.clone();
        let size_getter: SizeGetter =
            Box::new(move |cx: &App| entity_for_size.read(cx).measured_size());

        self.items.insert(
            id,
            CanvasItemEntry {
                origin,
                size: initial_size,
                z_index: 0,
                view: entity.into(),
                texture_getter,
                size_getter,
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
        let id = id.into();
        let initial_size = self.default_sizing.initial_size();

        let view = cx
            .new(|_| UnsupportedPlatformView { size: initial_size })
            .into();

        self.items.insert(
            id,
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

    /// Remove an item by ID.
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
    pub fn set_z_index(&mut self, id: &str, z_index: i32) {
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

    /// Check if an item exists.
    pub fn contains(&self, id: &str) -> bool {
        self.items.contains_key(id)
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Invalidate an item's texture (force re-render).
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

            let entity = cx.new(|cx| {
                TexturedView::with_options(sizing, gpui::RenderMode::Once, window, cx, render_fn)
            });

            // Update view, texture_getter, and size_getter
            let entity_for_texture = entity.clone();
            item.texture_getter = Box::new(move |cx: &App| entity_for_texture.read(cx).texture());
            let entity_for_size = entity.clone();
            item.size_getter = Box::new(move |cx: &App| entity_for_size.read(cx).measured_size());
            item.view = entity.into();
        }
    }

    /// Invalidate an item's texture (unsupported platform stub).
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
}

impl Default for TexturedCanvasItemsProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CanvasItemsProvider Implementation
// ============================================================================

impl CanvasItemsProvider for TexturedCanvasItemsProvider {
    fn items(&self) -> Vec<ItemDescriptor> {
        self.items
            .iter()
            .map(|(id, item)| ItemDescriptor {
                id: id.clone(),
                bounds: Bounds::new(item.origin, item.size),
                z_index: item.z_index,
            })
            .collect()
    }

    /// Get items with measured sizes (requires App context).
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn items_with_context(&self, cx: &App) -> Vec<ItemDescriptor> {
        self.items
            .iter()
            .map(|(id, item)| {
                let measured = (item.size_getter)(cx);
                let size = measured.unwrap_or(item.size);
                log::debug!(
                    "[TexturedProvider] Item '{}': initial={:?}, measured={:?}, using={:?}",
                    id,
                    item.size,
                    measured,
                    size
                );
                ItemDescriptor {
                    id: id.clone(),
                    bounds: Bounds::new(item.origin, size),
                    z_index: item.z_index,
                }
            })
            .collect()
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement> {
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
                    .border_2()
                    .border_color(gpui::rgb(0xff0000))
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

    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    fn render_item(
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

    fn item_count(&self) -> usize {
        self.items.len()
    }
}

// ============================================================================
// Unsupported Platform Placeholder
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new() {
        let provider = TexturedCanvasItemsProvider::new();
        assert_eq!(provider.item_count(), 0);
        assert!(provider.is_empty());
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
    fn test_set_position_nonexistent() {
        let mut provider = TexturedCanvasItemsProvider::new();
        provider.set_position("nonexistent", point(px(100.0), px(200.0)));
        assert!(!provider.contains("nonexistent"));
    }

    #[test]
    fn test_default() {
        let provider = TexturedCanvasItemsProvider::default();
        assert_eq!(provider.item_count(), 0);
    }
}
