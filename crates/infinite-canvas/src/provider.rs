//! Canvas items provider trait.
//!
//! This module defines the `CanvasItemsProvider` trait which abstracts
//! over different ways of providing items to an `InfiniteCanvas`.

use gpui::{AnyElement, App, Bounds, Pixels, Point};

/// Unique identifier for a canvas item.
pub type ItemId = String;

/// Describes a canvas item's position and bounds.
#[derive(Clone, Debug)]
pub struct ItemDescriptor {
    /// Unique identifier for this item.
    pub id: ItemId,
    /// Position and size on the canvas (in canvas space).
    pub bounds: Bounds<Pixels>,
    /// Z-index for rendering order (higher = on top).
    pub z_index: i32,
}

impl ItemDescriptor {
    /// Create a new item descriptor.
    pub fn new(id: impl Into<String>, bounds: Bounds<Pixels>) -> Self {
        Self {
            id: id.into(),
            bounds,
            z_index: 0,
        }
    }

    /// Create a new item descriptor with z-index.
    pub fn with_z_index(id: impl Into<String>, bounds: Bounds<Pixels>, z_index: i32) -> Self {
        Self {
            id: id.into(),
            bounds,
            z_index,
        }
    }

    /// Get the origin (top-left position) of this item.
    pub fn origin(&self) -> Point<Pixels> {
        self.bounds.origin
    }
}

/// Trait for providing items to an `InfiniteCanvas`.
///
/// Implementors of this trait provide a collection of items that can be
/// rendered on the canvas. The canvas handles camera transforms, culling,
/// and event handling, while the provider handles item storage and rendering.
///
/// # Built-in Providers
///
/// - `TexturedCanvasItemsProvider` - Items rendered as textures (zoomable)
/// - Future: `RenderedCanvasItemsProvider` - Items rendered directly
///
/// # Example
///
/// ```ignore
/// use infinite_canvas::{CanvasItemsProvider, ItemDescriptor};
///
/// struct MyProvider {
///     items: Vec<MyItem>,
/// }
///
/// impl CanvasItemsProvider for MyProvider {
///     fn items(&self) -> Vec<ItemDescriptor> {
///         self.items.iter().map(|item| ItemDescriptor::new(&item.id, item.bounds)).collect()
///     }
///
///     fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement> {
///         // Render item at the given screen bounds
///     }
/// }
/// ```
pub trait CanvasItemsProvider {
    /// Get descriptors for all items.
    ///
    /// Returns a list of item descriptors containing id, bounds (in canvas space),
    /// and z-index. The canvas uses this to determine what items are visible
    /// and how to transform them for rendering.
    ///
    /// Note: This returns initial/estimated sizes. For measured sizes (e.g., from
    /// TexturedView), use `items_with_context` instead.
    fn items(&self) -> Vec<ItemDescriptor>;

    /// Get descriptors for all items with access to App context.
    ///
    /// This allows providers to query measured sizes from their backing views.
    /// For example, `TexturedCanvasItemsProvider` uses this to return actual
    /// measured heights for `FixedWidth` sizing mode.
    ///
    /// The default implementation just calls `items()`.
    fn items_with_context(&self, _cx: &App) -> Vec<ItemDescriptor> {
        self.items()
    }

    /// Render an item at the given screen bounds.
    ///
    /// Called by the canvas for each visible item after applying camera transforms.
    /// The `screen_bounds` are in screen space (already transformed by the camera).
    ///
    /// # Arguments
    ///
    /// * `id` - The item's unique identifier
    /// * `screen_bounds` - The bounds to render at (in screen space, after camera transform)
    /// * `cx` - The GPUI app context
    ///
    /// # Returns
    ///
    /// An element to render, or `None` if the item cannot be rendered.
    fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement>;

    /// Get the number of items.
    fn item_count(&self) -> usize {
        self.items().len()
    }

    /// Check if the provider has any items.
    fn is_empty(&self) -> bool {
        self.item_count() == 0
    }

    /// Get the bounding box of all items (in canvas space).
    ///
    /// Returns `None` if there are no items.
    fn content_bounds(&self) -> Option<Bounds<Pixels>> {
        let items = self.items();
        if items.is_empty() {
            return None;
        }
        let mut bounds = items.first().unwrap().bounds;

        for item in &items {
            bounds = bounds.union(&item.bounds);
        }

        Some(bounds)
    }
}

// Implement for references to providers
impl<T: CanvasItemsProvider + ?Sized> CanvasItemsProvider for &T {
    fn items(&self) -> Vec<ItemDescriptor> {
        (*self).items()
    }

    fn items_with_context(&self, cx: &App) -> Vec<ItemDescriptor> {
        (*self).items_with_context(cx)
    }

    fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement> {
        (*self).render_item(id, screen_bounds, cx)
    }

    fn item_count(&self) -> usize {
        (*self).item_count()
    }

    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }

    fn content_bounds(&self) -> Option<Bounds<Pixels>> {
        (*self).content_bounds()
    }
}
