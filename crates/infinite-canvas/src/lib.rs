//! Infinite Canvas for GPUI
//!
//! A pannable, zoomable canvas component for GPUI that can display and layout
//! rectangular objects on an infinite 2D plane.
//!
//! # Core Concepts
//!
//! - **Camera**: Controls the viewport position and zoom level
//! - **Canvas**: The main component that renders items with pan/zoom
//! - **CanvasItem**: Rectangular objects positioned on the canvas
//! - **Layout**: Algorithms for arranging items (grid, tree, force-directed, etc.)
//!
//! # Example
//!
//! ```no_run
//! use infinite_canvas::prelude::*;
//! use gpui::*;
//!
//! // Create a canvas with some items
//! let canvas = InfiniteCanvas::new("my-canvas")
//!     .items(vec![
//!         CanvasItem::new("item-1", bounds(point(px(0.), px(0.)), size(px(100.), px(80.)))),
//!         CanvasItem::new("item-2", bounds(point(px(150.), px(0.)), size(px(100.), px(80.)))),
//!     ]);
//! ```

mod camera;
mod canvas;
mod item;
pub mod layout;
mod options;

pub use camera::*;
pub use canvas::*;
pub use item::*;
pub use layout::*;
pub use options::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::camera::Camera;
    pub use crate::canvas::InfiniteCanvas;
    pub use crate::item::{CanvasItem, ItemId};
    pub use crate::layout::{GridLayout, Layout, TreeLayout};
    pub use crate::options::CanvasOptions;
}

/// Initialize the infinite canvas component.
///
/// Call this at your application's entry point after initializing gpui-component.
pub fn init(_cx: &mut gpui::App) {
    // Reserved for future initialization needs (e.g., registering actions)
}
