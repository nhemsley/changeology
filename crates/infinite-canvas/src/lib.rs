//! Infinite Canvas for GPUI
//!
//! A pannable, zoomable canvas component for GPUI that displays items
//! from a `CanvasItemsProvider`.
//!
//! # Architecture
//!
//! - **`InfiniteCanvas`** - The main canvas component that handles camera, events, and rendering
//! - **`CanvasItemsProvider`** - Trait for providing items to the canvas
//! - **`TexturedCanvasItemsProvider`** - Provider that renders items as zoomable textures
//! - **`Camera`** - Viewport state (offset, zoom) with coordinate transforms
//! - **`CanvasOptions`** - Configuration for zoom limits, grid, etc.
//!
//! # Example
//!
//! ```ignore
//! use infinite_canvas::prelude::*;
//! use gpui::*;
//!
//! // Create a provider and add items
//! let mut provider = TexturedCanvasItemsProvider::new();
//! provider.add_item("card-1", point(px(0.0), px(0.0)), window, cx, || {
//!     div().p_4().bg(rgb(0x3498db)).child("Hello!")
//! });
//!
//! // Create the canvas with the provider
//! let canvas = InfiniteCanvas::new("my-canvas", provider)
//!     .options(CanvasOptions::new().show_grid(true));
//! ```

mod camera;
mod canvas;
mod options;
mod provider;
mod textured_provider;

pub use camera::Camera;
pub use canvas::{CanvasElement, InfiniteCanvas, SharedProvider};
pub use options::{
    CameraConstraints, CanvasOptions, ConstraintBehavior, ConstraintBounds, WheelBehavior,
};
pub use provider::{CanvasItemsProvider, ItemDescriptor, ItemId};
pub use textured_provider::{ItemSizing, TexturedCanvasItemsProvider};

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::camera::Camera;
    pub use crate::canvas::{InfiniteCanvas, SharedProvider};
    pub use crate::options::CanvasOptions;
    pub use crate::provider::{CanvasItemsProvider, ItemDescriptor, ItemId};
    pub use crate::textured_provider::{ItemSizing, TexturedCanvasItemsProvider};
}

/// Initialize the infinite canvas component.
///
/// Call this at your application's entry point after initializing gpui-component.
pub fn init(_cx: &mut gpui::App) {
    // Reserved for future initialization needs
}
