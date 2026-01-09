# Textured Canvas Architecture v2

## Overview

This document describes a provider-based architecture for the infinite canvas with textured item rendering. The key insight is separating:

1. **InfiniteCanvas** - Display layer that handles pan/zoom and renders items by handle
2. **CanvasItemsProvider** - Logic layer that provides items and their textures

---

## Core Problem: Texture Sizing

GPUI layout happens inside the render lifecycle:
1. `request_layout()` - element requests space
2. `prepaint()` - bounds committed  
3. `paint()` - actual drawing

This creates a chicken-and-egg problem: we need to know the size to create the texture, but size comes from layout.

### Solution: Declared Sizes

For MVP, items **declare their size explicitly**, similar to design tools (Figma, Miro):
- Each canvas item has a defined frame size
- Content renders within that frame
- If content exceeds frame, it clips (or scrolls)

This is the pragmatic approach - most canvas use cases have fixed-size "cards" or "nodes".

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Main Application                                │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                         InfiniteCanvas                                 │ │
│  │                                                                        │ │
│  │   - Owns camera (pan/zoom)                                            │ │
│  │   - Calls provider.items() to get what to display                     │ │
│  │   - Calls provider.texture(handle) to get textures                    │ │
│  │   - Renders items as img() elements with scaling                      │ │
│  │                                                                        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    │ trait CanvasItemsProvider              │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    TexturedCanvasItemsProvider                         │ │
│  │                                                                        │ │
│  │   - Defines items and their bounds                                    │ │
│  │   - Owns TextureRenderer (background thread)                          │ │
│  │   - Manages texture cache                                             │ │
│  │   - Responds to texture requests                                      │ │
│  │                                                                        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    │ flume channels                         │
│                                    ▼                                        │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
┌────────────────────────────────────┼────────────────────────────────────────┐
│                                    │                                        │
│                    Texture Renderer Thread                                  │
│                    (Application::textured())                                │
│                                                                             │
│   - Receives render requests                                               │
│   - Creates TexturedSurfaceWindow at requested size                        │
│   - Renders content to texture                                             │
│   - Returns pixels as RenderImage                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Types

### Handle

```rust
/// Opaque handle to reference a canvas item
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasItemHandle(pub u64);
```

### Item Descriptor

```rust
/// Description of a canvas item for display purposes
/// This is what the canvas needs to know to position and display an item
pub struct CanvasItemDescriptor {
    /// Unique handle for this item
    pub handle: CanvasItemHandle,
    
    /// Position and size in canvas world coordinates
    pub bounds: Bounds<Pixels>,
    
    /// Z-order for layering (higher = on top) Nick: ? not sure this is needed
    pub z_index: u32,
}
```

### Texture State

```rust
/// The state of a texture for a given item
pub enum TextureState {
    /// Texture not yet requested
    NotRequested,
    
    /// Texture is being rendered
    Pending,
    
    /// Texture is ready
    Ready(Arc<RenderImage>),
    
    /// Texture rendering failed
    Failed(String),
}
```

---

## Provider Trait

```rust
/// Trait for providing canvas items and their textures
/// 
/// The provider is the "source of truth" for what items exist
/// and how to render them. The canvas just displays what the
/// provider gives it.
pub trait CanvasItemsProvider {
    /// Get all items that should be displayed
    /// 
    /// Called every frame - should be cheap (return cached list)
    fn items(&self) -> &[CanvasItemDescriptor];
    
    /// Get the texture state for an item
    /// 
    /// If NotRequested, the canvas will call request_texture().
    /// If Pending, the canvas shows a placeholder.
    /// If Ready, the canvas displays the texture.
    fn texture_state(&self, handle: CanvasItemHandle) -> TextureState;
    
    /// Request that a texture be rendered for this item
    /// 
    /// This is async - the provider should start rendering and
    /// return immediately. Check texture_state() for completion.
    fn request_texture(&mut self, handle: CanvasItemHandle);
    
    /// Notify that a texture is no longer needed
    /// 
    /// The provider can choose to keep it cached or discard it.
    fn release_texture(&mut self, handle: CanvasItemHandle);
    
    /// Called each frame to allow the provider to do housekeeping
    /// 
    /// E.g., poll for completed texture renders
    fn tick(&mut self);
}
```

---

## Textured Provider Implementation

```rust
use flume::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

/// A canvas items provider that renders items to textures
/// using a background Application::textured() instance
pub struct TexturedCanvasItemsProvider<F> {
    /// Item definitions (what to render)
    item_defs: Vec<ItemDefinition<F>>,
    
    /// Computed descriptors (for items() return)
    descriptors: Vec<CanvasItemDescriptor>,
    
    /// Texture cache by handle
    textures: HashMap<CanvasItemHandle, TextureState>,
    
    /// Channel to send render requests
    request_tx: Sender<RenderRequest>,
    
    /// Channel to receive render responses
    response_rx: Receiver<RenderResponse>,
    
    /// Next handle ID
    next_handle: u64,
}

/// Internal definition of an item
struct ItemDefinition<F> {
    handle: CanvasItemHandle,
    bounds: Bounds<Pixels>,
    z_index: u32,
    /// Factory function to create the content element
    content_factory: F,
}

/// Request sent to the renderer thread
struct RenderRequest {
    handle: CanvasItemHandle,
    size: Size<Pixels>,
    /// Serialized/cloneable content description
    content: RenderContent,
}

/// Response from the renderer thread
struct RenderResponse {
    handle: CanvasItemHandle,
    result: Result<Arc<RenderImage>, String>,
}

/// Content that can be sent to the render thread
/// (Must be Send + Sync, so can't contain closures directly)
#[derive(Clone)]
pub enum RenderContent {
    /// Simple colored box with text
    TextBox {
        text: String,
        background: u32,  // RGB hex
        text_color: u32,
    },
    
    /// Placeholder for more complex content types
    Custom {
        type_id: String,
        data: Vec<u8>,  // Serialized data
    },
}

impl<F> TexturedCanvasItemsProvider<F> 
where
    F: Fn() -> RenderContent + 'static,
{
    pub fn new() -> Self {
        let (request_tx, request_rx) = flume::unbounded();
        let (response_tx, response_rx) = flume::unbounded();
        
        // Spawn the renderer thread
        thread::spawn(move || {
            run_texture_renderer(request_rx, response_tx);
        });
        
        Self {
            item_defs: Vec::new(),
            descriptors: Vec::new(),
            textures: HashMap::new(),
            request_tx,
            response_rx,
            next_handle: 0,
        }
    }
    
    /// Add an item to the canvas
    pub fn add_item(
        &mut self,
        bounds: Bounds<Pixels>,
        z_index: u32,
        content_factory: F,
    ) -> CanvasItemHandle {
        let handle = CanvasItemHandle(self.next_handle);
        self.next_handle += 1;
        
        self.item_defs.push(ItemDefinition {
            handle,
            bounds,
            z_index,
            content_factory,
        });
        
        self.descriptors.push(CanvasItemDescriptor {
            handle,
            bounds,
            z_index,
        });
        
        self.textures.insert(handle, TextureState::NotRequested);
        
        handle
    }
    
    /// Update an item's bounds
    pub fn set_bounds(&mut self, handle: CanvasItemHandle, bounds: Bounds<Pixels>) {
        if let Some(def) = self.item_defs.iter_mut().find(|d| d.handle == handle) {
            def.bounds = bounds;
            
            // Also update descriptor
            if let Some(desc) = self.descriptors.iter_mut().find(|d| d.handle == handle) {
                desc.bounds = bounds;
            }
            
            // Invalidate texture (size changed)
            self.textures.insert(handle, TextureState::NotRequested);
        }
    }
    
    /// Remove an item
    pub fn remove_item(&mut self, handle: CanvasItemHandle) {
        self.item_defs.retain(|d| d.handle != handle);
        self.descriptors.retain(|d| d.handle != handle);
        self.textures.remove(&handle);
    }
}

impl<F> CanvasItemsProvider for TexturedCanvasItemsProvider<F>
where
    F: Fn() -> RenderContent + 'static,
{
    fn items(&self) -> &[CanvasItemDescriptor] {
        &self.descriptors
    }
    
    fn texture_state(&self, handle: CanvasItemHandle) -> TextureState {
        self.textures
            .get(&handle)
            .cloned()
            .unwrap_or(TextureState::NotRequested)
    }
    
    fn request_texture(&mut self, handle: CanvasItemHandle) {
        if let Some(def) = self.item_defs.iter().find(|d| d.handle == handle) {
            let content = (def.content_factory)();
            
            self.request_tx.send(RenderRequest {
                handle,
                size: def.bounds.size,
                content,
            }).ok();
            
            self.textures.insert(handle, TextureState::Pending);
        }
    }
    
    fn release_texture(&mut self, handle: CanvasItemHandle) {
        // Keep in cache but mark as not needed
        // Could implement LRU eviction here
    }
    
    fn tick(&mut self) {
        // Poll for completed renders
        while let Ok(response) = self.response_rx.try_recv() {
            let state = match response.result {
                Ok(image) => TextureState::Ready(image),
                Err(e) => TextureState::Failed(e),
            };
            self.textures.insert(response.handle, state);
        }
    }
}
```

---

## Texture Renderer Thread

```rust
fn run_texture_renderer(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
) {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use gpui::{Application, App, WindowOptions, WindowBounds, Bounds, point, px};
        
        let app = Application::textured();
        
        app.run(move |cx: &mut App| {
            // Process requests in a loop
            process_render_requests(request_rx, response_tx, cx);
        });
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        // On other platforms, just fail gracefully
        while let Ok(request) = request_rx.recv() {
            response_tx.send(RenderResponse {
                handle: request.handle,
                result: Err("TexturedSurface only supported on Linux".into()),
            }).ok();
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn process_render_requests(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
    cx: &mut App,
) {
    use gpui::{WindowOptions, WindowBounds, Bounds, point, px, AnyWindowHandle};
    
    // Keep processing until channel closes
    while let Ok(request) = request_rx.recv() {
        let bounds = Bounds::new(
            point(px(0.0), px(0.0)),
            request.size,
        );
        
        let content = request.content.clone();
        
        // Open window for this render
        let window_result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| ContentView { content })
            },
        );
        
        match window_result {
            Ok(window) => {
                let handle: AnyWindowHandle = window.into();
                let item_handle = request.handle;
                let response_tx = response_tx.clone();
                let size = request.size;
                
                // Render and capture
                cx.spawn(async move |cx| {
                    let result = render_and_capture(handle, size, &cx).await;
                    response_tx.send(RenderResponse {
                        handle: item_handle,
                        result,
                    }).ok();
                    
                    // Close window
                    cx.update_window(handle, |_, _, cx| {
                        cx.remove_window();
                    }).ok();
                }).detach();
            }
            Err(e) => {
                response_tx.send(RenderResponse {
                    handle: request.handle,
                    result: Err(format!("Failed to open window: {}", e)),
                }).ok();
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
async fn render_and_capture(
    handle: AnyWindowHandle,
    size: Size<Pixels>,
    cx: &gpui::AsyncApp,
) -> Result<Arc<RenderImage>, String> {
    // Draw to texture
    let draw_ok = cx.update_window(handle, |_, window, cx| {
        window.draw_and_present(cx)
    }).map_err(|e| e.to_string())?;
    
    if !draw_ok {
        return Err("Window not in valid state for drawing".into());
    }
    
    // Read pixels
    let pixels = cx.update_window(handle, |_, window, _| {
        window.read_pixels()
    }).map_err(|e| e.to_string())?
      .ok_or("read_pixels returned None")?;
    
    // Convert to RenderImage
    pixels_to_render_image(&pixels, size.width.0 as u32, size.height.0 as u32)
        .ok_or("Failed to convert pixels to image".into())
}

fn pixels_to_render_image(pixels: &[u8], width: u32, height: u32) -> Option<RenderImage> {
    use image::{RgbaImage, Frame};
    use smallvec::smallvec;
    
    // Convert BGRA to RGBA
    let mut rgba = Vec::with_capacity(pixels.len());
    for chunk in pixels.chunks(4) {
        if chunk.len() == 4 {
            rgba.push(chunk[2]); // R
            rgba.push(chunk[1]); // G
            rgba.push(chunk[0]); // B
            rgba.push(chunk[3]); // A
        }
    }
    
    let image = RgbaImage::from_raw(width, height, rgba)?;
    Some(RenderImage::new(smallvec![Frame::new(image)]))
}
```

---

## Content View (Renderer Side): Nick: Not sure what this does exactly

```rust
use gpui::{Render, Window, Context, IntoElement, div, prelude::*, rgb};

struct ContentView {
    content: RenderContent,
}

impl Render for ContentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.content {
            RenderContent::TextBox { text, background, text_color } => {
                div()
                    .size_full()
                    .bg(rgb(*background))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .text_xl()
                            .text_color(rgb(*text_color))
                            .child(text.clone())
                    )
            }
            RenderContent::Custom { type_id, data } => {
                // Would dispatch to registered content renderers
                div()
                    .size_full()
                    .bg(rgb(0xff0000))
                    .child(format!("Unknown content type: {}", type_id))
            }
        }
    }
}
```

---

## InfiniteCanvas Integration

```rust
use gpui::{Entity, Render, Window, Context, IntoElement, div, img, prelude::*};

pub struct InfiniteCanvas<P: CanvasItemsProvider> {
    provider: P,
    camera: Camera,
}

impl<P: CanvasItemsProvider> InfiniteCanvas<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            camera: Camera::default(),
        }
    }
}

impl<P: CanvasItemsProvider + 'static> Render for InfiniteCanvas<P> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Let provider do housekeeping (poll for textures, etc.)
        self.provider.tick();
        
        let viewport = window.viewport_size();
        let camera = &self.camera;
        
        div()
            .size_full()
            .overflow_hidden()
            .bg(rgb(0xf0f0f0))
            // Mouse handlers for pan/zoom would go here
            .children(
                self.provider.items().iter().filter_map(|item| {
                    // Transform to screen coordinates
                    let screen_bounds = camera.world_to_screen(item.bounds);
                    
                    // Culling: skip if off-screen
                    if !intersects(screen_bounds, viewport) {
                        return None;
                    }
                    
                    // Get or request texture
                    let content = match self.provider.texture_state(item.handle) {
                        TextureState::NotRequested => {
                            self.provider.request_texture(item.handle);
                            placeholder_element()
                        }
                        TextureState::Pending => {
                            placeholder_element()
                        }
                        TextureState::Ready(image) => {
                            img(image)
                                .size_full()
                                .object_fit(ObjectFit::Fill)
                                .into_any_element()
                        }
                        TextureState::Failed(msg) => {
                            error_element(&msg)
                        }
                    };
                    
                    Some(
                        div()
                            .absolute()
                            .left(screen_bounds.origin.x)
                            .top(screen_bounds.origin.y)
                            .w(screen_bounds.size.width)
                            .h(screen_bounds.size.height)
                            .child(content)
                    )
                })
            )
    }
}

fn placeholder_element() -> AnyElement {
    div()
        .size_full()
        .bg(rgb(0xdddddd))
        .flex()
        .justify_center()
        .items_center()
        .child("Loading...")
        .into_any_element()
}

fn error_element(msg: &str) -> AnyElement {
    div()
        .size_full()
        .bg(rgb(0xffcccc))
        .flex()
        .justify_center()
        .items_center()
        .child(format!("Error: {}", msg))
        .into_any_element()
}
```

---

## Usage Example

```rust
fn main() {
    let app = Application::new();
    
    app.run(|cx| {
        // Create the provider with some items
        let mut provider = TexturedCanvasItemsProvider::new();
        
        provider.add_item(
            Bounds::new(point(px(100.0), px(100.0)), size(px(200.0), px(150.0))),
            0,
            || RenderContent::TextBox {
                text: "Hello World".into(),
                background: 0x3498db,
                text_color: 0xffffff,
            },
        );
        
        provider.add_item(
            Bounds::new(point(px(350.0), px(200.0)), size(px(200.0), px(150.0))),
            0,
            || RenderContent::TextBox {
                text: "Second Item".into(),
                background: 0xe74c3c,
                text_color: 0xffffff,
            },
        );
        
        // Create the canvas with the provider
        let canvas = InfiniteCanvas::new(provider);
        
        cx.open_window(
            WindowOptions::default(),
            |_, cx| cx.new(|_| canvas),
        );
    });
}
```

---

## Key Design Decisions

### 1. Declared Sizes
Items declare their size upfront. This sidesteps the layout-before-texture problem.

### 2. Handle-Based References  
Items are referenced by opaque handles, not indices or direct references. This allows the provider to manage items internally however it wants.

### 3. Provider Owns Rendering Logic
The canvas doesn't know *how* textures are made - it just requests them by handle. The provider could use TexturedSurface, pre-rendered images, or anything else.

### 4. Async Texture Loading
Textures render asynchronously. The canvas shows placeholders until ready, keeping the UI responsive.

### 5. Texture State Machine
Clear states (NotRequested → Pending → Ready/Failed) make it easy to handle all cases in the render loop.

---

## Future Enhancements

1. **LOD (Level of Detail)** - Provider could return different resolution textures based on zoom
2. **Texture Pooling** - Reuse texture windows instead of creating/destroying
3. **Batch Rendering** - Render multiple items to one large texture
4. **Dirty Tracking** - Only re-render items that changed
5. **Priority Queue** - Render visible items first
6. **Memory Management** - LRU cache for textures, eviction policies

---

## Dependencies

```toml
[dependencies]
gpui = "0.2.2"
flume = "0.11"
image = "0.25"
smallvec = "1"
```

---

## References

- `vendor/zed/crates/gpui/examples/textured_surface.rs`
- `vendor/zed/crates/gpui/src/elements/img.rs`
- `vendor/zed/crates/gpui/src/assets.rs` - RenderImage
- `research/textured-surface-workflow.md`
