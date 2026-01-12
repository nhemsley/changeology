# Infinite Canvas

An infinite canvas component for GPUI with pan, zoom, and provider-based item rendering.

## Architecture

```
InfiniteCanvas
├── Camera (built-in, handles pan/zoom/coordinate transforms)
├── CanvasOptions (grid, zoom limits, speeds, etc.)
└── provider: impl CanvasItemsProvider
    ├── TexturedCanvasItemsProvider (renders items as zoomable textures)
    └── (future) RenderedCanvasItemsProvider (renders items directly)
```

## Features

- **Pan & Zoom**: Built-in camera controls with middle-mouse drag to pan and scroll wheel to zoom
- **Coordinate Systems**: Automatic conversion between screen space and canvas space
- **Provider-based Items**: Pluggable item providers for different rendering strategies
- **Textured Rendering**: Items rendered as textures for smooth zooming (Linux/FreeBSD)
- **Background Grid**: Optional configurable grid display
- **Viewport Culling**: Only visible items are rendered for performance

## Quick Start

```rust
use infinite_canvas::prelude::*;
use gpui::*;
use std::cell::RefCell;
use std::rc::Rc;

// Create a provider (wrapped in Rc<RefCell<>> for sharing)
let provider = Rc::new(RefCell::new(
    TexturedCanvasItemsProvider::with_sizing(ItemSizing::FixedWidth {
        width: px(280.0),
        estimated_height: px(150.0),
    })
));

// Add items to the provider
provider.borrow_mut().add_item(
    "card-1",
    point(px(50.0), px(50.0)),
    window,
    cx,
    || div().p_4().bg(rgb(0x3498db)).child("Hello!")
);

// Create the canvas with the provider
let canvas = InfiniteCanvas::new("my-canvas", provider.clone())
    .options(CanvasOptions::new().show_grid(true));
```

## Core Types

### `InfiniteCanvas<P>`

The main canvas component. Takes a `SharedProvider<P>` and handles:
- Camera state management (persisted across renders)
- Pan/zoom input handling
- Grid rendering
- Item culling and rendering via the provider

### `CanvasItemsProvider` Trait

Implement this trait to create custom item providers:

```rust
pub trait CanvasItemsProvider {
    /// Get descriptors for all items (id, bounds, z_index)
    fn items(&self) -> Vec<ItemDescriptor>;
    
    /// Render an item at the given screen bounds
    fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement>;
}
```

### `TexturedCanvasItemsProvider`

Built-in provider that renders items as textures using GPUI's `TexturedView`:
- Items are rendered once to a texture
- Textures scale smoothly when zooming
- Async rendering doesn't block the UI
- Platform support: Linux/FreeBSD (other platforms show placeholders)

### `Camera`

Viewport state with coordinate transforms:

```rust
let mut camera = Camera::default();

// Pan the camera
camera.pan(point(px(10.), px(20.)));

// Zoom around a point (e.g., cursor position)
camera.zoom_around(1.1, cursor_position, 0.1, 8.0);

// Convert coordinates
let canvas_point = camera.screen_to_canvas(screen_point);
let screen_bounds = camera.canvas_to_screen_bounds(canvas_bounds);
```

### `CanvasOptions`

Configuration for canvas behavior:

```rust
let options = CanvasOptions::new()
    .min_zoom(0.1)
    .max_zoom(8.0)
    .zoom_speed(1.0)
    .show_grid(true)
    .grid_size(px(20.))
    .wheel_behavior(WheelBehavior::Zoom);
```

## Controls

| Input | Action |
|-------|--------|
| Scroll wheel | Zoom in/out (centered on cursor) |
| Middle-click drag | Pan canvas |

## Running the Example

```bash
cargo run -p infinite-canvas --example textured
```

## Module Structure

- `camera.rs` - Camera state and coordinate conversion
- `canvas.rs` - Main canvas component and rendering
- `options.rs` - Configuration options
- `provider.rs` - `CanvasItemsProvider` trait
- `textured_provider.rs` - Textured items provider implementation

## License

Apache-2.0