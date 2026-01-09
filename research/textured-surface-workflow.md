# TexturedSurface Workflow Research

## Overview

The `gpui-oneshot-render` branch (commit `156a053de99c3fb079f9d4697e7eba793a08bfe2`) in `vendor/zed` provides a `TexturedSurface` platform implementation that enables GPUI rendering to GPU textures instead of display windows. This document explores how to leverage this for the infinite canvas render-to-texture use case.

---

## What's Available

### Branch: `gpui-oneshot-render`

The vendored Zed codebase includes these additions:

| File | Purpose |
|------|---------|
| `crates/gpui/src/app.rs` | `Application::textured()` constructor |
| `crates/gpui/src/platform.rs` | `textured_platform()` function |
| `crates/gpui/src/platform/linux/textured_surface/mod.rs` | Module exports |
| `crates/gpui/src/platform/linux/textured_surface/client.rs` | `TexturedSurfaceClient` - platform implementation |
| `crates/gpui/src/platform/linux/textured_surface/display.rs` | `TexturedSurfaceDisplay` - virtual display |
| `crates/gpui/src/platform/linux/textured_surface/window.rs` | `TexturedSurfaceWindow` - core rendering (~1000 lines) |
| `crates/gpui/src/window.rs` | `draw_and_present()`, `read_pixels()` methods |
| `crates/gpui/examples/textured_surface.rs` | Working example |

### Key APIs

```rust
// Create a textured (headless) application
let app = Application::textured();

app.run(move |cx: &mut App| {
    // Open a "window" that renders to texture
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        },
        |_, cx| cx.new(|_| MyView::new()),
    )?;

    // Force render to texture
    cx.update_window(window_handle, |_, window, cx| {
        window.draw_and_present(cx)
    });

    // Read pixels back to CPU
    let pixels: Option<Vec<u8>> = cx.update_window(window_handle, |_, window, _| {
        window.read_pixels()
    });
});
```

---

## Current Limitations for Infinite Canvas

### Problem 1: Separate Application Context

`Application::textured()` creates a **completely separate** GPUI application context. This means:

- Separate GPU context
- Separate atlas (glyphs, images not shared)
- Separate entity system
- Cannot mix with a normal windowed application

**For infinite canvas, we need to render textures within an existing window's context.**

### Problem 2: Full Window Lifecycle

TexturedSurface uses the full GPUI window lifecycle:
- Creates a `Window` struct with all its machinery
- Manages input handlers, focus, dispatch trees
- Has its own entity system

**For canvas items, we just need to render a Scene to a texture - much lighter weight.**

### Problem 3: No Texture Sharing

The rendered texture is:
- Read back to CPU as `Vec<u8>` pixels
- Not exposed as a `gpu::TextureView` for GPU-side compositing

**For efficient canvas rendering, we need to keep textures on the GPU.**

---

## Workflow Options

### Option A: Use TexturedSurface As-Is (Suboptimal)

Create a secondary `Application::textured()` instance for rendering thumbnails.

```rust
// In main app
fn render_thumbnail(item: &CanvasItem) -> Vec<u8> {
    let pixels = Rc::new(RefCell::new(None));
    let pixels_clone = pixels.clone();
    
    // Spawn a separate textured app (expensive!)
    let app = Application::textured();
    app.run(move |cx| {
        let window = cx.open_window(..., |_, cx| {
            cx.new(|_| ItemView::new(item.clone()))
        });
        
        cx.update_window(window, |_, w, cx| w.draw_and_present(cx));
        *pixels_clone.borrow_mut() = cx.update_window(window, |_, w, _| w.read_pixels());
        cx.quit();
    });
    
    pixels.borrow().clone().unwrap()
}
```

**Pros:**
- Works today with no GPUI modifications
- Full GPUI rendering (text, images, etc.)

**Cons:**
- Creates new GPU context per render (slow)
- Pixels must round-trip through CPU (slow)
- No atlas sharing (glyphs re-rasterized)
- Blocking call

### Option B: Extract Scene Rendering (Better)

Use the rendering code from `TexturedSurfaceWindow` but integrate with existing window's GPU context.

This is what the `render-to-texture-methodology.md` document proposes:

1. Refactor `BladeRenderer::draw()` to accept a target texture
2. Share GPU context, atlas, pipelines between window and offscreen rendering
3. Keep textures on GPU for compositing

**Pros:**
- Efficient GPU resource sharing
- Textures stay on GPU
- Can reuse existing atlas

**Cons:**
- Requires GPUI modifications
- Still need to build Scene somehow

### Option C: Hybrid Approach (Pragmatic)

Use TexturedSurface for **initial prototyping**, then optimize later.

**Phase 1: Prototype with TexturedSurface**
- Use `Application::textured()` in a background thread
- Render thumbnails as PNG/pixels
- Upload to main window as images
- Proves the concept works

**Phase 2: Optimize with shared rendering**
- Apply the `BladeRenderer` refactoring
- Keep textures on GPU
- Eliminate CPU round-trip

---

## Recommended Workflow

### For Infinite Canvas MVP

1. **Don't use TexturedSurface initially**
   - Too heavyweight for per-item rendering
   - CPU pixel round-trip kills performance

2. **Use native GPUI rendering with LOD**
   - Render items directly using `layout_as_root()` + `prepaint_at()`
   - At low zoom, simplify rendering (no text, basic shapes)
   - This works today with no GPUI changes

3. **Add texture caching later**
   - When performance demands it
   - Apply the `BladeRenderer::draw_to_texture()` refactoring
   - Cache rendered items as GPU textures

### For Thumbnail/Export Features

TexturedSurface is perfect for:
- Generating PNG exports
- Creating preview thumbnails for file browsers
- Visual regression testing
- Any "render once, read pixels" use case

Example for canvas export:

```rust
pub fn export_canvas_to_png(canvas: &InfiniteCanvas, path: &Path) -> Result<()> {
    let pixels = Rc::new(RefCell::new(None));
    let pixels_clone = pixels.clone();
    let canvas_clone = canvas.clone();
    
    let app = Application::textured();
    app.run(move |cx: &mut App| {
        let bounds = Bounds::new(
            point(px(0.0), px(0.0)),
            canvas_clone.content_bounds().size,
        );
        
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| CanvasExportView::new(canvas_clone)),
        ).unwrap();
        
        let handle: AnyWindowHandle = window.into();
        let pixels_for_cb = pixels_clone.clone();
        let width = bounds.size.width.0 as u32;
        let height = bounds.size.height.0 as u32;
        
        cx.spawn(async move |cx| {
            cx.update_window(handle, |_, w, cx| w.draw_and_present(cx)).ok();
            if let Ok(Some(data)) = cx.update_window(handle, |_, w, _| w.read_pixels()) {
                *pixels_for_cb.borrow_mut() = Some((data, width, height));
            }
            cx.update(|cx| cx.quit()).ok();
        }).detach();
    });
    
    if let Some((pixels, width, height)) = pixels.borrow().clone() {
        save_pixels_as_png(&pixels, width, height, path)?;
    }
    
    Ok(())
}
```

---

## Technical Deep Dive: TexturedSurfaceWindow

### How It Works

```
Application::textured()
    │
    ▼
textured_platform() → TexturedSurfaceClient
    │
    ▼
open_window() → TexturedSurfaceWindow
    │
    ├── GPU Context (presentation: false)
    ├── Render Target Texture
    ├── TexturedSurfacePipelines (same shaders as BladeRenderer)
    ├── BladeAtlas (for glyphs/images)
    └── BufferBelt (GPU memory management)
```

### Key Implementation: render_scene_to_texture()

Location: `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs` L398-567

```rust
fn render_scene_to_texture(&mut self, scene: &Scene) {
    self.command_encoder.start();
    self.atlas.before_frame(&mut self.command_encoder);
    self.command_encoder.init_texture(self.render_target);

    let globals = GlobalParams {
        viewport_size: [width, height],
        premultiplied_alpha: 1,
        pad: 0,
    };

    // Target is render_target_view (texture), not window surface
    let mut pass = self.command_encoder.render(
        "textured_surface_main",
        gpu::RenderTargetSet {
            colors: &[gpu::RenderTarget {
                view: self.render_target_view,  // <-- THE KEY DIFFERENCE
                init_op: gpu::InitOp::Clear(gpu::TextureColor::TransparentBlack),
                finish_op: gpu::FinishOp::Store,
            }],
            depth_stencil: None,
        },
    );

    // Same batch processing as BladeRenderer
    for batch in scene.batches() {
        match batch {
            PrimitiveBatch::Quads(quads) => { /* ... */ }
            PrimitiveBatch::Shadows(shadows) => { /* ... */ }
            PrimitiveBatch::Paths(paths) => { /* ... */ }
            // etc.
        }
    }

    drop(pass);
    let sync_point = self.gpu.submit(&mut self.command_encoder);
    // NO present() - texture stays in VRAM
}
```

### Key Implementation: read_pixels_to_buffer()

Location: `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs` L631-676

```rust
fn read_pixels_to_buffer(&mut self) {
    let width = (self.bounds.size.width.0 * self.scale_factor) as u32;
    let height = (self.bounds.size.height.0 * self.scale_factor) as u32;
    let buffer_size = (width * height * 4) as u64;

    // Create staging buffer in CPU-accessible memory
    let staging = self.gpu.create_buffer(gpu::BufferDesc {
        name: "pixel_readback",
        size: buffer_size,
        memory: gpu::Memory::Shared,  // CPU-accessible
    });

    // Copy texture → buffer
    self.command_encoder.start();
    {
        let mut transfer = self.command_encoder.transfer("readback");
        transfer.copy_texture_to_buffer(
            gpu::TexturePiece {
                texture: self.render_target,
                mip_level: 0,
                array_layer: 0,
                origin: [0, 0, 0],
            },
            staging.into(),
            row_pitch as u32,
            gpu::Extent { width, height, depth: 1 },
        );
    }

    let sync_point = self.gpu.submit(&mut self.command_encoder);
    self.gpu.wait_for(&sync_point, MAX_FRAME_TIME_MS);

    // Read pixels from staging buffer
    let pixels = unsafe {
        std::slice::from_raw_parts(staging.data() as *const u8, buffer_size as usize)
    }.to_vec();

    self.gpu.destroy_buffer(staging);
    self.rendered_pixels = Some(pixels);
}
```

---

## Future Work: GPU Texture Sharing

To enable efficient texture caching for infinite canvas:

### 1. Expose texture_view() (already exists!)

```rust
// vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs L211-213
pub fn texture_view(&self) -> gpu::TextureView {
    self.0.borrow().render_target_view
}
```

### 2. Add to PlatformWindow trait

```rust
// In platform.rs
pub(crate) trait PlatformWindow {
    // ... existing methods ...
    
    /// Get the rendered texture view (for GPU-side compositing)
    fn texture_view(&self) -> Option<gpu::TextureView> {
        None
    }
}
```

### 3. Create texture compositing primitive

Either use existing `PolychromeSprite` or add:

```rust
pub struct TextureQuad {
    pub order: DrawOrder,
    pub bounds: Bounds<ScaledPixels>,
    pub texture_view: gpu::TextureView,
    pub content_mask: ContentMask<ScaledPixels>,
}
```

---

## Summary

| Use Case | Recommended Approach |
|----------|---------------------|
| Infinite canvas MVP | Native GPUI rendering with LOD |
| Canvas item caching | Future: `BladeRenderer::draw_to_texture()` |
| PNG export | `Application::textured()` |
| Thumbnails (one-shot) | `Application::textured()` |
| Visual testing | `Application::textured()` |

The TexturedSurface implementation proves that GPUI can render to textures. For real-time canvas use, we need tighter integration with the existing window's GPU context, which requires the refactoring outlined in `render-to-texture-methodology.md`.

---

## References

- `vendor/zed/crates/gpui/examples/textured_surface.rs` - Working example
- `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs` - Core implementation
- `research/render-to-texture-methodology.md` - BladeRenderer refactoring proposal
- `research/gpui-blade-rendering-pipeline.md` - Full rendering pipeline analysis