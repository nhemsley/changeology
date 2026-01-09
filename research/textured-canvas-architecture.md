# Textured Canvas Architecture

## Overview

This document describes an architecture for rendering canvas items to textures using a separate `Application::textured()` instance, then displaying them as scaled images in the main infinite canvas window.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Main Application                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     InfiniteCanvas (Window)                          │   │
│  │                                                                      │   │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐                          │   │
│  │   │  img()   │  │  img()   │  │  img()   │   ← Scaled images        │   │
│  │   │ item_1   │  │ item_2   │  │ item_3   │     from textures        │   │
│  │   └──────────┘  └──────────┘  └──────────┘                          │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              ▲                                              │
│                              │ RenderImage (via flume channel)             │
│                              │                                              │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               │
┌──────────────────────────────┼──────────────────────────────────────────────┐
│                              │                                              │
│              Texture Renderer Thread (Application::textured())              │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │  TexturedSurfaceWindow                                               │  │
│   │                                                                      │  │
│   │   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │  │
│   │   │ CanvasItem  │───▶│   render    │───▶│   pixels    │             │  │
│   │   │    View     │    │  to texture │    │  (BGRA)     │             │  │
│   │   └─────────────┘    └─────────────┘    └─────────────┘             │  │
│   │                                                                      │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Components

### 1. Render Request / Response Types

```rust
/// A request to render a canvas item to a texture
#[derive(Clone)]
pub struct RenderRequest {
    /// Unique identifier for this item
    pub item_id: u64,
    /// The content to render (serializable description)
    pub content: CanvasItemContent,
    /// Size to render at (native resolution)
    pub size: Size<Pixels>,
}

/// The rendered result
pub struct RenderResponse {
    /// The item ID this response is for
    pub item_id: u64,
    /// The rendered image (BGRA pixels wrapped as RenderImage)
    pub image: Arc<RenderImage>,
    /// Original render size
    pub size: Size<Pixels>,
}

/// Description of what to render
#[derive(Clone)]
pub enum CanvasItemContent {
    /// A simple colored box with text
    TextBox {
        text: String,
        background: Hsla,
        text_color: Hsla,
    },
    /// A custom view (requires a factory function)
    Custom(Arc<dyn Fn() -> AnyView + Send + Sync>),
}
```

### 2. Texture Renderer (Background Thread)

```rust
use flume::{Receiver, Sender};
use gpui::{Application, App, RenderImage, px, size};
use image::Frame;
use std::sync::Arc;
use std::thread;

pub struct TextureRenderer {
    request_tx: Sender<RenderRequest>,
    response_rx: Receiver<RenderResponse>,
    _thread: thread::JoinHandle<()>,
}

impl TextureRenderer {
    pub fn new() -> Self {
        let (request_tx, request_rx) = flume::unbounded::<RenderRequest>();
        let (response_tx, response_rx) = flume::unbounded::<RenderResponse>();
        
        let thread = thread::spawn(move || {
            Self::run_renderer(request_rx, response_tx);
        });
        
        Self {
            request_tx,
            response_rx,
            _thread: thread,
        }
    }
    
    /// Send a render request
    pub fn request_render(&self, request: RenderRequest) {
        self.request_tx.send(request).ok();
    }
    
    /// Poll for completed renders (non-blocking)
    pub fn poll_responses(&self) -> Vec<RenderResponse> {
        self.response_rx.try_iter().collect()
    }
    
    fn run_renderer(
        request_rx: Receiver<RenderRequest>,
        response_tx: Sender<RenderResponse>,
    ) {
        // This runs in a separate thread with its own Application
        let app = Application::textured();
        
        app.run(move |cx: &mut App| {
            // Process requests as they come in
            Self::process_requests(request_rx, response_tx, cx);
        });
    }
    
    fn process_requests(
        request_rx: Receiver<RenderRequest>,
        response_tx: Sender<RenderResponse>,
        cx: &mut App,
    ) {
        use gpui::{WindowOptions, WindowBounds, Bounds, point};
        
        while let Ok(request) = request_rx.recv() {
            let bounds = Bounds::new(
                point(px(0.0), px(0.0)),
                request.size,
            );
            
            // Create a window for this render
            let window_result = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| ItemRenderView::new(request.content.clone()))
                },
            );
            
            if let Ok(window) = window_result {
                let handle: gpui::AnyWindowHandle = window.into();
                let item_id = request.item_id;
                let size = request.size;
                let response_tx = response_tx.clone();
                
                cx.spawn(async move |cx| {
                    // Draw to texture
                    let draw_result = cx.update_window(handle, |_, window, cx| {
                        window.draw_and_present(cx)
                    });
                    
                    if draw_result.is_ok() {
                        // Read pixels
                        if let Ok(Some(pixels)) = cx.update_window(handle, |_, window, _| {
                            window.read_pixels()
                        }) {
                            // Convert BGRA pixels to RenderImage
                            if let Some(image) = pixels_to_render_image(
                                &pixels,
                                size.width.0 as u32,
                                size.height.0 as u32,
                            ) {
                                response_tx.send(RenderResponse {
                                    item_id,
                                    image: Arc::new(image),
                                    size,
                                }).ok();
                            }
                        }
                    }
                    
                    // Close the window
                    cx.update_window(handle, |_, _, cx| {
                        cx.remove_window();
                    }).ok();
                }).detach();
            }
        }
    }
}

/// Convert raw BGRA pixels to a RenderImage
fn pixels_to_render_image(pixels: &[u8], width: u32, height: u32) -> Option<RenderImage> {
    use image::{RgbaImage, Frame};
    
    // Convert BGRA to RGBA
    let mut rgba = Vec::with_capacity(pixels.len());
    for chunk in pixels.chunks(4) {
        if chunk.len() == 4 {
            rgba.push(chunk[2]); // R (was B)
            rgba.push(chunk[1]); // G
            rgba.push(chunk[0]); // B (was R)
            rgba.push(chunk[3]); // A
        }
    }
    
    let image = RgbaImage::from_raw(width, height, rgba)?;
    Some(RenderImage::new(smallvec::smallvec![Frame::new(image)]))
}
```

### 3. Item Render View (Used by Texture Renderer)

```rust
use gpui::{Render, Window, Context, IntoElement, div, prelude::*};

struct ItemRenderView {
    content: CanvasItemContent,
}

impl ItemRenderView {
    fn new(content: CanvasItemContent) -> Self {
        Self { content }
    }
}

impl Render for ItemRenderView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.content {
            CanvasItemContent::TextBox { text, background, text_color } => {
                div()
                    .size_full()
                    .bg(*background)
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .text_xl()
                            .text_color(*text_color)
                            .child(text.clone())
                    )
            }
            CanvasItemContent::Custom(factory) => {
                // Would need to handle custom views
                div().size_full().bg(gpui::red())
            }
        }
    }
}
```

### 4. Infinite Canvas Integration

```rust
use gpui::{Entity, img, RenderImage, Size, Pixels};
use std::collections::HashMap;
use std::sync::Arc;

pub struct InfiniteCanvas {
    items: Vec<CanvasItem>,
    texture_cache: HashMap<u64, Arc<RenderImage>>,
    texture_renderer: Arc<TextureRenderer>,
    camera: Camera,
}

pub struct CanvasItem {
    pub id: u64,
    pub bounds: Bounds<Pixels>,
    pub content: CanvasItemContent,
    pub needs_render: bool,
}

impl InfiniteCanvas {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            texture_cache: HashMap::new(),
            texture_renderer: Arc::new(TextureRenderer::new()),
            camera: Camera::default(),
        }
    }
    
    /// Request textures for items that need them
    fn request_textures(&mut self) {
        for item in &mut self.items {
            if item.needs_render && !self.texture_cache.contains_key(&item.id) {
                self.texture_renderer.request_render(RenderRequest {
                    item_id: item.id,
                    content: item.content.clone(),
                    size: item.bounds.size,
                });
                item.needs_render = false;
            }
        }
    }
    
    /// Poll for completed texture renders
    fn poll_texture_updates(&mut self) {
        for response in self.texture_renderer.poll_responses() {
            self.texture_cache.insert(response.item_id, response.image);
        }
    }
}

impl Render for InfiniteCanvas {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Poll for texture updates
        self.poll_texture_updates();
        
        // Request any pending textures
        self.request_textures();
        
        let zoom = self.camera.zoom;
        let offset = self.camera.offset;
        
        div()
            .size_full()
            .overflow_hidden()
            .children(
                self.items.iter().filter_map(|item| {
                    // Transform item bounds by camera
                    let screen_bounds = self.camera.world_to_screen(item.bounds);
                    
                    // Skip if off-screen
                    if !self.is_visible(&screen_bounds) {
                        return None;
                    }
                    
                    // Get cached texture or show placeholder
                    let element = if let Some(texture) = self.texture_cache.get(&item.id) {
                        // Display as scaled image
                        img(texture.clone())
                            .size_full()
                            .object_fit(ObjectFit::Fill)
                            .into_any_element()
                    } else {
                        // Placeholder while loading
                        div()
                            .size_full()
                            .bg(gpui::rgb(0xcccccc))
                            .flex()
                            .justify_center()
                            .items_center()
                            .child("Loading...")
                            .into_any_element()
                    };
                    
                    Some(
                        div()
                            .absolute()
                            .left(screen_bounds.origin.x)
                            .top(screen_bounds.origin.y)
                            .w(screen_bounds.size.width)
                            .h(screen_bounds.size.height)
                            .child(element)
                    )
                })
            )
    }
}
```

---

## Data Flow

1. **Main thread** creates `InfiniteCanvas` with a `TextureRenderer`
2. **Canvas items** are added with `CanvasItemContent` descriptions
3. **On render**, canvas checks which items need textures and sends `RenderRequest`s
4. **Background thread** (running `Application::textured()`) receives requests
5. **For each request**:
   - Opens a `TexturedSurfaceWindow`
   - Creates an `ItemRenderView` with the content
   - Calls `draw_and_present()` to render to GPU texture
   - Calls `read_pixels()` to get BGRA pixel data
   - Converts to `RenderImage` and sends via `RenderResponse`
6. **Main thread** polls for responses and updates `texture_cache`
7. **On next render**, cached textures are displayed as `img()` elements
8. **Zoom/pan** simply scales/moves the `img()` elements - texture stays same resolution

---

## Key Benefits

1. **Full GPUI rendering** - Text, shapes, images, all work
2. **Decoupled** - Texture rendering doesn't block main UI
3. **Simple scaling** - Just change `img()` element size for zoom
4. **Caching** - Textures persist until invalidated
5. **No GPUI modifications** - Uses existing APIs

---

## Limitations

1. **Memory** - Each item needs its own texture in CPU memory (then uploaded to GPU)
2. **Latency** - Items show placeholder until texture is ready
3. **No GPU sharing** - Separate GPU context, textures copied via CPU
4. **Thread overhead** - Separate application instance

---

## Dependencies

```toml
[dependencies]
gpui = "0.2.2"
flume = "0.11"         # Cross-thread channels (what Zed uses)
image = "0.25"         # For Frame/RgbaImage
smallvec = "1"         # For RenderImage::new()
```

---

## Future Optimizations

1. **Batch rendering** - Render multiple items in one window
2. **LOD** - Different texture resolutions for different zoom levels
3. **Texture pooling** - Reuse textures for similar-sized items
4. **GPU sharing** - Apply `BladeRenderer::draw_to_texture()` refactoring
5. **Incremental updates** - Only re-render changed items

---

## References

- `vendor/zed/crates/gpui/examples/textured_surface.rs` - TexturedSurface example
- `vendor/zed/crates/gpui/src/elements/img.rs` - Image element
- `vendor/zed/crates/gpui/src/assets.rs` - RenderImage type
- `vendor/zed/crates/gpui/src/style.rs` - ObjectFit enum
- `research/textured-surface-workflow.md` - TexturedSurface analysis