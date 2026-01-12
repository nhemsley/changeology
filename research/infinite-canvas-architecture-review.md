# Infinite Canvas Architecture Review

> **Date**: 2026-01-12  
> **Status**: Implementation Complete  
> **Location**: `changeology/crates/infinite-canvas/`

## Executive Summary

The `infinite-canvas` crate has been redesigned with a clean, provider-based architecture:

```
InfiniteCanvas<P: CanvasItemsProvider>
├── Camera (built-in, handles pan/zoom/coordinate transforms)
├── CanvasOptions (grid, zoom limits, speeds, etc.)
└── provider: SharedProvider<P> = Rc<RefCell<P>>
    └── TexturedCanvasItemsProvider (renders items as zoomable textures)
```

**Key Design Decisions:**
1. `InfiniteCanvas` owns the camera and wires up all pan/zoom events
2. Items are provided via the `CanvasItemsProvider` trait
3. Providers are shared via `Rc<RefCell<P>>` to allow mutation while canvas renders
4. `TexturedCanvasItemsProvider` uses GPUI's `TexturedView` for smooth zooming

---

## New Architecture

### Core Components

**`InfiniteCanvas<P>`** - The main canvas component
- Takes a `SharedProvider<P>` (= `Rc<RefCell<P>>`)
- Manages camera state internally (via GPUI element state)
- Handles all pan/zoom input events
- Renders grid, culls items, calls provider to render each item

**`CanvasItemsProvider` trait** - Abstraction for item sources
```rust
pub trait CanvasItemsProvider {
    fn items(&self) -> Vec<ItemDescriptor>;
    fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement>;
}
```

**`TexturedCanvasItemsProvider`** - Built-in provider for textured items
- Renders items as textures via `TexturedView`
- Items scale smoothly when zooming
- Async rendering doesn't block UI

### Usage Pattern

```rust
use infinite_canvas::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

struct MyView {
    provider: Rc<RefCell<TexturedCanvasItemsProvider>>,
}

impl Render for MyView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        InfiniteCanvas::new("canvas", self.provider.clone())
            .options(CanvasOptions::new().show_grid(true))
    }
}
```

The user only needs to:
1. Create a provider wrapped in `Rc<RefCell<>>`
2. Add items to the provider
3. Pass the provider to `InfiniteCanvas`

The canvas handles everything else (camera, events, rendering).

---

## Files Structure

```
infinite-canvas/
├── src/
│   ├── lib.rs              # Re-exports, prelude
│   ├── camera.rs           # Camera state and coordinate transforms
│   ├── canvas.rs           # InfiniteCanvas component
│   ├── options.rs          # CanvasOptions, WheelBehavior
│   ├── provider.rs         # CanvasItemsProvider trait
│   └── textured_provider.rs # TexturedCanvasItemsProvider implementation
├── examples/
│   └── textured.rs         # Example using InfiniteCanvas with TexturedCanvasItemsProvider
├── Cargo.toml
└── README.md
```

---

## Key Components

### `CanvasItemsProvider` Trait

```rust
pub trait CanvasItemsProvider {
    /// Get descriptors for all items (id, canvas bounds, z_index)
    fn items(&self) -> Vec<ItemDescriptor>;
    
    /// Render an item at the given screen bounds (after camera transform)
    fn render_item(&self, id: &str, screen_bounds: Bounds<Pixels>, cx: &App) -> Option<AnyElement>;
    
    /// Get the number of items (default impl)
    fn item_count(&self) -> usize;
    
    /// Get content bounds (default impl)
    fn content_bounds(&self) -> Option<Bounds<Pixels>>;
}
```

### `TexturedCanvasItemsProvider`

Implements `CanvasItemsProvider` using GPUI's `TexturedView`:
- Items are rendered once to a texture in a background thread
- Textures scale smoothly when zooming
- Platform support: Linux/FreeBSD (others show placeholders)

The `invalidate()` bug from the old implementation has been **fixed** - it now properly updates both `view` and `texture_getter`.

### `Camera`

Unchanged from before - provides coordinate transforms and zoom/pan methods:
- `screen_to_canvas()` / `canvas_to_screen()`
- `canvas_to_screen_bounds()`
- `pan(delta)`
- `zoom_around(factor, anchor, min, max)`
- `zoom_to_fit(bounds, viewport, padding, min, max)`

### `CanvasOptions`

Configuration for canvas behavior:
- `min_zoom`, `max_zoom` - zoom limits
- `zoom_speed`, `pan_speed` - input sensitivity
- `show_grid`, `grid_size` - background grid
- `wheel_behavior` - zoom, pan, or none
- `locked` - disable all input

---

## Comparison: Old vs New

| Aspect | Old Architecture | New Architecture |
|--------|------------------|------------------|
| Camera ownership | User's view | InfiniteCanvas (internal) |
| Event handling | User implements | InfiniteCanvas (automatic) |
| Item rendering | User calls provider | InfiniteCanvas calls provider |
| Provider sharing | Not needed | `Rc<RefCell<P>>` |
| Example complexity | ~300 lines | ~100 lines |

---

## Example: New textured.rs

```rust
use infinite_canvas::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

struct TexturedCanvasView {
    provider: Rc<RefCell<TexturedCanvasItemsProvider>>,
}

impl TexturedCanvasView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let provider = Rc::new(RefCell::new(
            TexturedCanvasItemsProvider::with_sizing(ItemSizing::FixedWidth {
                width: px(280.0),
                estimated_height: px(150.0),
            })
        ));

        // Add items
        provider.borrow_mut().add_item("card-1", point(px(50.0), px(50.0)), window, cx, || {
            div().p_4().bg(rgb(0x3498db)).child("Hello!")
        });

        Self { provider }
    }
}

impl Render for TexturedCanvasView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        InfiniteCanvas::new("canvas", self.provider.clone())
            .options(CanvasOptions::new().show_grid(true).min_zoom(0.1).max_zoom(10.0))
    }
}
```

**That's it!** No manual camera management, no event handlers, no coordinate transforms.

---

## Architecture Diagram (New)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User's View                                 │
│                                                                     │
│  struct MyView {                                                    │
│      provider: Rc<RefCell<TexturedCanvasItemsProvider>>,           │
│  }                                                                  │
│                                                                     │
│  impl Render {                                                      │
│      fn render(...) {                                               │
│          InfiniteCanvas::new("id", self.provider.clone())          │
│              .options(CanvasOptions::new().show_grid(true))        │
│      }                                                              │
│  }                                                                  │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      InfiniteCanvas<P>                              │
│                                                                     │
│  - Owns Camera (via element state, persists across renders)        │
│  - Handles pan (middle-drag) and zoom (scroll wheel)               │
│  - Renders background grid                                          │
│  - Culls items outside viewport                                     │
│  - Calls provider.items() and provider.render_item()               │
│  - Fires on_camera_change callback                                  │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│              TexturedCanvasItemsProvider                            │
│              (implements CanvasItemsProvider)                       │
│                                                                     │
│  - Stores items (position, size, z-index)                          │
│  - Creates TexturedView for each item                              │
│  - Returns textures scaled to screen bounds                        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Migration Notes

### For existing code using the old pattern:

1. Wrap provider in `Rc<RefCell<>>`
2. Remove manual camera management
3. Remove pan/zoom event handlers
4. Replace custom rendering loop with `InfiniteCanvas`
5. Use `provider.borrow_mut()` to add items

### DiffCanvasView migration:

The existing `DiffCanvasView` should be updated to use `InfiniteCanvas` instead of manually implementing camera/rendering. This will significantly simplify the code.
