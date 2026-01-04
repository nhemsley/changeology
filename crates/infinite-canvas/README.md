# Infinite Canvas

An infinite canvas component for GPUI with pan, zoom, and layout algorithms.

## Features

- **Pan & Zoom**: Smooth camera controls with mouse wheel zoom and drag-to-pan
- **Coordinate Systems**: Automatic conversion between screen space and canvas space
- **Layout Algorithms**: Built-in layouts for arranging items:
  - Grid layout
  - Tree layout (top-down, left-to-right, bottom-up, right-to-left)
  - Pack layout (bin packing for variable-sized items)
- **Background Grid**: Optional configurable grid display
- **Viewport Culling**: Only visible items are rendered for performance

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
infinite-canvas = { path = "../infinite-canvas" }
```

## Quick Start

```rust
use infinite_canvas::{InfiniteCanvas, CanvasItem, Camera, CanvasOptions};
use gpui::*;

// Create items to display on the canvas
let items = vec![
    CanvasItem::new("item-1", Bounds::new(point(px(0.), px(0.)), size(px(100.), px(80.)))),
    CanvasItem::new("item-2", Bounds::new(point(px(150.), px(0.)), size(px(100.), px(80.)))),
];

// Create the canvas
let canvas = InfiniteCanvas::new("my-canvas")
    .camera(Camera::default())
    .options(CanvasOptions::new().show_grid(true))
    .items(items);
```

## Core Concepts

### Camera

The `Camera` controls the viewport into the infinite canvas:

```rust
use infinite_canvas::Camera;

let mut camera = Camera::default();

// Pan the camera
camera.pan(point(px(10.), px(20.)));

// Zoom around a point (e.g., cursor position)
camera.zoom_around(1.1, cursor_position, 0.1, 8.0);

// Convert between coordinate systems
let canvas_point = camera.screen_to_canvas(screen_point);
let screen_point = camera.canvas_to_screen(canvas_point);
```

### Canvas Items

Items are rectangular objects placed on the canvas:

```rust
use infinite_canvas::{CanvasItem, ItemId};

// Simple item with no data
let item = CanvasItem::new("my-item", bounds);

// Item with associated data
let item = CanvasItem::with_data("my-item", bounds, MyData { ... });

// Item properties
item.with_selected(true)
    .with_z_index(10)
    .with_visible(true);
```

### Layout Algorithms

#### Grid Layout

Arrange items in a regular grid:

```rust
use infinite_canvas::layout::{GridLayout, Layout};

let layout = GridLayout::new()
    .columns(4)
    .cell_size(size(px(100.), px(100.)))
    .gap(px(10.));

layout.apply(&mut items);
```

#### Tree Layout

Hierarchical tree arrangement:

```rust
use infinite_canvas::layout::{TreeLayout, TreeLayoutStyle, TreeNode};

let layout = TreeLayout::new()
    .style(TreeLayoutStyle::TopDown)
    .node_spacing(px(20.))
    .level_spacing(px(60.));

let mut tree = TreeNode::with_children(root_item, vec![
    TreeNode::new(child1),
    TreeNode::new(child2),
]);

layout.apply_tree(&mut tree);
```

#### Pack Layout

Bin packing for variable-sized items:

```rust
use infinite_canvas::layout::{PackLayout, Layout};

let layout = PackLayout::new(px(800.))
    .padding(px(10.));

layout.apply(&mut items);
```

### Canvas Options

Configure canvas behavior:

```rust
use infinite_canvas::{CanvasOptions, WheelBehavior};

let options = CanvasOptions::new()
    .min_zoom(0.1)
    .max_zoom(8.0)
    .zoom_speed(1.0)
    .pan_speed(1.0)
    .show_grid(true)
    .grid_size(px(20.))
    .wheel_behavior(WheelBehavior::Zoom);
```

## Running the Example

```bash
cargo run --example basic
```

## Controls

| Input | Action |
|-------|--------|
| Scroll wheel | Zoom in/out (centered on cursor) |
| Middle-click drag | Pan canvas |

## Architecture

The crate is structured as follows:

- `camera.rs` - Camera state and coordinate conversion
- `canvas.rs` - Main canvas component and rendering
- `item.rs` - Canvas items and collections
- `layout.rs` - Layout algorithms
- `options.rs` - Configuration options

## License

Apache-2.0