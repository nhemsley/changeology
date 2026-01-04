//! Layout algorithms for arranging canvas items.
//!
//! This module provides various layout algorithms for automatically
//! positioning canvas items on the infinite canvas.
//!
//! # Available Layouts
//!
//! - **Grid Layout**: Arrange items in a regular grid
//! - **Tree Layout**: Hierarchical tree arrangement (top-down, left-to-right, radial)
//! - **Pack Layout**: Rectangle packing for variable-sized items
//! - **Force Layout**: Physics-based force-directed layout (future)

use gpui::{point, px, size, Bounds, Pixels, Point, Size};

use crate::item::CanvasItem;

/// A trait for layout algorithms.
///
/// Layout algorithms take a slice of canvas items and modify their
/// positions according to the layout strategy.
pub trait Layout {
    /// Apply the layout to the given items.
    ///
    /// This modifies the `bounds.origin` of each item in place.
    fn apply<D: Clone>(&self, items: &mut [CanvasItem<D>]);

    /// Calculate the total bounds that would be occupied by the layout.
    fn calculate_bounds<D: Clone>(&self, items: &[CanvasItem<D>]) -> Option<Bounds<Pixels>>;
}

// ============================================================================
// Grid Layout
// ============================================================================

/// Grid layout arranges items in a regular grid pattern.
///
/// Items are placed left-to-right, top-to-bottom in rows.
///
/// # Example
///
/// ```
/// use infinite_canvas::layout::{GridLayout, Layout};
/// use infinite_canvas::CanvasItem;
/// use gpui::{px, point, size, Bounds};
///
/// let layout = GridLayout::new()
///     .columns(3)
///     .cell_size(size(px(100.), px(80.)))
///     .gap(px(10.));
///
/// let mut items = vec![
///     CanvasItem::new("a", Bounds::new(point(px(0.), px(0.)), size(px(50.), px(50.)))),
///     CanvasItem::new("b", Bounds::new(point(px(0.), px(0.)), size(px(50.), px(50.)))),
///     CanvasItem::new("c", Bounds::new(point(px(0.), px(0.)), size(px(50.), px(50.)))),
/// ];
///
/// layout.apply(&mut items);
/// ```
#[derive(Clone, Debug)]
pub struct GridLayout {
    /// Number of columns in the grid.
    pub columns: usize,

    /// Size of each cell in the grid.
    pub cell_size: Size<Pixels>,

    /// Gap between cells.
    pub gap: Pixels,

    /// Starting position for the grid.
    pub origin: Point<Pixels>,

    /// Whether to center items within their cells.
    pub center_items: bool,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            columns: 4,
            cell_size: size(px(100.), px(100.)),
            gap: px(10.),
            origin: Point::default(),
            center_items: true,
        }
    }
}

impl GridLayout {
    /// Create a new grid layout with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of columns.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Set the cell size.
    pub fn cell_size(mut self, size: Size<Pixels>) -> Self {
        self.cell_size = size;
        self
    }

    /// Set the gap between cells.
    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = gap;
        self
    }

    /// Set the starting origin.
    pub fn origin(mut self, origin: Point<Pixels>) -> Self {
        self.origin = origin;
        self
    }

    /// Set whether to center items within their cells.
    pub fn center_items(mut self, center: bool) -> Self {
        self.center_items = center;
        self
    }

    /// Calculate the position for an item at a given index.
    fn position_for_index(&self, index: usize) -> Point<Pixels> {
        let row = index / self.columns;
        let col = index % self.columns;

        let x = self.origin.x + (self.cell_size.width + self.gap) * col as f32;
        let y = self.origin.y + (self.cell_size.height + self.gap) * row as f32;

        point(x, y)
    }
}

impl Layout for GridLayout {
    fn apply<D: Clone>(&self, items: &mut [CanvasItem<D>]) {
        for (index, item) in items.iter_mut().enumerate() {
            let cell_origin = self.position_for_index(index);

            if self.center_items {
                // Center the item within its cell
                let offset_x = (self.cell_size.width - item.size().width) / 2.0;
                let offset_y = (self.cell_size.height - item.size().height) / 2.0;
                item.set_position(point(
                    cell_origin.x + offset_x.max(px(0.)),
                    cell_origin.y + offset_y.max(px(0.)),
                ));
            } else {
                item.set_position(cell_origin);
            }
        }
    }

    fn calculate_bounds<D: Clone>(&self, items: &[CanvasItem<D>]) -> Option<Bounds<Pixels>> {
        if items.is_empty() {
            return None;
        }

        let num_items = items.len();
        let rows = (num_items + self.columns - 1) / self.columns;
        let cols = num_items.min(self.columns);

        let width = self.cell_size.width * cols as f32 + self.gap * (cols.saturating_sub(1)) as f32;
        let height =
            self.cell_size.height * rows as f32 + self.gap * (rows.saturating_sub(1)) as f32;

        Some(Bounds::new(self.origin, size(width, height)))
    }
}

// ============================================================================
// Tree Layout
// ============================================================================

/// The style of tree layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeLayoutStyle {
    /// Top-to-bottom tree (root at top).
    #[default]
    TopDown,

    /// Left-to-right tree (root at left).
    LeftToRight,

    /// Bottom-to-top tree (root at bottom).
    BottomUp,

    /// Right-to-left tree (root at right).
    RightToLeft,
}

/// A node in a tree structure for layout purposes.
#[derive(Clone, Debug)]
pub struct TreeNode<D = ()> {
    /// The canvas item for this node.
    pub item: CanvasItem<D>,

    /// Children of this node.
    pub children: Vec<TreeNode<D>>,
}

impl<D> TreeNode<D> {
    /// Create a new tree node with no children.
    pub fn new(item: CanvasItem<D>) -> Self {
        Self {
            item,
            children: Vec::new(),
        }
    }

    /// Create a tree node with children.
    pub fn with_children(item: CanvasItem<D>, children: Vec<TreeNode<D>>) -> Self {
        Self { item, children }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: TreeNode<D>) {
        self.children.push(child);
    }

    /// Check if this is a leaf node.
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Count total nodes in the tree.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }

    /// Get the depth of the tree.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Collect all items from the tree into a flat vector.
    pub fn flatten(&self) -> Vec<&CanvasItem<D>> {
        let mut items = vec![&self.item];
        for child in &self.children {
            items.extend(child.flatten());
        }
        items
    }

    /// Collect all mutable items from the tree.
    pub fn flatten_mut(&mut self) -> Vec<&mut CanvasItem<D>> {
        let mut items = vec![&mut self.item];
        for child in &mut self.children {
            items.extend(child.flatten_mut());
        }
        items
    }
}

/// Tree layout arranges items in a hierarchical tree structure.
///
/// Supports different orientations (top-down, left-to-right, etc.).
#[derive(Clone, Debug)]
pub struct TreeLayout {
    /// The style of the tree layout.
    pub style: TreeLayoutStyle,

    /// Horizontal spacing between sibling nodes.
    pub node_spacing: Pixels,

    /// Vertical spacing between levels.
    pub level_spacing: Pixels,

    /// Starting position for the tree.
    pub origin: Point<Pixels>,
}

impl Default for TreeLayout {
    fn default() -> Self {
        Self {
            style: TreeLayoutStyle::TopDown,
            node_spacing: px(20.),
            level_spacing: px(60.),
            origin: Point::default(),
        }
    }
}

impl TreeLayout {
    /// Create a new tree layout with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the layout style.
    pub fn style(mut self, style: TreeLayoutStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the spacing between sibling nodes.
    pub fn node_spacing(mut self, spacing: Pixels) -> Self {
        self.node_spacing = spacing;
        self
    }

    /// Set the spacing between levels.
    pub fn level_spacing(mut self, spacing: Pixels) -> Self {
        self.level_spacing = spacing;
        self
    }

    /// Set the starting origin.
    pub fn origin(mut self, origin: Point<Pixels>) -> Self {
        self.origin = origin;
        self
    }

    /// Apply the layout to a tree structure.
    pub fn apply_tree<D>(&self, root: &mut TreeNode<D>) {
        match self.style {
            TreeLayoutStyle::TopDown => {
                self.layout_top_down(root, 0);
            }
            TreeLayoutStyle::LeftToRight => {
                self.layout_left_to_right(root, 0);
            }
            TreeLayoutStyle::BottomUp => {
                // First layout top-down, then flip
                self.layout_top_down(root, 0);
                let total_height = self.calculate_tree_height(root);
                self.flip_vertical(root, total_height);
            }
            TreeLayoutStyle::RightToLeft => {
                // First layout left-to-right, then flip
                self.layout_left_to_right(root, 0);
                let total_width = self.calculate_tree_width(root);
                self.flip_horizontal(root, total_width);
            }
        }
    }

    /// Calculate the width needed for a subtree (for top-down layout).
    fn subtree_width<D>(&self, node: &TreeNode<D>) -> Pixels {
        if node.children.is_empty() {
            node.item.size().width
        } else {
            let children_width: Pixels = node
                .children
                .iter()
                .map(|c| self.subtree_width(c))
                .fold(px(0.), |acc, w| acc + w);
            let gaps = self.node_spacing * (node.children.len().saturating_sub(1)) as f32;
            let total_children_width = children_width + gaps;
            total_children_width.max(node.item.size().width)
        }
    }

    /// Layout the tree top-down.
    fn layout_top_down<D>(&self, node: &mut TreeNode<D>, depth: usize) -> Pixels {
        let y = self.origin.y + (self.level_spacing + node.item.size().height) * depth as f32;

        if node.children.is_empty() {
            node.item.set_position(point(self.origin.x, y));
            return node.item.size().width;
        }

        // Layout children first to get their widths
        let mut child_widths: Vec<Pixels> = Vec::new();
        let mut total_width = px(0.);

        for child in &node.children {
            let w = self.subtree_width(child);
            child_widths.push(w);
            total_width += w;
        }

        total_width += self.node_spacing * (node.children.len().saturating_sub(1)) as f32;

        // Position children
        let mut x = self.origin.x;
        for (child, width) in node.children.iter_mut().zip(child_widths.iter()) {
            let old_origin = self.origin;
            let child_layout = TreeLayout {
                origin: point(x, self.origin.y),
                ..self.clone()
            };
            child_layout.layout_top_down(child, depth + 1);

            // Restore origin for next iteration
            let _ = old_origin;
            x += *width + self.node_spacing;
        }

        // Position this node centered above its children
        let node_x = self.origin.x + (total_width - node.item.size().width) / 2.0;
        node.item.set_position(point(node_x, y));

        total_width
    }

    /// Calculate the height needed for a subtree (for left-to-right layout).
    fn subtree_height<D>(&self, node: &TreeNode<D>) -> Pixels {
        if node.children.is_empty() {
            node.item.size().height
        } else {
            let children_height: Pixels = node
                .children
                .iter()
                .map(|c| self.subtree_height(c))
                .fold(px(0.), |acc, h| acc + h);
            let gaps = self.node_spacing * (node.children.len().saturating_sub(1)) as f32;
            let total_children_height = children_height + gaps;
            total_children_height.max(node.item.size().height)
        }
    }

    /// Layout the tree left-to-right.
    fn layout_left_to_right<D>(&self, node: &mut TreeNode<D>, depth: usize) -> Pixels {
        let x = self.origin.x + (self.level_spacing + node.item.size().width) * depth as f32;

        if node.children.is_empty() {
            node.item.set_position(point(x, self.origin.y));
            return node.item.size().height;
        }

        // Layout children first to get their heights
        let mut child_heights: Vec<Pixels> = Vec::new();
        let mut total_height = px(0.);

        for child in &node.children {
            let h = self.subtree_height(child);
            child_heights.push(h);
            total_height += h;
        }

        total_height += self.node_spacing * (node.children.len().saturating_sub(1)) as f32;

        // Position children
        let mut y = self.origin.y;
        for (child, height) in node.children.iter_mut().zip(child_heights.iter()) {
            let child_layout = TreeLayout {
                origin: point(self.origin.x, y),
                ..self.clone()
            };
            child_layout.layout_left_to_right(child, depth + 1);

            y += *height + self.node_spacing;
        }

        // Position this node centered to the left of its children
        let node_y = self.origin.y + (total_height - node.item.size().height) / 2.0;
        node.item.set_position(point(x, node_y));

        total_height
    }

    fn calculate_tree_height<D>(&self, node: &TreeNode<D>) -> Pixels {
        let mut max_y = node.item.bounds.origin.y + node.item.size().height;
        for child in &node.children {
            max_y = max_y.max(self.calculate_tree_height(child));
        }
        max_y
    }

    fn calculate_tree_width<D>(&self, node: &TreeNode<D>) -> Pixels {
        let mut max_x = node.item.bounds.origin.x + node.item.size().width;
        for child in &node.children {
            max_x = max_x.max(self.calculate_tree_width(child));
        }
        max_x
    }

    fn flip_vertical<D>(&self, node: &mut TreeNode<D>, max_y: Pixels) {
        let current_y = node.item.bounds.origin.y;
        let new_y = max_y - current_y - node.item.size().height;
        node.item.bounds.origin.y = new_y;

        for child in &mut node.children {
            self.flip_vertical(child, max_y);
        }
    }

    fn flip_horizontal<D>(&self, node: &mut TreeNode<D>, max_x: Pixels) {
        let current_x = node.item.bounds.origin.x;
        let new_x = max_x - current_x - node.item.size().width;
        node.item.bounds.origin.x = new_x;

        for child in &mut node.children {
            self.flip_horizontal(child, max_x);
        }
    }
}

impl Layout for TreeLayout {
    fn apply<D: Clone>(&self, _items: &mut [CanvasItem<D>]) {
        // Tree layout requires hierarchical structure, so this is a no-op.
        // Use `apply_tree` instead.
    }

    fn calculate_bounds<D: Clone>(&self, _items: &[CanvasItem<D>]) -> Option<Bounds<Pixels>> {
        // Cannot calculate without tree structure
        None
    }
}

// ============================================================================
// Pack Layout
// ============================================================================

/// Pack layout uses a simple bin-packing algorithm to arrange items.
///
/// This is useful for variable-sized items that should fill a space efficiently.
#[derive(Clone, Debug)]
pub struct PackLayout {
    /// Width of the container.
    pub container_width: Pixels,

    /// Padding between items.
    pub padding: Pixels,

    /// Starting position.
    pub origin: Point<Pixels>,
}

impl Default for PackLayout {
    fn default() -> Self {
        Self {
            container_width: px(800.),
            padding: px(10.),
            origin: Point::default(),
        }
    }
}

impl PackLayout {
    /// Create a new pack layout.
    pub fn new(container_width: Pixels) -> Self {
        Self {
            container_width,
            ..Default::default()
        }
    }

    /// Set the padding between items.
    pub fn padding(mut self, padding: Pixels) -> Self {
        self.padding = padding;
        self
    }

    /// Set the starting origin.
    pub fn origin(mut self, origin: Point<Pixels>) -> Self {
        self.origin = origin;
        self
    }
}

impl Layout for PackLayout {
    fn apply<D: Clone>(&self, items: &mut [CanvasItem<D>]) {
        if items.is_empty() {
            return;
        }

        // Simple row-based packing
        let mut current_x = self.origin.x;
        let mut current_y = self.origin.y;
        let mut row_height = px(0.);

        for item in items.iter_mut() {
            let item_width = item.size().width;
            let item_height = item.size().height;

            // Check if item fits in current row
            if current_x + item_width > self.origin.x + self.container_width
                && current_x > self.origin.x
            {
                // Move to next row
                current_x = self.origin.x;
                current_y += row_height + self.padding;
                row_height = px(0.);
            }

            item.set_position(point(current_x, current_y));

            current_x += item_width + self.padding;
            row_height = row_height.max(item_height);
        }
    }

    fn calculate_bounds<D: Clone>(&self, items: &[CanvasItem<D>]) -> Option<Bounds<Pixels>> {
        if items.is_empty() {
            return None;
        }

        // Apply layout to a clone to calculate bounds
        let mut items: Vec<_> = items.iter().cloned().collect();
        self.apply(&mut items);

        let mut max_x = self.origin.x;
        let mut max_y = self.origin.y;

        for item in &items {
            max_x = max_x.max(item.bounds.origin.x + item.size().width);
            max_y = max_y.max(item.bounds.origin.y + item.size().height);
        }

        Some(Bounds::new(
            self.origin,
            size(max_x - self.origin.x, max_y - self.origin.y),
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, w: f32, h: f32) -> CanvasItem {
        CanvasItem::new(id, Bounds::new(point(px(0.), px(0.)), size(px(w), px(h))))
    }

    #[test]
    fn test_grid_layout() {
        let layout = GridLayout::new()
            .columns(2)
            .cell_size(size(px(100.), px(100.)))
            .gap(px(10.))
            .center_items(false);

        let mut items = vec![
            make_item("a", 50., 50.),
            make_item("b", 50., 50.),
            make_item("c", 50., 50.),
        ];

        layout.apply(&mut items);

        // First row
        assert_eq!(items[0].position().x, px(0.));
        assert_eq!(items[0].position().y, px(0.));

        assert_eq!(items[1].position().x, px(110.)); // 100 + 10 gap
        assert_eq!(items[1].position().y, px(0.));

        // Second row
        assert_eq!(items[2].position().x, px(0.));
        assert_eq!(items[2].position().y, px(110.));
    }

    #[test]
    fn test_grid_bounds_calculation() {
        let layout = GridLayout::new()
            .columns(3)
            .cell_size(size(px(100.), px(100.)))
            .gap(px(10.));

        let items = vec![
            make_item("a", 50., 50.),
            make_item("b", 50., 50.),
            make_item("c", 50., 50.),
            make_item("d", 50., 50.),
        ];

        let bounds = layout.calculate_bounds(&items).unwrap();
        assert_eq!(bounds.size.width, px(320.)); // 3 * 100 + 2 * 10
        assert_eq!(bounds.size.height, px(210.)); // 2 * 100 + 1 * 10
    }

    #[test]
    fn test_tree_node_count() {
        let mut root = TreeNode::new(make_item("root", 100., 50.));
        root.add_child(TreeNode::new(make_item("a", 80., 40.)));
        root.add_child(TreeNode::with_children(
            make_item("b", 80., 40.),
            vec![TreeNode::new(make_item("b1", 60., 30.))],
        ));

        assert_eq!(root.count(), 4);
        assert_eq!(root.depth(), 3);
    }

    #[test]
    fn test_pack_layout() {
        let layout = PackLayout::new(px(200.)).padding(px(10.));

        let mut items = vec![
            make_item("a", 80., 50.),
            make_item("b", 80., 60.),
            make_item("c", 80., 40.),
        ];

        layout.apply(&mut items);

        // First row: a and b
        assert_eq!(items[0].position().x, px(0.));
        assert_eq!(items[0].position().y, px(0.));

        assert_eq!(items[1].position().x, px(90.)); // 80 + 10
        assert_eq!(items[1].position().y, px(0.));

        // Second row: c (doesn't fit in first row)
        assert_eq!(items[2].position().x, px(0.));
        assert_eq!(items[2].position().y, px(70.)); // 60 (max height of row 1) + 10
    }
}
