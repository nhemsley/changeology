# Render-to-Texture Methods Reference

This document provides direct links to the actual rendering methods in GPUI and the Blade renderer that would be relevant for implementing Phase 3 (full texture pipeline) for the infinite canvas.

## Overview

GPUI uses the Blade graphics abstraction layer for GPU rendering. There are two key implementations:

1. **BladeRenderer** - The main renderer used for window-based rendering
2. **TexturedSurfaceWindow** - A headless renderer that renders to a texture (proof that render-to-texture works)

---

## Blade Renderer (Window Rendering)

**File:** `vendor/zed/crates/gpui/src/platform/blade/blade_renderer.rs`

### Core Structure

```rust
// L328-345
pub struct BladeRenderer {
    gpu: Arc<gpu::Context>,
    surface: gpu::Surface,
    surface_config: gpu::SurfaceConfig,
    command_encoder: gpu::CommandEncoder,
    last_sync_point: Option<gpu::SyncPoint>,
    pipelines: BladePipelines,
    instance_belt: BufferBelt,
    atlas: Arc<BladeAtlas>,
    atlas_sampler: gpu::Sampler,
    path_intermediate_texture: gpu::Texture,
    path_intermediate_texture_view: gpu::TextureView,
    path_intermediate_msaa_texture: Option<gpu::Texture>,
    path_intermediate_msaa_texture_view: Option<gpu::TextureView>,
    rendering_parameters: RenderingParameters,
    // ... macOS-specific fields
}
```

### Key Methods

| Method | Lines | Description |
|--------|-------|-------------|
| `BladeRenderer::new()` | L348-432 | Initializes GPU context, creates surface, pipelines, atlas, and intermediate textures |
| `BladeRenderer::draw()` | L644-919 | **Main scene rendering** - processes all primitive batches and submits to GPU |
| `BladeRenderer::draw_paths_to_intermediate()` | L565-623 | Renders vector paths to intermediate texture with MSAA |
| `BladeRenderer::update_drawable_size_impl()` | L471-513 | Resizes render targets when window size changes |
| `BladePipelines::new()` | L138-305 | Creates all GPU render pipelines (shaders) |

### Draw Method Flow (L644-919)

1. Start command encoder
2. Acquire frame from surface
3. Initialize render target
4. Begin render pass with clear
5. Process each `PrimitiveBatch`:
   - `Quads` → quad pipeline
   - `Shadows` → shadow pipeline
   - `Paths` → rasterize to intermediate, then composite
   - `Underlines` → underline pipeline
   - `MonochromeSprites` → glyph rendering
   - `PolychromeSprites` → image rendering
   - `Surfaces` → video frames (macOS)
6. Present frame
7. Submit and wait for GPU

---

## Textured Surface Window (Headless Render-to-Texture)

**File:** `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs`

### Core Structure

```rust
// L45-74
pub struct TexturedSurfaceWindowState {
    handle: AnyWindowHandle,
    bounds: Bounds<Pixels>,
    scale_factor: f32,
    
    gpu: Arc<gpu::Context>,
    render_target: gpu::Texture,          // The output texture!
    render_target_view: gpu::TextureView,
    
    atlas: Arc<BladeAtlas>,
    atlas_sampler: gpu::Sampler,
    command_encoder: gpu::CommandEncoder,
    pipelines: TexturedSurfacePipelines,
    instance_belt: BufferBelt,
    
    path_intermediate_texture: gpu::Texture,
    path_intermediate_texture_view: gpu::TextureView,
    path_intermediate_msaa_texture: Option<gpu::Texture>,
    path_intermediate_msaa_texture_view: Option<gpu::TextureView>,
    
    rendered_pixels: Option<Vec<u8>>,     // CPU-side pixel data
    // ... callbacks
}
```

### Key Methods

| Method | Lines | Description |
|--------|-------|-------------|
| `TexturedSurfaceWindow::new()` | L92-203 | Creates GPU context without presentation, creates render target texture |
| `TexturedSurfaceWindowState::render_scene_to_texture()` | L398-567 | **Core render-to-texture logic** - same as BladeRenderer but targets a texture |
| `TexturedSurfaceWindowState::draw_paths_to_intermediate()` | L569-629 | Path rasterization (identical to BladeRenderer) |
| `TexturedSurfaceWindowState::read_pixels_to_buffer()` | L631-676 | Copies texture to CPU memory for pixel readback |
| `TexturedSurfaceWindow::texture_view()` | L211-213 | Exposes the GPU texture view (key for compositing!) |

### Render Target Creation (L92-203)

```rust
// Key: GPU context without presentation (headless)
let gpu = Arc::new(unsafe {
    gpu::Context::init(gpu::ContextDesc {
        presentation: false,  // No display needed!
        validation: cfg!(debug_assertions),
        ..Default::default()
    })
});

// Create render target texture
let render_target = gpu.create_texture(gpu::TextureDesc {
    name: "textured_surface_target",
    format: gpu::TextureFormat::Bgra8UnormSrgb,
    size: gpu::Extent { width, height, depth: 1 },
    usage: gpu::TextureUsage::TARGET | gpu::TextureUsage::COPY,
    // ...
});
```

### Pixel Readback (L631-676)

```rust
fn read_pixels_to_buffer(&mut self) {
    // Create staging buffer in shared memory
    let staging = self.gpu.create_buffer(gpu::BufferDesc {
        memory: gpu::Memory::Shared,
        // ...
    });
    
    // Copy texture to buffer via transfer pass
    transfer.copy_texture_to_buffer(/* ... */);
    
    // Read pixels to Vec<u8>
    let pixels = unsafe {
        std::slice::from_raw_parts(staging.data() as *const u8, buffer_size)
    }.to_vec();
}
```

---

## Scene Primitives

**File:** `vendor/zed/crates/gpui/src/scene.rs`

### Scene Structure (L24-35)

```rust
pub(crate) struct Scene {
    pub(crate) paint_operations: Vec<PaintOperation>,
    primitive_bounds: BoundsTree<ScaledPixels>,
    layer_stack: Vec<DrawOrder>,
    pub(crate) shadows: Vec<Shadow>,
    pub(crate) quads: Vec<Quad>,
    pub(crate) paths: Vec<Path<ScaledPixels>>,
    pub(crate) underlines: Vec<Underline>,
    pub(crate) monochrome_sprites: Vec<MonochromeSprite>,
    pub(crate) polychrome_sprites: Vec<PolychromeSprite>,
    pub(crate) surfaces: Vec<PaintSurface>,
}
```

### Primitive Batches (L435-449)

```rust
pub(crate) enum PrimitiveBatch<'a> {
    Shadows(&'a [Shadow]),
    Quads(&'a [Quad]),
    Paths(&'a [Path<ScaledPixels>]),
    Underlines(&'a [Underline]),
    MonochromeSprites { texture_id: AtlasTextureId, sprites: &'a [MonochromeSprite] },
    PolychromeSprites { texture_id: AtlasTextureId, sprites: &'a [PolychromeSprite] },
    Surfaces(&'a [PaintSurface]),
}
```

---

## Key Differences: BladeRenderer vs TexturedSurfaceWindow

| Aspect | BladeRenderer | TexturedSurfaceWindow |
|--------|--------------|----------------------|
| **Target** | Window surface | GPU texture |
| **Presentation** | `gpu::Surface` (display) | `gpu::Texture` (offscreen) |
| **GPU Init** | `presentation: true` | `presentation: false` |
| **Output** | Screen pixels | Texture view / pixel buffer |
| **Use Case** | Normal window rendering | Headless / offscreen rendering |

---

## What's Needed for Infinite Canvas Render-to-Texture

Based on this analysis, to implement Phase 3 (full texture pipeline):

### 1. Texture Creation (Already Demonstrated)

Use the pattern from `TexturedSurfaceWindow::new()`:
```rust
let item_texture = gpu.create_texture(gpu::TextureDesc {
    name: "canvas_item",
    format: gpu::TextureFormat::Bgra8UnormSrgb,
    size: gpu::Extent { width, height, depth: 1 },
    usage: gpu::TextureUsage::TARGET | gpu::TextureUsage::COPY,
    // ...
});
```

### 2. Render to Texture (Already Demonstrated)

Use `render_scene_to_texture()` pattern - target a texture view instead of surface.

### 3. Texture Compositing (NEEDS IMPLEMENTATION)

**This is the missing piece.** Options:

a) **Use PolychromeSprite path**: Register texture in atlas, render as sprite
   - Requires atlas modifications to accept external textures

b) **New TextureQuad primitive**: Add to `PrimitiveBatch` enum
   ```rust
   // Would need to add to scene.rs
   pub struct TextureQuad {
       pub order: DrawOrder,
       pub bounds: Bounds<ScaledPixels>,
       pub texture_view: gpu::TextureView,
       pub content_mask: ContentMask<ScaledPixels>,
   }
   
   // And to PrimitiveBatch
   TextureQuads(&'a [TextureQuad]),
   ```

c) **Paint method on Window**: Add `paint_texture_quad()` that wraps texture as sprite
   ```rust
   impl Window {
       pub fn paint_texture_quad(
           &mut self,
           bounds: Bounds<Pixels>,
           texture: gpu::TextureView,
       );
   }
   ```

### 4. Resource Management (NEEDS IMPLEMENTATION)

- Texture pool for reuse
- Cache invalidation when item content changes
- Memory limits / eviction strategy

---

## File Quick Reference

| File | Purpose |
|------|---------|
| `vendor/zed/crates/gpui/src/platform/blade/blade_renderer.rs` | Main GPU renderer |
| `vendor/zed/crates/gpui/src/platform/blade/blade_atlas.rs` | Texture atlas for sprites/glyphs |
| `vendor/zed/crates/gpui/src/platform/blade/blade_context.rs` | GPU context wrapper |
| `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs` | Headless texture renderer |
| `vendor/zed/crates/gpui/src/platform/linux/textured_surface/client.rs` | Platform client for textured surface |
| `vendor/zed/crates/gpui/src/platform/linux/textured_surface/display.rs` | Virtual display for headless rendering |
| `vendor/zed/crates/gpui/src/scene.rs` | Scene graph and primitives |
| `vendor/zed/crates/gpui/examples/textured_surface.rs` | **Working example** of render-to-texture |

---

## Example: TexturedSurface Usage

**File:** `vendor/zed/crates/gpui/examples/textured_surface.rs`

This example demonstrates the full render-to-texture workflow:

```rust
// 1. Create headless application
let app = Application::textured();

app.run(move |cx: &mut App| {
    // 2. Open a "window" (actually renders to texture)
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        },
        |_, cx| cx.new(|_| DemoView::new(1)),
    )?;
    
    // 3. Draw to texture
    cx.update_window(window_handle, |_, window, cx| {
        window.draw_and_present(cx)
    });
    
    // 4. Read pixels back to CPU
    let pixels = cx.update_window(window_handle, |_, window, _| {
        window.read_pixels()
    });
});
```

Run with:
```bash
cargo run --example textured_surface --features wayland
```

---

## Next Steps

1. **Prototype**: Create a minimal render-to-texture + composite test
2. **Decide compositing approach**: PolychromeSprite reuse vs new primitive
3. **Implement texture pool**: For efficient memory management
4. **Integrate with canvas**: Replace direct rendering with cached textures below zoom threshold