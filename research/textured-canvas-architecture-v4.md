# Textured Canvas Architecture v4

## Overview

This document describes a **single-threaded** architecture for the infinite canvas with textured item rendering. Key insight: GPUI requires the main thread, so we embrace that constraint.

**Key Benefits:**
1. **Closures just work** - No thread crossing, no serialization
2. **Simpler architecture** - No channels, no synchronization
3. **Layout-based sizing** - Elements size themselves naturally
4. **Clean API** - `provider.add_item("id", || div().child("content"))`

---

## Core Insight: Single Thread

GPUI enforces main thread execution:
```rust
assert!(
    executor.is_main_thread(),
    "must construct App on main thread"
);
```

Instead of fighting this, we work with it:
- Texture rendering happens on the main thread
- Throttle to avoid UI stalls (one texture per frame)
- Use `cx.spawn()` for async-style work without blocking
- Cache aggressively

---

## API Design

### Adding Items

```rust
// Simple - element sizes itself via layout
provider.add_item("card-1", || {
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

// With explicit position (origin only, size from layout)
provider.add_item_at("card-2", point(px(300.0), px(100.0)), || {
    div()
        .p_4()
        .bg(rgb(0xe74c3c))
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_2xl().child("Title"))
        .child(div().text_sm().child("Description text"))
});

// Complex content with any GPUI elements
provider.add_item("card-3", || {
    div()
        .p_4()
        .bg(rgb(0x2ecc71))
        .child(img("path/to/image.png"))
        .child(svg("icons/star.svg"))
        .child(
            div()
                .flex()
                .gap_1()
                .children((0..5).map(|i| {
                    div().px_2().py_1().bg(rgb(0x333333)).rounded().child(format!("Tag {}", i))
                }))
        )
});
```

### Managing Items

```rust
// Update position
provider.set_position("card-1", point(px(200.0), px(150.0)));

// Get bounds (after layout/render)
if let Some(bounds) = provider.bounds("card-1") {
    println!("Card is at {:?} with size {:?}", bounds.origin, bounds.size);
}

// Remove item
provider.remove_item("card-2");

// Invalidate (force re-render)
provider.invalidate("card-1");
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Main Thread (only thread)                           │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                         InfiniteCanvas                                 │ │
│  │                                                                        │ │
│  │   - Owns camera (pan/zoom)                                            │ │
│  │   - Calls provider.items() to get what to display                     │ │
│  │   - Calls provider.texture(id) to get textures                        │ │
│  │   - Renders items as img() elements with scaling                      │ │
│  │                                                                        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    │ trait CanvasItemsProvider              │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    TexturedCanvasItemsProvider                         │ │
│  │                                                                        │ │
│  │   items: HashMap<String, ItemDefinition>                              │ │
│  │       └── factory: Box<dyn Fn() -> AnyElement>  ← closures work!      │ │
│  │       └── origin: Point<Pixels>                                       │ │
│  │       └── size: Option<Size<Pixels>>  ← from layout                   │ │
│  │                                                                        │ │
│  │   textures: HashMap<String, TextureState>                             │ │
│  │   render_queue: VecDeque<String>  ← throttled rendering               │ │
│  │                                                                        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                        │
│                                    │ when texture requested                 │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    Texture Rendering (same thread)                     │ │
│  │                                                                        │ │
│  │   1. Call factory() to get AnyElement                                 │ │
│  │   2. Create TexturedSurfaceWindow at large size                       │ │
│  │   3. layout_as_root(MinContent) → get natural size                    │ │
│  │   4. Resize window to measured size                                   │ │
│  │   5. draw_and_present() → render to texture                           │ │
│  │   6. read_pixels() → get BGRA data                                    │ │
│  │   7. Convert to RenderImage, cache it                                 │ │
│  │                                                                        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Types

```rust
/// String-based identifier for canvas items
pub type CanvasItemId = String;

/// Description of a canvas item for display
#[derive(Clone)]
pub struct CanvasItemDescriptor {
    pub id: CanvasItemId,
    pub bounds: Bounds<Pixels>,
    pub z_index: u32,
}

/// State of a texture
#[derive(Clone)]
pub enum TextureState {
    /// Not yet requested
    NotRequested,
    
    /// Queued for rendering
    Queued,
    
    /// Ready to display
    Ready {
        image: Arc<RenderImage>,
        size: Size<Pixels>,
    },
    
    /// Rendering failed
    Failed(String),
}

/// Internal item definition
struct ItemDefinition {
    id: CanvasItemId,
    origin: Point<Pixels>,
    size: Option<Size<Pixels>>,
    z_index: u32,
    factory: Box<dyn Fn() -> AnyElement>,
}
```

---

## Provider Trait

```rust
pub trait CanvasItemsProvider {
    /// Get all items to display
    fn items(&self) -> Vec<CanvasItemDescriptor>;
    
    /// Get texture state for an item
    fn texture_state(&self, id: &str) -> TextureState;
    
    /// Request texture rendering (queues it)
    fn request_texture(&mut self, id: &str);
    
    /// Process render queue (call once per frame)
    /// Returns true if any work was done
    fn tick(&mut self, cx: &mut App) -> bool;
}
```

---

## Provider Implementation

```rust
use std::collections::{HashMap, VecDeque};
use gpui::*;

pub struct TexturedCanvasItemsProvider {
    items: HashMap<CanvasItemId, ItemDefinition>,
    textures: HashMap<CanvasItemId, TextureState>,
    render_queue: VecDeque<CanvasItemId>,
    
    /// Max textures to render per tick (throttling)
    renders_per_tick: usize,
}

impl TexturedCanvasItemsProvider {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            textures: HashMap::new(),
            render_queue: VecDeque::new(),
            renders_per_tick: 1,  // Conservative default
        }
    }
    
    /// Add an item (auto-positioned at origin)
    pub fn add_item<F>(&mut self, id: impl Into<String>, factory: F)
    where
        F: Fn() -> AnyElement + 'static,
    {
        self.add_item_at(id, point(px(0.0), px(0.0)), factory);
    }
    
    /// Add an item at a specific position
    pub fn add_item_at<F>(
        &mut self,
        id: impl Into<String>,
        origin: Point<Pixels>,
        factory: F,
    )
    where
        F: Fn() -> AnyElement + 'static,
    {
        let id = id.into();
        
        self.items.insert(id.clone(), ItemDefinition {
            id: id.clone(),
            origin,
            size: None,
            z_index: 0,
            factory: Box::new(factory),
        });
        
        self.textures.insert(id, TextureState::NotRequested);
    }
    
    /// Set item position
    pub fn set_position(&mut self, id: &str, origin: Point<Pixels>) {
        if let Some(item) = self.items.get_mut(id) {
            item.origin = origin;
        }
    }
    
    /// Get item bounds (None if not yet measured)
    pub fn bounds(&self, id: &str) -> Option<Bounds<Pixels>> {
        self.items.get(id).and_then(|item| {
            item.size.map(|size| Bounds::new(item.origin, size))
        })
    }
    
    /// Remove an item
    pub fn remove_item(&mut self, id: &str) {
        self.items.remove(id);
        self.textures.remove(id);
        self.render_queue.retain(|i| i != id);
    }
    
    /// Invalidate an item (force re-render)
    pub fn invalidate(&mut self, id: &str) {
        if self.items.contains_key(id) {
            self.textures.insert(id.to_string(), TextureState::NotRequested);
        }
    }
    
    /// Set throttling (textures per frame)
    pub fn set_renders_per_tick(&mut self, count: usize) {
        self.renders_per_tick = count.max(1);
    }
}

impl CanvasItemsProvider for TexturedCanvasItemsProvider {
    fn items(&self) -> Vec<CanvasItemDescriptor> {
        self.items.values().map(|item| {
            let size = item.size.unwrap_or(size(px(100.0), px(100.0)));
            CanvasItemDescriptor {
                id: item.id.clone(),
                bounds: Bounds::new(item.origin, size),
                z_index: item.z_index,
            }
        }).collect()
    }
    
    fn texture_state(&self, id: &str) -> TextureState {
        self.textures.get(id).cloned().unwrap_or(TextureState::NotRequested)
    }
    
    fn request_texture(&mut self, id: &str) {
        if self.items.contains_key(id) {
            if let Some(TextureState::NotRequested) = self.textures.get(id) {
                self.textures.insert(id.to_string(), TextureState::Queued);
                self.render_queue.push_back(id.to_string());
            }
        }
    }
    
    fn tick(&mut self, cx: &mut App) -> bool {
        let mut work_done = false;
        
        for _ in 0..self.renders_per_tick {
            if let Some(id) = self.render_queue.pop_front() {
                if let Some(item) = self.items.get(&id) {
                    match render_item_texture(item, cx) {
                        Ok((image, size)) => {
                            // Update measured size
                            if let Some(item) = self.items.get_mut(&id) {
                                item.size = Some(size);
                            }
                            
                            self.textures.insert(
                                id,
                                TextureState::Ready { image, size },
                            );
                        }
                        Err(e) => {
                            self.textures.insert(id, TextureState::Failed(e));
                        }
                    }
                    work_done = true;
                }
            }
        }
        
        work_done
    }
}
```

---

## Texture Rendering

```rust
/// Render an item to a texture (blocking, but fast)
fn render_item_texture(
    item: &ItemDefinition,
    cx: &mut App,
) -> Result<(Arc<RenderImage>, Size<Pixels>), String> {
    // This only works on Linux with textured surface support
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        return Err("Textured rendering only supported on Linux".into());
    }
    
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        render_item_texture_linux(item, cx)
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn render_item_texture_linux(
    item: &ItemDefinition,
    _cx: &mut App,
) -> Result<(Arc<RenderImage>, Size<Pixels>), String> {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    // Capture results
    let result: Rc<RefCell<Option<Result<(Arc<RenderImage>, Size<Pixels>), String>>>> = 
        Rc::new(RefCell::new(None));
    let result_clone = result.clone();
    
    // Clone the factory output for the closure
    // We need to actually call the factory here and capture what we need
    let element_factory = &item.factory;
    
    // Create a temporary textured application
    // Note: This creates a new GPU context, which is not ideal but works for MVP
    let app = Application::textured();
    
    app.run(move |cx: &mut App| {
        // Phase 1: Measure
        let measure_bounds = Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(2000.0), px(2000.0)),
        );
        
        let window_result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(measure_bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| MeasureRenderView::new(element_factory))
            },
        );
        
        match window_result {
            Ok(window) => {
                let handle: AnyWindowHandle = window.into();
                let result_for_spawn = result_clone.clone();
                
                cx.spawn(async move |cx| {
                    // Draw to trigger layout
                    let _ = cx.update_window(handle, |_, window, cx| {
                        window.draw_and_present(cx)
                    });
                    
                    // Get measured size
                    let measured_size = cx.update_window(handle, |root, _window, cx| {
                        if let Ok(view) = root.downcast::<MeasureRenderView>() {
                            view.read(cx).measured_size
                        } else {
                            None
                        }
                    }).ok().flatten();
                    
                    // Close measure window
                    cx.update_window(handle, |_, _, cx| cx.remove_window()).ok();
                    
                    let Some(measured_size) = measured_size else {
                        *result_for_spawn.borrow_mut() = Some(Err("Failed to measure".into()));
                        cx.update(|cx| cx.quit()).ok();
                        return;
                    };
                    
                    // Phase 2: Render at measured size
                    let render_bounds = Bounds::new(
                        point(px(0.0), px(0.0)),
                        measured_size,
                    );
                    
                    let render_window = cx.update(|cx| {
                        cx.open_window(
                            WindowOptions {
                                window_bounds: Some(WindowBounds::Windowed(render_bounds)),
                                ..Default::default()
                            },
                            |_, cx| cx.new(|_| MeasureRenderView::new_for_render(element_factory)),
                        )
                    });
                    
                    if let Ok(Ok(window)) = render_window {
                        let handle: AnyWindowHandle = window.into();
                        
                        // Draw
                        let _ = cx.update_window(handle, |_, window, cx| {
                            window.draw_and_present(cx)
                        });
                        
                        // Read pixels
                        if let Ok(Some(pixels)) = cx.update_window(handle, |_, window, _| {
                            window.read_pixels()
                        }) {
                            let width = measured_size.width.0 as u32;
                            let height = measured_size.height.0 as u32;
                            
                            if let Some(image) = pixels_to_render_image(&pixels, width, height) {
                                *result_for_spawn.borrow_mut() = Some(Ok((
                                    Arc::new(image),
                                    measured_size,
                                )));
                            } else {
                                *result_for_spawn.borrow_mut() = Some(Err("Pixel conversion failed".into()));
                            }
                        } else {
                            *result_for_spawn.borrow_mut() = Some(Err("Failed to read pixels".into()));
                        }
                        
                        cx.update_window(handle, |_, _, cx| cx.remove_window()).ok();
                    } else {
                        *result_for_spawn.borrow_mut() = Some(Err("Failed to open render window".into()));
                    }
                    
                    cx.update(|cx| cx.quit()).ok();
                }).detach();
            }
            Err(e) => {
                *result_clone.borrow_mut() = Some(Err(format!("Failed to open window: {}", e)));
                cx.quit();
            }
        }
    });
    
    // Extract result after app.run() completes
    result.borrow_mut().take().unwrap_or(Err("No result".into()))
}

/// Convert BGRA pixels to RenderImage
fn pixels_to_render_image(pixels: &[u8], width: u32, height: u32) -> Option<RenderImage> {
    use image::{RgbaImage, Frame};
    use smallvec::smallvec;
    
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
    Some(RenderImage::new(smallvec![Frame::new(image)]))
}
```

---

## Measure/Render View

```rust
struct MeasureRenderView {
    element: Option<AnyElement>,
    measured_size: Option<Size<Pixels>>,
}

impl MeasureRenderView {
    fn new<F: Fn() -> AnyElement>(factory: &F) -> Self {
        Self {
            element: Some(factory()),
            measured_size: None,
        }
    }
    
    fn new_for_render<F: Fn() -> AnyElement>(factory: &F) -> Self {
        Self {
            element: Some(factory()),
            measured_size: None,  // Not needed for render phase
        }
    }
}

impl Render for MeasureRenderView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(mut element) = self.element.take() {
            // Measure using MinContent
            let size = element.layout_as_root(
                AvailableSpace::min_size(),
                window,
                cx,
            );
            self.measured_size = Some(size);
            
            // Return the element for rendering
            element
        } else {
            div().size_full()
        }
    }
}
```

---

## InfiniteCanvas Integration

```rust
pub struct InfiniteCanvas<P: CanvasItemsProvider> {
    provider: P,
    camera: Camera,
}

#[derive(Default)]
pub struct Camera {
    pub offset: Point<Pixels>,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            offset: point(px(0.0), px(0.0)),
            zoom: 1.0,
        }
    }
    
    pub fn world_to_screen(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        Bounds::new(
            point(
                (bounds.origin.x + self.offset.x) * self.zoom,
                (bounds.origin.y + self.offset.y) * self.zoom,
            ),
            size(
                bounds.size.width * self.zoom,
                bounds.size.height * self.zoom,
            ),
        )
    }
}

impl<P: CanvasItemsProvider + 'static> InfiniteCanvas<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            camera: Camera::new(),
        }
    }
    
    pub fn provider(&self) -> &P {
        &self.provider
    }
    
    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.provider
    }
    
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

impl<P: CanvasItemsProvider + 'static> Render for InfiniteCanvas<P> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process render queue (throttled)
        // Note: Need to get App from somewhere - this is a simplification
        // In practice, you'd use cx.update() or similar
        
        let viewport = window.viewport_size();
        let items = self.provider.items();
        
        div()
            .size_full()
            .overflow_hidden()
            .bg(rgb(0xf5f5f5))
            .children(
                items.iter().filter_map(|item| {
                    // Transform to screen space
                    let screen_bounds = self.camera.world_to_screen(item.bounds);
                    
                    // Culling
                    if !bounds_intersect(&screen_bounds, &Bounds::new(point(px(0.0), px(0.0)), viewport)) {
                        return None;
                    }
                    
                    // Get or request texture
                    let content = match self.provider.texture_state(&item.id) {
                        TextureState::NotRequested => {
                            self.provider.request_texture(&item.id);
                            placeholder_element("Loading...")
                        }
                        TextureState::Queued => {
                            placeholder_element("Queued...")
                        }
                        TextureState::Ready { ref image, .. } => {
                            img(image.clone())
                                .size_full()
                                .object_fit(ObjectFit::Fill)
                                .into_any_element()
                        }
                        TextureState::Failed(ref msg) => {
                            error_element(msg)
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

fn placeholder_element(msg: &str) -> AnyElement {
    div()
        .size_full()
        .bg(rgb(0xe0e0e0))
        .flex()
        .justify_center()
        .items_center()
        .child(msg.to_string())
        .into_any_element()
}

fn error_element(msg: &str) -> AnyElement {
    div()
        .size_full()
        .bg(rgb(0xffcccc))
        .flex()
        .justify_center()
        .items_center()
        .text_color(rgb(0xcc0000))
        .child(format!("Error: {}", msg))
        .into_any_element()
}

fn bounds_intersect(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> bool {
    a.origin.x < b.origin.x + b.size.width
        && a.origin.x + a.size.width > b.origin.x
        && a.origin.y < b.origin.y + b.size.height
        && a.origin.y + a.size.height > b.origin.y
}
```

---

## Usage Example

```rust
fn main() {
    Application::new()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            // Create provider
            let mut provider = TexturedCanvasItemsProvider::new();
            
            // Add items with closures - they just work!
            provider.add_item("hello", || {
                div()
                    .p_4()
                    .bg(rgb(0x3498db))
                    .rounded_lg()
                    .shadow_lg()
                    .child(
                        div()
                            .text_xl()
                            .text_color(white())
                            .child("Hello World")
                    )
            });
            
            provider.add_item_at("complex", point(px(250.0), px(50.0)), || {
                div()
                    .p_4()
                    .bg(rgb(0xe74c3c))
                    .rounded_lg()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().text_2xl().text_color(white()).child("Complex Card"))
                    .child(div().text_sm().text_color(rgb(0xffcccc)).child("With multiple children"))
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .children((1..=3).map(|i| {
                                div()
                                    .px_2()
                                    .py_1()
                                    .bg(rgb(0xffffff))
                                    .rounded()
                                    .text_xs()
                                    .child(format!("Tag {}", i))
                            }))
                    )
            });
            
            // Create canvas
            let canvas = InfiniteCanvas::new(provider);
            
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(
                        Bounds::centered(None, size(px(1200.0), px(800.0)), cx)
                    )),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| canvas),
            )
            .unwrap();
        });
}
```

---

## Throttling Strategy

To avoid UI stalls, we throttle texture rendering:

```rust
// In the main render loop or a timer:
fn update(&mut self, cx: &mut Context<Self>) {
    // Process one texture per frame
    if self.provider.tick(cx.app_mut()) {
        // More work pending, request another frame
        cx.notify();
    }
}
```

**Throttling options:**
1. **Per-frame limit** - `renders_per_tick = 1` (default, safest)
2. **Time budget** - Render until X ms have passed
3. **Priority queue** - Visible items first

---

## Performance Considerations

### Costs
1. **GPU context per render** - `Application::textured()` creates new GPU context
2. **Blocking call** - `app.run()` blocks until complete
3. **Memory** - Each texture stored as RenderImage

### Mitigations
1. **Aggressive caching** - Only render once per item
2. **Throttling** - One texture per frame
3. **Culling** - Don't request textures for off-screen items
4. **LRU eviction** - Drop textures for items not viewed recently

### Future Optimizations
1. **Shared GPU context** - Reuse renderer's GPU context
2. **Texture pooling** - Reuse texture memory
3. **LOD** - Lower resolution textures when zoomed out
4. **Incremental rendering** - Partial updates

---

## Limitations

1. **Linux only** - TexturedSurface requires Linux/FreeBSD
2. **Blocking renders** - Brief UI stalls during texture creation
3. **Memory usage** - Each item needs its own texture
4. **No animation** - Textures are static snapshots

---

## Dependencies

```toml
[dependencies]
gpui = "0.2.2"
image = "0.25"
smallvec = "1"
```

No `flume` needed - single threaded!

---

## Summary

This v4 architecture embraces GPUI's single-threaded nature:

| Aspect | Approach |
|--------|----------|
| Threading | Single (main) thread |
| Closures | Work directly, no serialization |
| Sizing | Layout-based via `layout_as_root()` |
| Throttling | One texture per frame |
| API | `provider.add_item("id", \|\| div().child(...))` |

The key insight is that fighting GPUI's threading model adds complexity without benefit. By staying single-threaded, closures just work, and the architecture becomes much simpler.