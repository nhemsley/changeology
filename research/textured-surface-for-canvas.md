# Using TexturedSurface for Infinite Canvas Rendering

## Overview

This document explores how the TexturedSurface implementation from the `gpui-oneshot-render` branch (commit 156a053) could be adapted or reused to enable render-to-texture capabilities for the infinite canvas component.

## What TexturedSurface Does

The TexturedSurface implementation provides a headless GPUI rendering platform that:

1. **Renders GPUI content to a GPU texture** instead of a display window
2. **Supports full GPUI element rendering** - divs, text, shadows, paths, sprites
3. **Allows pixel readback** - can extract rendered pixels as `Vec<u8>`
4. **Works without a display server** - useful for server-side rendering, testing, and offscreen rendering

### Key Components

```
crates/gpui/src/platform/linux/textured_surface/
├── mod.rs           # Module exports
├── client.rs        # TexturedSurfaceClient - platform client implementation
├── display.rs       # TexturedSurfaceDisplay - virtual display
└── window.rs        # TexturedSurfaceWindow - the core rendering logic (~1000 lines)
```

### Architecture

```
Application::textured()
    └── textured_platform()
        └── TexturedSurfaceClient
            └── open_window()
                └── TexturedSurfaceWindow
                    ├── GPU Context (blade-graphics)
                    ├── Render Target Texture
                    ├── Atlas for sprites/glyphs
                    └── Rendering Pipelines
```

## Key Implementation Details

### 1. GPU Initialization (No Display Required)

```rust
let gpu = Arc::new(
    unsafe {
        gpu::Context::init(gpu::ContextDesc {
            presentation: false,  // Key: no presentation/display needed
            validation: cfg!(debug_assertions),
            ..Default::default()
        })
    }
)?;
```

### 2. Render Target Creation

```rust
let render_target = gpu.create_texture(gpu::TextureDesc {
    name: "textured_surface_target",
    format: gpu::TextureFormat::Bgra8UnormSrgb,
    size: gpu::Extent { width, height, depth: 1 },
    usage: gpu::TextureUsage::TARGET | gpu::TextureUsage::COPY,
    // ...
});
```

### 3. Scene Rendering

The `render_scene_to_texture` method processes GPUI's `Scene` primitives:
- Quads (rectangles, backgrounds)
- Shadows
- Paths (vector shapes)
- Underlines
- MonochromeSprites (glyphs)
- PolychromeSprites (images)

### 4. Pixel Readback

```rust
fn read_pixels_to_buffer(&mut self) {
    // Create staging buffer in shared memory
    let staging = self.gpu.create_buffer(gpu::BufferDesc {
        memory: gpu::Memory::Shared,
        // ...
    });
    
    // Copy texture to buffer
    transfer.copy_texture_to_buffer(/* ... */);
    
    // Read pixels
    let pixels = unsafe {
        std::slice::from_raw_parts(staging.data() as *const u8, buffer_size).to_vec()
    };
    
    self.rendered_pixels = Some(pixels);
}
```

## How This Could Work for Infinite Canvas

### Option A: Offscreen Rendering Per Item (Full Reuse)

Use TexturedSurface to render each canvas item to a texture, then composite on the main canvas.

```rust
struct CanvasItemTexture {
    texture: gpu::Texture,
    texture_view: gpu::TextureView,
    size: Size<Pixels>,
    needs_redraw: bool,
}

impl InfiniteCanvas {
    fn render_item_to_texture(
        &mut self,
        item: &CanvasItem,
        textured_window: &mut TexturedSurfaceWindow,
        window: &mut Window,
        cx: &mut App,
    ) -> CanvasItemTexture {
        // 1. Create/update the item's view
        // 2. Draw to textured surface
        textured_window.draw_and_present(cx);
        
        // 3. Get the rendered texture
        // (Would need to expose texture directly, not just pixels)
    }
}
```

**Pros:**
- Items render at native resolution
- Can cache textures (only re-render when item changes)
- Zoom = scale the texture (fast)

**Cons:**
- Significant memory usage (one texture per item)
- Complex texture management
- May need to modify TexturedSurface to expose textures directly

### Option B: Render-to-Texture Element (New Abstraction)

Create a new GPUI element that renders its children to a texture, then draws that texture scaled.

```rust
pub struct RenderToTexture {
    children: SmallVec<[AnyElement; 1]>,
    texture_size: Size<Pixels>,  // Native render size
    display_bounds: Bounds<Pixels>,  // Actual display bounds (scaled)
}

impl Element for RenderToTexture {
    fn paint(&mut self, ...) {
        // 1. Render children to offscreen texture at texture_size
        // 2. Draw texture to display_bounds (with scaling)
    }
}
```

**Usage in Canvas:**
```rust
fn render_item(&self, item: &CanvasItem, zoom: f32) -> impl IntoElement {
    let native_size = item.bounds.size;
    let screen_size = size(native_size.width * zoom, native_size.height * zoom);
    
    render_to_texture(native_size)
        .display_at(screen_bounds)
        .child(item_content(item))
}
```

### Option C: Lazy Texture Caching (Hybrid)

Only use textures for items below a certain zoom threshold.

```rust
fn render_item(&self, item: &CanvasItem, zoom: f32) -> impl IntoElement {
    if zoom >= 0.5 {
        // Render normally - text is still readable
        item_content(item).into_any_element()
    } else {
        // Too small - use cached texture thumbnail
        cached_texture_thumbnail(item).into_any_element()
    }
}
```

## Technical Requirements

### To Adapt TexturedSurface for Canvas Use

1. **Expose Texture Handles** - Currently only exposes pixel data via `read_pixels()`. Would need to expose the GPU texture directly for efficient compositing.

2. **Multiple Render Targets** - Current implementation has one render target per window. Would need pooling or dynamic creation.

3. **Texture Compositing** - Need a way to draw a texture at arbitrary position/scale in the main render pass. This likely requires adding a new primitive type or using the existing `PolychromeSprite` path.

4. **Resource Management** - Texture pooling, caching, eviction strategies.

### Changes Needed to GPUI/Blade

```rust
// New primitive for texture compositing
pub struct TextureQuad {
    pub order: DrawOrder,
    pub bounds: Bounds<ScaledPixels>,
    pub texture: gpu::TextureView,
    pub content_mask: ContentMask<ScaledPixels>,
}

// New method on Window
impl Window {
    pub fn paint_texture_quad(
        &mut self, 
        bounds: Bounds<Pixels>,
        texture: gpu::TextureView,
        source_rect: Option<Bounds<Pixels>>,
    );
}
```

## Recommended Approach

### Phase 1: Node-Graph Style (No Textures)
Use the current GPUI layout system with `layout_as_root` + `prepaint_at`:
- Content scales with zoom
- LOD for readability at low zoom
- No GPU complexity

### Phase 2: Texture Thumbnails (If Needed)
Add optional texture caching for zoomed-out views:
- Generate thumbnail textures for items below 50% zoom
- Use current GPUI rendering above 50%
- Thumbnails regenerate when content changes

### Phase 3: Full Texture Pipeline (If Performance Demands)
Adapt TexturedSurface for canvas item rendering:
- Render each item to texture at native resolution
- Composite textures with zoom scaling
- Full caching and pooling

## Code References

- `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs` - Core rendering
- `vendor/zed/crates/gpui/examples/textured_surface.rs` - Usage example
- `vendor/zed/crates/gpui/src/platform/blade/` - GPU abstraction layer
- `vendor/zed/crates/gpui/src/scene.rs` - Scene primitives

## Conclusion

The TexturedSurface implementation proves that GPUI can render to textures. For the infinite canvas:

1. **Short term**: Use GPUI's native layout with zoom scaling (Phase 1)
2. **Medium term**: Add LOD and thumbnail caching if needed (Phase 2)
3. **Long term**: Full texture-based rendering if performance demands (Phase 3)

The key insight is that TexturedSurface is currently designed for "one window = one texture" headless rendering. Adapting it for "many items = many textures within one window" would require significant architectural changes to GPUI's rendering pipeline, specifically around texture compositing and resource management.