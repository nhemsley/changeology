//! Canvas items that can be placed on the infinite canvas.
//!
//! This module provides the `CanvasItem` type which represents rectangular
//! objects positioned on the canvas, along with the `ItemId` identifier type.

use gpui::{Bounds, Pixels, Point, Size};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A unique identifier for a canvas item.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(Arc<str>);

impl ItemId {
    /// Create a new item ID from a string.
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    /// Get the string representation of this ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ItemId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ItemId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<Arc<str>> for ItemId {
    fn from(s: Arc<str>) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A rectangular item on the infinite canvas.
///
/// Canvas items have a unique identifier, bounds (position and size),
/// and optional associated data.
///
/// # Type Parameters
///
/// * `D` - The type of data associated with this item. Use `()` if no data is needed.
#[derive(Clone, Debug)]
pub struct CanvasItem<D = ()> {
    /// Unique identifier for this item.
    pub id: ItemId,

    /// The bounds of this item in canvas space.
    pub bounds: Bounds<Pixels>,

    /// User-defined data associated with this item.
    pub data: D,

    /// Whether this item is selected.
    pub selected: bool,

    /// Whether this item is visible.
    pub visible: bool,

    /// Whether this item is locked (cannot be moved/resized).
    pub locked: bool,

    /// Z-index for rendering order (higher = on top).
    pub z_index: i32,
}

impl CanvasItem<()> {
    /// Create a new canvas item with no associated data.
    pub fn new(id: impl Into<ItemId>, bounds: Bounds<Pixels>) -> Self {
        Self {
            id: id.into(),
            bounds,
            data: (),
            selected: false,
            visible: true,
            locked: false,
            z_index: 0,
        }
    }
}

impl<D> CanvasItem<D> {
    /// Create a new canvas item with associated data.
    pub fn with_data(id: impl Into<ItemId>, bounds: Bounds<Pixels>, data: D) -> Self {
        Self {
            id: id.into(),
            bounds,
            data,
            selected: false,
            visible: true,
            locked: false,
            z_index: 0,
        }
    }

    /// Create a canvas item from position and size.
    pub fn from_position_size(
        id: impl Into<ItemId>,
        position: Point<Pixels>,
        size: Size<Pixels>,
        data: D,
    ) -> Self {
        Self::with_data(id, Bounds::new(position, size), data)
    }

    /// Get the position (origin) of this item.
    pub fn position(&self) -> Point<Pixels> {
        self.bounds.origin
    }

    /// Get the size of this item.
    pub fn size(&self) -> Size<Pixels> {
        self.bounds.size
    }

    /// Get the center point of this item.
    pub fn center(&self) -> Point<Pixels> {
        Point::new(
            self.bounds.origin.x + self.bounds.size.width / 2.0,
            self.bounds.origin.y + self.bounds.size.height / 2.0,
        )
    }

    /// Set the position of this item.
    pub fn set_position(&mut self, position: Point<Pixels>) {
        self.bounds.origin = position;
    }

    /// Set the size of this item.
    pub fn set_size(&mut self, size: Size<Pixels>) {
        self.bounds.size = size;
    }

    /// Move the item by a delta.
    pub fn translate(&mut self, delta: Point<Pixels>) {
        self.bounds.origin.x += delta.x;
        self.bounds.origin.y += delta.y;
    }

    /// Check if this item contains a point (in canvas space).
    pub fn contains(&self, point: Point<Pixels>) -> bool {
        self.bounds.contains(&point)
    }

    /// Check if this item intersects with another bounds.
    pub fn intersects(&self, other: &Bounds<Pixels>) -> bool {
        self.bounds.intersects(other)
    }

    /// Set the selected state.
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the visible state.
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the locked state.
    pub fn with_locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }

    /// Set the z-index.
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Map the data to a new type.
    pub fn map_data<D2>(self, f: impl FnOnce(D) -> D2) -> CanvasItem<D2> {
        CanvasItem {
            id: self.id,
            bounds: self.bounds,
            data: f(self.data),
            selected: self.selected,
            visible: self.visible,
            locked: self.locked,
            z_index: self.z_index,
        }
    }
}

impl<D: Default> CanvasItem<D> {
    /// Create a new canvas item with default data.
    pub fn with_default_data(id: impl Into<ItemId>, bounds: Bounds<Pixels>) -> Self {
        Self::with_data(id, bounds, D::default())
    }
}

/// A collection of canvas items with helper methods.
#[derive(Clone, Debug, Default)]
pub struct CanvasItems<D = ()> {
    items: Vec<CanvasItem<D>>,
}

impl<D> CanvasItems<D> {
    /// Create a new empty collection.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Create a collection from a vector of items.
    pub fn from_vec(items: Vec<CanvasItem<D>>) -> Self {
        Self { items }
    }

    /// Add an item to the collection.
    pub fn push(&mut self, item: CanvasItem<D>) {
        self.items.push(item);
    }

    /// Remove an item by ID.
    pub fn remove(&mut self, id: &ItemId) -> Option<CanvasItem<D>> {
        if let Some(index) = self.items.iter().position(|item| &item.id == id) {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Get an item by ID.
    pub fn get(&self, id: &ItemId) -> Option<&CanvasItem<D>> {
        self.items.iter().find(|item| &item.id == id)
    }

    /// Get a mutable reference to an item by ID.
    pub fn get_mut(&mut self, id: &ItemId) -> Option<&mut CanvasItem<D>> {
        self.items.iter_mut().find(|item| &item.id == id)
    }

    /// Get all items.
    pub fn all(&self) -> &[CanvasItem<D>] {
        &self.items
    }

    /// Get all items mutably.
    pub fn all_mut(&mut self) -> &mut [CanvasItem<D>] {
        &mut self.items
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over items.
    pub fn iter(&self) -> impl Iterator<Item = &CanvasItem<D>> {
        self.items.iter()
    }

    /// Iterate over items mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut CanvasItem<D>> {
        self.items.iter_mut()
    }

    /// Get items that intersect with the given bounds.
    pub fn intersecting<'a>(
        &'a self,
        bounds: &'a Bounds<Pixels>,
    ) -> impl Iterator<Item = &'a CanvasItem<D>> {
        self.items.iter().filter(|item| item.intersects(bounds))
    }

    /// Get visible items.
    pub fn visible(&self) -> impl Iterator<Item = &CanvasItem<D>> {
        self.items.iter().filter(|item| item.visible)
    }

    /// Get selected items.
    pub fn selected(&self) -> impl Iterator<Item = &CanvasItem<D>> {
        self.items.iter().filter(|item| item.selected)
    }

    /// Get the bounding box of all items.
    pub fn bounds(&self) -> Option<Bounds<Pixels>> {
        if self.items.is_empty() {
            return None;
        }

        let mut min_x = Pixels::MAX;
        let mut min_y = Pixels::MAX;
        let mut max_x = Pixels::MIN;
        let mut max_y = Pixels::MIN;

        for item in &self.items {
            min_x = min_x.min(item.bounds.origin.x);
            min_y = min_y.min(item.bounds.origin.y);
            max_x = max_x.max(item.bounds.origin.x + item.bounds.size.width);
            max_y = max_y.max(item.bounds.origin.y + item.bounds.size.height);
        }

        Some(Bounds::new(
            Point::new(min_x, min_y),
            Size::new(max_x - min_x, max_y - min_y),
        ))
    }

    /// Find the item at a given point (topmost by z-index).
    pub fn item_at(&self, point: Point<Pixels>) -> Option<&CanvasItem<D>> {
        self.items
            .iter()
            .filter(|item| item.visible && item.contains(point))
            .max_by_key(|item| item.z_index)
    }

    /// Sort items by z-index (for rendering order).
    pub fn sort_by_z_index(&mut self) {
        self.items.sort_by_key(|item| item.z_index);
    }

    /// Clear all items.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl<D> IntoIterator for CanvasItems<D> {
    type Item = CanvasItem<D>;
    type IntoIter = std::vec::IntoIter<CanvasItem<D>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, D> IntoIterator for &'a CanvasItems<D> {
    type Item = &'a CanvasItem<D>;
    type IntoIter = std::slice::Iter<'a, CanvasItem<D>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<D> FromIterator<CanvasItem<D>> for CanvasItems<D> {
    fn from_iter<I: IntoIterator<Item = CanvasItem<D>>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, px, size};

    fn make_bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }

    #[test]
    fn test_item_creation() {
        let item = CanvasItem::new("test-item", make_bounds(10.0, 20.0, 100.0, 50.0));

        assert_eq!(item.id.as_str(), "test-item");
        assert_eq!(item.position().x, px(10.0));
        assert_eq!(item.position().y, px(20.0));
        assert_eq!(item.size().width, px(100.0));
        assert_eq!(item.size().height, px(50.0));
    }

    #[test]
    fn test_item_center() {
        let item = CanvasItem::new("test", make_bounds(0.0, 0.0, 100.0, 100.0));
        let center = item.center();

        assert_eq!(center.x, px(50.0));
        assert_eq!(center.y, px(50.0));
    }

    #[test]
    fn test_item_contains() {
        let item = CanvasItem::new("test", make_bounds(10.0, 10.0, 100.0, 100.0));

        assert!(item.contains(point(px(50.0), px(50.0))));
        assert!(!item.contains(point(px(5.0), px(5.0))));
        assert!(!item.contains(point(px(200.0), px(200.0))));
    }

    #[test]
    fn test_item_translate() {
        let mut item = CanvasItem::new("test", make_bounds(10.0, 20.0, 100.0, 50.0));
        item.translate(point(px(5.0), px(-10.0)));

        assert_eq!(item.position().x, px(15.0));
        assert_eq!(item.position().y, px(10.0));
    }

    #[test]
    fn test_items_collection() {
        let mut items = CanvasItems::new();
        items.push(CanvasItem::new("a", make_bounds(0.0, 0.0, 50.0, 50.0)));
        items.push(CanvasItem::new("b", make_bounds(100.0, 0.0, 50.0, 50.0)));

        assert_eq!(items.len(), 2);
        assert!(items.get(&ItemId::new("a")).is_some());
        assert!(items.get(&ItemId::new("c")).is_none());
    }

    #[test]
    fn test_items_bounds() {
        let mut items = CanvasItems::new();
        items.push(CanvasItem::new("a", make_bounds(10.0, 20.0, 50.0, 50.0)));
        items.push(CanvasItem::new("b", make_bounds(100.0, 30.0, 60.0, 40.0)));

        let bounds = items.bounds().unwrap();
        assert_eq!(bounds.origin.x, px(10.0));
        assert_eq!(bounds.origin.y, px(20.0));
        assert_eq!(bounds.size.width, px(150.0)); // 160 - 10
        assert_eq!(bounds.size.height, px(50.0)); // 70 - 20
    }

    #[test]
    fn test_item_at_respects_z_index() {
        let mut items = CanvasItems::new();
        items.push(CanvasItem::new("bottom", make_bounds(0.0, 0.0, 100.0, 100.0)).with_z_index(0));
        items.push(CanvasItem::new("top", make_bounds(0.0, 0.0, 100.0, 100.0)).with_z_index(1));

        let hit = items.item_at(point(px(50.0), px(50.0))).unwrap();
        assert_eq!(hit.id.as_str(), "top");
    }
}
