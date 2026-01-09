# Textured Canvas Architecture v3

## Overview

This document describes a provider-based architecture for the infinite canvas with textured item rendering. Key improvements in v3:

1. **Layout-based sizing** - Element sizes are determined by GPUI layout, not declared upfront
2. **Cleaner API** - `provider.add_item("id", || div().child("content"))`
3. **Two-phase rendering** - Layout pass to get size, then render at that size

---

## Core Problem: Texture Sizing

GPUI layout happens inside the render lifecycle:
1. `request_layout()` - element requests space from Taffy
2. `prepaint()` - bounds committed  
3. `paint()` - actual drawing

We need to know the size to create the texture, but size comes from layout.

### Solution: Two-Phase Rendering

1. **Phase 1: Layout** - Create a window with `AvailableSpace::MinContent`, let element layout itself, capture the resulting size
2. **Phase 2: Render** - Create a texture at the measured size, render the element

GPUI provides `AnyElement::layout_as_root(available_space, window, cx) -> Size<Pixels>` which does exactly what we need for Phase 1.

---

## API Design

### Clean Add Item API

```rust
// Simple text box
provider.add_item("item-1", || {
    div()
        .p_4()
        .bg(rgb(0x3498db))
        .rounded_lg()
        .child(
            div()
                .text_xl()
                .text_color(white())
                .child("Hello World")
        )
});

// Complex content
provider.add_item("item-2", || {
    div()
        .p_4()
        .bg(rgb(0xe74c3c))
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_2xl().child("Title"))
        .child(div().text_sm().child("Some description text"))
        .child(
            div()
                .flex()
                .gap_1()
                .child(div().px_2().py_1().bg(rgb(0x333333)).rounded().child("Tag 1"))
                .child(div().px_2().py_1().bg(rgb(0x333333)).rounded().child("Tag 2"))
        )
});

// With explicit position (origin only, size from layout)
provider.add_item_at("item-3", point(px(500.0), px(200.0)), || {
    div().p_4().bg(rgb(0x2ecc71)).child("Positioned item")
});
```

### Position Management

Items can be:
1. **Auto-positioned** - Provider determines layout (grid, force-directed, etc.)
2. **Explicitly positioned** - User specifies origin, size from layout

```rust
// Set position after creation
provider.set_position("item-1", point(px(100.0), px(100.0)));

// Get current bounds (after layout)
let bounds = provider.bounds("item-1"); // Option<Bounds<Pixels>>
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Main Application                                │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                         InfiniteCanvas                                 │ │
│  │   - Owns camera (pan/zoom)                                            │ │
│  │   - Calls provider.items() to get what to display                     │ │
│  │   - Calls provider.texture(id) to get textures                        │ │
│  │   - Renders items as img() elements with scaling                      │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    │ trait CanvasItemsProvider              │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    TexturedCanvasItemsProvider                         │ │
│  │   - Stores item factories: HashMap<String, Box<dyn Fn() -> AnyElement>>│ │
│  │   - Manages positions and computed sizes                              │ │
│  │   - Owns TextureRenderer (background thread)                          │ │
│  │   - Manages texture cache                                             │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    │ flume channels                         │
│                                    ▼                                        │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
┌────────────────────────────────────┼────────────────────────────────────────┐
│                    Texture Renderer Thread                                  │
│                    (Application::textured())                                │
│                                                                             │
│   Phase 1: Layout                                                          │
│   - Create window with large available space                               │
│   - Call element.layout_as_root(AvailableSpace::MinContent)               │
│   - Capture resulting size                                                 │
│                                                                             │
│   Phase 2: Render                                                          │
│   - Resize window to measured size (or create new one)                    │
│   - Render element to texture                                              │
│   - Read pixels, send back as RenderImage                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Types

### Item ID

```rust
/// String-based identifier for canvas items
pub type CanvasItemId = String;
```

### Item Descriptor

```rust
/// Description of a canvas item for display
pub struct CanvasItemDescriptor {
    /// Unique identifier
    pub id: CanvasItemId,
    
    /// Position and size in canvas world coordinates
    /// Size is determined by layout, origin by user or auto-layout
    pub bounds: Bounds<Pixels>,
    
    /// Z-order for layering
    pub z_index: u32,
}
```

### Texture State

```rust
pub enum TextureState {
    /// Not yet requested
    NotRequested,
    
    /// Layout in progress (measuring size)
    Measuring,
    
    /// Rendering in progress (size known)
    Rendering { size: Size<Pixels> },
    
    /// Ready to display
    Ready {
        image: Arc<RenderImage>,
        size: Size<Pixels>,
    },
    
    /// Failed
    Failed(String),
}
```

### Render Messages

```rust
/// Request to render an item
struct RenderRequest {
    id: CanvasItemId,
    /// Serialized element factory (see below)
    content: SerializedContent,
}

/// Response with measured size
struct MeasureResponse {
    id: CanvasItemId,
    size: Size<Pixels>,
}

/// Response with rendered pixels
struct RenderResponse {
    id: CanvasItemId,
    result: Result<(Arc<RenderImage>, Size<Pixels>), String>,
}
```

---

## Content Serialization Problem

The element factory `|| div().child("text")` contains closures and non-Send types. We need to get this to the renderer thread.

### Option A: Serialize Content Description

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum SerializedContent {
    TextBox {
        text: String,
        background: u32,
        padding: f32,
        // ... other style props
    },
    Container {
        background: u32,
        children: Vec<SerializedContent>,
        // ...
    },
    // Extensible for custom types
    Custom {
        type_id: String,
        json: String,
    },
}

// Renderer side reconstructs element from description
fn build_element(content: &SerializedContent) -> AnyElement {
    match content {
        SerializedContent::TextBox { text, background, padding } => {
            div()
                .p(*padding)
                .bg(rgb(*background))
                .child(text.clone())
                .into_any_element()
        }
        // ...
    }
}
```

### Option B: Registered Factories (Preferred for MVP)

Keep factories on the main thread, only send IDs. Renderer requests the element when needed.

```rust
// Main thread
struct ItemDefinition {
    id: CanvasItemId,
    origin: Point<Pixels>,
    z_index: u32,
    // Factory stays on main thread
    factory: Box<dyn Fn() -> AnyElement>,
}

// For renderer thread, we need a serializable representation
// The factory is called on main thread, element is "described" to renderer
```

### Option C: Element Snapshot (Best for Full GPUI Support)

Main thread builds the element, serializes its "description" to renderer:

```rust
// This would require GPUI support for element serialization
// Not available today, but could be added
```

### Pragmatic Solution for MVP

Use a **hybrid approach**:
1. Simple content types (text boxes, shapes) use `SerializedContent`
2. Complex content that needs full GPUI renders on main thread, cached as image data

```rust
pub enum ItemContent {
    /// Simple content that can be serialized
    Simple(SerializedContent),
    
    /// Pre-rendered image data (for complex content)
    Prerendered(Arc<RenderImage>),
}
```

---

## Provider Trait

```rust
pub trait CanvasItemsProvider {
    /// Get all items to display
    fn items(&self) -> &[CanvasItemDescriptor];
    
    /// Get texture state for an item
    fn texture_state(&self, id: &str) -> TextureState;
    
    /// Request texture rendering
    fn request_texture(&mut self, id: &str);
    
    /// Housekeeping each frame
    fn tick(&mut self);
}
```

---

## Textured Provider Implementation

```rust
pub struct TexturedCanvasItemsProvider {
    /// Item definitions
    items: HashMap<CanvasItemId, ItemDefinition>,
    
    /// Computed descriptors (cached for items() return)
    descriptors: Vec<CanvasItemDescriptor>,
    descriptors_dirty: bool,
    
    /// Texture states
    textures: HashMap<CanvasItemId, TextureState>,
    
    /// Communication with renderer
    request_tx: Sender<RenderRequest>,
    response_rx: Receiver<RenderResponse>,
}

struct ItemDefinition {
    id: CanvasItemId,
    origin: Point<Pixels>,
    size: Option<Size<Pixels>>,  // None until measured
    z_index: u32,
    content: SerializedContent,
}

impl TexturedCanvasItemsProvider {
    pub fn new() -> Self {
        let (request_tx, request_rx) = flume::unbounded();
        let (response_tx, response_rx) = flume::unbounded();
        
        // Spawn renderer thread
        std::thread::spawn(move || {
            run_texture_renderer(request_rx, response_tx);
        });
        
        Self {
            items: HashMap::new(),
            descriptors: Vec::new(),
            descriptors_dirty: false,
            textures: HashMap::new(),
            request_tx,
            response_rx,
        }
    }
    
    /// Add an item with auto-positioned origin
    pub fn add_item<F>(&mut self, id: impl Into<String>, content_fn: F)
    where
        F: FnOnce() -> SerializedContent,
    {
        let id = id.into();
        let content = content_fn();
        
        self.items.insert(id.clone(), ItemDefinition {
            id: id.clone(),
            origin: point(px(0.0), px(0.0)),  // Auto-layout will position
            size: None,
            z_index: 0,
            content,
        });
        
        self.textures.insert(id, TextureState::NotRequested);
        self.descriptors_dirty = true;
    }
    
    /// Add an item at a specific position
    pub fn add_item_at<F>(
        &mut self,
        id: impl Into<String>,
        origin: Point<Pixels>,
        content_fn: F,
    )
    where
        F: FnOnce() -> SerializedContent,
    {
        let id = id.into();
        let content = content_fn();
        
        self.items.insert(id.clone(), ItemDefinition {
            id: id.clone(),
            origin,
            size: None,
            z_index: 0,
            content,
        });
        
        self.textures.insert(id.clone(), TextureState::NotRequested);
        self.descriptors_dirty = true;
    }
    
    /// Set item position
    pub fn set_position(&mut self, id: &str, origin: Point<Pixels>) {
        if let Some(item) = self.items.get_mut(id) {
            item.origin = origin;
            self.descriptors_dirty = true;
        }
    }
    
    /// Get item bounds (if measured)
    pub fn bounds(&self, id: &str) -> Option<Bounds<Pixels>> {
        self.items.get(id).and_then(|item| {
            item.size.map(|size| Bounds::new(item.origin, size))
        })
    }
    
    /// Remove an item
    pub fn remove_item(&mut self, id: &str) {
        self.items.remove(id);
        self.textures.remove(id);
        self.descriptors_dirty = true;
    }
    
    fn rebuild_descriptors(&mut self) {
        self.descriptors.clear();
        for item in self.items.values() {
            let size = item.size.unwrap_or(size(px(100.0), px(100.0)));  // Placeholder
            self.descriptors.push(CanvasItemDescriptor {
                id: item.id.clone(),
                bounds: Bounds::new(item.origin, size),
                z_index: item.z_index,
            });
        }
        self.descriptors_dirty = false;
    }
}

impl CanvasItemsProvider for TexturedCanvasItemsProvider {
    fn items(&self) -> &[CanvasItemDescriptor] {
        // Note: Would need interior mutability for lazy rebuild
        &self.descriptors
    }
    
    fn texture_state(&self, id: &str) -> TextureState {
        self.textures.get(id).cloned().unwrap_or(TextureState::NotRequested)
    }
    
    fn request_texture(&mut self, id: &str) {
        if let Some(item) = self.items.get(id) {
            self.request_tx.send(RenderRequest {
                id: id.to_string(),
                content: item.content.clone(),
            }).ok();
            
            self.textures.insert(id.to_string(), TextureState::Measuring);
        }
    }
    
    fn tick(&mut self) {
        // Poll for responses
        while let Ok(response) = self.response_rx.try_recv() {
            match response.result {
                Ok((image, size)) => {
                    // Update size
                    if let Some(item) = self.items.get_mut(&response.id) {
                        item.size = Some(size);
                        self.descriptors_dirty = true;
                    }
                    
                    self.textures.insert(
                        response.id,
                        TextureState::Ready { image, size },
                    );
                }
                Err(e) => {
                    self.textures.insert(response.id, TextureState::Failed(e));
                }
            }
        }
        
        if self.descriptors_dirty {
            self.rebuild_descriptors();
        }
    }
}
```

---

## Texture Renderer Thread (Two-Phase)

```rust
fn run_texture_renderer(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
) {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use gpui::{Application, App};
        
        let app = Application::textured();
        
        app.run(move |cx: &mut App| {
            process_requests_two_phase(request_rx, response_tx, cx);
        });
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn process_requests_two_phase(
    request_rx: Receiver<RenderRequest>,
    response_tx: Sender<RenderResponse>,
    cx: &mut App,
) {
    use gpui::{
        WindowOptions, WindowBounds, Bounds, AvailableSpace,
        point, px, size, AnyWindowHandle,
    };
    
    while let Ok(request) = request_rx.recv() {
        let id = request.id.clone();
        let response_tx = response_tx.clone();
        
        // Phase 1: Measure
        // Create a large window and let element size itself
        let measure_bounds = Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(2000.0), px(2000.0)),  // Large available space
        );
        
        let content = request.content.clone();
        
        let window_result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(measure_bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| MeasureView { content: content.clone() }),
        );
        
        match window_result {
            Ok(window) => {
                let handle: AnyWindowHandle = window.into();
                
                cx.spawn(async move |cx| {
                    // Get the measured size from the view
                    let size_result = cx.update_window(handle, |root, window, cx| {
                        // The view should have computed its size during layout
                        if let Ok(view) = root.downcast::<MeasureView>() {
                            view.read(cx).measured_size
                        } else {
                            None
                        }
                    });
                    
                    // Close measure window
                    cx.update_window(handle, |_, _, cx| cx.remove_window()).ok();
                    
                    let measured_size = match size_result {
                        Ok(Some(size)) => size,
                        _ => {
                            response_tx.send(RenderResponse {
                                id,
                                result: Err("Failed to measure element".into()),
                            }).ok();
                            return;
                        }
                    };
                    
                    // Phase 2: Render at measured size
                    let render_bounds = Bounds::new(
                        point(px(0.0), px(0.0)),
                        measured_size,
                    );
                    
                    let render_result = cx.update(|cx| {
                        let window = cx.open_window(
                            WindowOptions {
                                window_bounds: Some(WindowBounds::Windowed(render_bounds)),
                                ..Default::default()
                            },
                            |_, cx| cx.new(|_| RenderView { content }),
                        );
                        window
                    });
                    
                    match render_result {
                        Ok(Ok(window)) => {
                            let handle: AnyWindowHandle = window.into();
                            
                            // Draw and capture
                            let draw_ok = cx.update_window(handle, |_, window, cx| {
                                window.draw_and_present(cx)
                            }).unwrap_or(Ok(false)).unwrap_or(false);
                            
                            if draw_ok {
                                if let Ok(Some(pixels)) = cx.update_window(handle, |_, window, _| {
                                    window.read_pixels()
                                }) {
                                    let width = measured_size.width.0 as u32;
                                    let height = measured_size.height.0 as u32;
                                    
                                    if let Some(image) = pixels_to_render_image(&pixels, width, height) {
                                        response_tx.send(RenderResponse {
                                            id,
                                            result: Ok((Arc::new(image), measured_size)),
                                        }).ok();
                                    } else {
                                        response_tx.send(RenderResponse {
                                            id,
                                            result: Err("Failed to convert pixels".into()),
                                        }).ok();
                                    }
                                }
                            }
                            
                            cx.update_window(handle, |_, _, cx| cx.remove_window()).ok();
                        }
                        _ => {
                            response_tx.send(RenderResponse {
                                id,
                                result: Err("Failed to open render window".into()),
                            }).ok();
                        }
                    }
                }).detach();
            }
            Err(e) => {
                response_tx.send(RenderResponse {
                    id: request.id,
                    result: Err(format!("Failed to open measure window: {}", e)),
                }).ok();
            }
        }
    }
}
```

---

## Measure View

```rust
struct MeasureView {
    content: SerializedContent,
    measured_size: Option<Size<Pixels>>,
}

impl Render for MeasureView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Build element and let it size itself
        let mut element = build_element(&self.content);
        
        // Use layout_as_root to get natural size
        let size = element.layout_as_root(
            AvailableSpace::min_size(),  // MinContent for both dimensions
            window,
            cx,
        );
        
        self.measured_size = Some(size);
        
        // Return a placeholder - we don't actually render this
        div().size_full()
    }
}
```

---

## Render View

```rust
struct RenderView {
    content: SerializedContent,
}

impl Render for RenderView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        build_element(&self.content)
    }
}
```

---

## Build Element from SerializedContent

```rust
fn build_element(content: &SerializedContent) -> AnyElement {
    match content {
        SerializedContent::TextBox { text, background, text_color, padding } => {
            div()
                .p(*padding)
                .bg(rgb(*background))
                .child(
                    div()
                        .text_color(rgb(*text_color))
                        .child(text.clone())
                )
                .into_any_element()
        }
        
        SerializedContent::Container { 
            background, 
            padding, 
            gap,
            direction,
            children 
        } => {
            let mut container = div()
                .p(*padding)
                .bg(rgb(*background))
                .flex()
                .gap(*gap);
            
            container = match direction {
                Direction::Row => container.flex_row(),
                Direction::Column => container.flex_col(),
            };
            
            for child in children {
                container = container.child(build_element(child));
            }
            
            container.into_any_element()
        }
        
        SerializedContent::Custom { type_id, json } => {
            // Placeholder for custom types
            div()
                .p_2()
                .bg(rgb(0xff0000))
                .child(format!("Custom: {}", type_id))
                .into_any_element()
        }
    }
}
```

---

## Usage Example

```rust
fn main() {
    let app = Application::new();
    
    app.run(|cx| {
        let mut provider = TexturedCanvasItemsProvider::new();
        
        // Simple API - content sizes itself
        provider.add_item("card-1", || SerializedContent::TextBox {
            text: "Hello World".into(),
            background: 0x3498db,
            text_color: 0xffffff,
            padding: 16.0,
        });
        
        // With position
        provider.add_item_at("card-2", point(px(300.0), px(100.0)), || {
            SerializedContent::Container {
                background: 0xe74c3c,
                padding: 16.0,
                gap: 8.0,
                direction: Direction::Column,
                children: vec![
                    SerializedContent::TextBox {
                        text: "Title".into(),
                        background: 0x00000000,  // Transparent
                        text_color: 0xffffff,
                        padding: 0.0,
                    },
                    SerializedContent::TextBox {
                        text: "Description goes here".into(),
                        background: 0x00000000,
                        text_color: 0xcccccc,
                        padding: 0.0,
                    },
                ],
            }
        });
        
        let canvas = InfiniteCanvas::new(provider);
        
        cx.open_window(
            WindowOptions::default(),
            |_, cx| cx.new(|_| canvas),
        );
    });
}
```

---

## Key Changes from v2

1. **Layout-based sizing** - Elements size themselves via `layout_as_root(AvailableSpace::MinContent)`
2. **Two-phase rendering** - Measure first, then render at measured size
3. **String IDs** - More ergonomic than opaque handles
4. **Cleaner API** - `add_item(id, || content)` instead of explicit bounds
5. **SerializedContent** - Structured content description instead of raw closures

---

## Limitations

1. **SerializedContent** - Limited to what we can describe/serialize
2. **Two window opens per item** - Measure + Render (could optimize with window reuse)
3. **No full closure support** - Can't use arbitrary GPUI elements directly

---

## Future Work

1. **Element serialization in GPUI** - Would enable arbitrary elements
2. **Window pooling** - Reuse windows for measure/render
3. **Incremental layout** - Cache sizes, only re-measure on content change
4. **Max-width constraints** - `add_item_with_max_width(id, max_width, content)`

---

## Dependencies

```toml
[dependencies]
gpui = "0.2.2"
flume = "0.11"
image = "0.25"
smallvec = "1"
serde = { version = "1.0", features = ["derive"] }
```
