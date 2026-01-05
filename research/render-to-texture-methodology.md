# Render-to-Texture Methodology

## Goal

Enable "oneshot" rendering of a GPUI Scene directly to a GPU texture, bypassing the typical GPUI app lifecycle (Window, App, entity system, etc.). This is useful for:

- Caching canvas items as textures for efficient zoom/pan
- Generating thumbnails
- Offscreen rendering for image export
- Any scenario where you want GPUI rendering without a window

## Design Principles

1. **Minimize changes to GPUI** - Work with existing structures, don't fork
2. **Reuse existing rendering code** - BladeRenderer already does 95% of what we need
3. **Stay outside the lifecycle** - No Window/App context required for the render itself
4. **Share GPU resources** - Use the same atlas, pipelines, and GPU context as the main renderer

---

## Current Architecture

### BladeRenderer::draw() (L644-919)

The current method implicitly uses `self.surface`:

```rust
pub fn draw(&mut self, scene: &Scene) {
    // 1. Acquire frame from window surface
    let frame = self.surface.acquire_frame();
    
    // 2. Get viewport size from surface config
    let viewport_size = [
        self.surface_config.size.width as f32,
        self.surface_config.size.height as f32,
    ];
    
    // 3. Get alpha mode from surface
    let premultiplied_alpha = match self.surface.info().alpha { ... };
    
    // 4. Render to frame.texture_view()
    let mut pass = self.command_encoder.render(..., frame.texture_view(), ...);
    
    // 5. Process all batches (quads, shadows, paths, sprites, etc.)
    for batch in scene.batches() { ... }
    
    // 6. Present to window
    self.command_encoder.present(frame);
}
```

### Surface-Specific Code Points

| Line | Code | Purpose |
|------|------|---------|
| L650-652 | `self.surface.acquire_frame()` | Get window frame |
| L656-657 | `self.surface_config.size` | Viewport dimensions |
| L658 | `self.surface.info().alpha` | Alpha blending mode |
| L670 | `frame.texture_view()` | Render target |
| L718 | `frame.texture_view()` | Resume after paths |
| L910 | `present(frame)` | Display on window |

**Key insight:** The batch processing loop (L679-907) is completely target-agnostic. It just needs a texture view and viewport size.

---

## Proposed Changes

### 1. Add RenderTarget Enum

```rust
// In blade_renderer.rs

/// A render target for the blade renderer
pub enum RenderTarget<'a> {
    /// Render to a window surface frame (will be presented)
    Surface(&'a gpu::Frame),
    /// Render to an offscreen texture (no presentation)
    Texture {
        view: gpu::TextureView,
        size: gpu::Extent,
    },
}

impl<'a> RenderTarget<'a> {
    pub fn texture_view(&self) -> gpu::TextureView {
        match self {
            RenderTarget::Surface(frame) => frame.texture_view(),
            RenderTarget::Texture { view, .. } => *view,
        }
    }
    
    pub fn size(&self) -> [f32; 2] {
        match self {
            RenderTarget::Surface(frame) => {
                // Would need surface_config passed in or stored
                unimplemented!()
            }
            RenderTarget::Texture { size, .. } => {
                [size.width as f32, size.height as f32]
            }
        }
    }
}
```

### 2. Refactor BladeRenderer::draw()

Split into two methods:

```rust
impl BladeRenderer {
    /// Render a scene to a specific target (internal implementation)
    fn draw_to_target(
        &mut self,
        scene: &Scene,
        target_view: gpu::TextureView,
        viewport_size: [f32; 2],
        premultiplied_alpha: bool,
    ) -> gpu::SyncPoint {
        self.command_encoder.start();
        self.atlas.before_frame(&mut self.command_encoder);

        let globals = GlobalParams {
            viewport_size,
            premultiplied_alpha: if premultiplied_alpha { 1 } else { 0 },
            pad: 0,
        };

        let mut pass = self.command_encoder.render(
            "main",
            gpu::RenderTargetSet {
                colors: &[gpu::RenderTarget {
                    view: target_view,
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::TransparentBlack),
                    finish_op: gpu::FinishOp::Store,
                }],
                depth_stencil: None,
            },
        );

        // ... existing batch processing loop (unchanged) ...
        // Note: paths section needs target_view saved for resume
        
        for batch in scene.batches() {
            match batch {
                PrimitiveBatch::Quads(quads) => { /* unchanged */ }
                PrimitiveBatch::Shadows(shadows) => { /* unchanged */ }
                PrimitiveBatch::Paths(paths) => {
                    drop(pass);
                    self.draw_paths_to_intermediate(paths, viewport_size[0], viewport_size[1]);
                    // Resume with saved target_view
                    pass = self.command_encoder.render(
                        "main",
                        gpu::RenderTargetSet {
                            colors: &[gpu::RenderTarget {
                                view: target_view,  // Use passed-in view
                                init_op: gpu::InitOp::Load,
                                finish_op: gpu::FinishOp::Store,
                            }],
                            depth_stencil: None,
                        },
                    );
                    // ... rest of paths handling
                }
                // ... other batches unchanged ...
            }
        }
        
        drop(pass);
        
        let sync_point = self.gpu.submit(&mut self.command_encoder);
        self.instance_belt.flush(&sync_point);
        self.atlas.after_frame(&sync_point);
        
        sync_point
    }

    /// Render to window surface (existing behavior)
    pub fn draw(&mut self, scene: &Scene) {
        let frame = self.surface.acquire_frame();
        self.command_encoder.init_texture(frame.texture());
        
        let viewport_size = [
            self.surface_config.size.width as f32,
            self.surface_config.size.height as f32,
        ];
        let premultiplied_alpha = matches!(
            self.surface.info().alpha,
            gpu::AlphaMode::PreMultiplied
        );
        
        let sync_point = self.draw_to_target(
            scene,
            frame.texture_view(),
            viewport_size,
            premultiplied_alpha,
        );
        
        self.command_encoder.start();
        self.command_encoder.present(frame);
        self.gpu.submit(&mut self.command_encoder);
        
        self.wait_for_gpu();
        self.last_sync_point = Some(sync_point);
    }

    /// Render to an offscreen texture (NEW)
    pub fn draw_to_texture(
        &mut self,
        scene: &Scene,
        texture_view: gpu::TextureView,
        size: gpu::Extent,
    ) -> gpu::SyncPoint {
        let viewport_size = [size.width as f32, size.height as f32];
        
        self.draw_to_target(
            scene,
            texture_view,
            viewport_size,
            true, // Offscreen textures typically use premultiplied alpha
        )
    }
}
```

### 3. Expose Texture Creation Helper

```rust
impl BladeRenderer {
    /// Create an offscreen render target texture
    pub fn create_render_texture(&self, width: u32, height: u32) -> (gpu::Texture, gpu::TextureView) {
        let texture = self.gpu.create_texture(gpu::TextureDesc {
            name: "offscreen_render_target",
            format: gpu::TextureFormat::Bgra8UnormSrgb,
            size: gpu::Extent {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: gpu::TextureDimension::D2,
            usage: gpu::TextureUsage::TARGET | gpu::TextureUsage::COPY,
            external: None,
        });
        
        let view = self.gpu.create_texture_view(
            texture,
            gpu::TextureViewDesc {
                name: "offscreen_render_target_view",
                format: gpu::TextureFormat::Bgra8UnormSrgb,
                dimension: gpu::ViewDimension::D2,
                subresources: &Default::default(),
            },
        );
        
        (texture, view)
    }
    
    /// Get access to the GPU context for advanced usage
    pub fn gpu(&self) -> &Arc<gpu::Context> {
        &self.gpu
    }
}
```

---

## Usage Pattern

### From Within a GPUI App (e.g., Infinite Canvas)

```rust
impl InfiniteCanvas {
    fn render_item_to_texture(
        &mut self,
        item: &CanvasItem,
        window: &mut Window,
        cx: &mut App,
    ) -> CachedTexture {
        // 1. Build a Scene for this item
        let mut scene = Scene::default();
        
        // 2. Paint the item's content into the scene
        //    (This is the tricky part - need to extract paint logic)
        self.paint_item_to_scene(&mut scene, item, window, cx);
        
        // 3. Get the renderer from the platform window
        //    (This requires exposing it somehow)
        let renderer = window.platform_window.renderer_mut();
        
        // 4. Create or reuse a texture
        let (texture, view) = renderer.create_render_texture(
            item.size.width as u32,
            item.size.height as u32,
        );
        
        // 5. Render the scene to the texture
        let sync_point = renderer.draw_to_texture(
            &scene,
            view,
            gpu::Extent {
                width: item.size.width as u32,
                height: item.size.height as u32,
                depth: 1,
            },
        );
        
        CachedTexture { texture, view, sync_point }
    }
}
```

---

## Challenges and Solutions

### Challenge 1: Accessing the Renderer

The `BladeRenderer` is owned by the platform window implementation (e.g., `WaylandWindowState`), not directly accessible from `Window`.

**Solutions:**
- Add a method to `PlatformWindow` trait: `fn renderer(&self) -> Option<&BladeRenderer>`
- Or expose through a new `Window` method
- Or create a separate `OffscreenRenderer` that shares the GPU context

### Challenge 2: Building a Scene Without Window Context

The `paint_*` methods on `Window` add to `self.next_frame.scene` and use window state (scale_factor, content_mask, etc.).

**Solutions:**
- Create standalone paint functions that take explicit parameters:
  ```rust
  pub fn paint_quad_to_scene(
      scene: &mut Scene,
      quad: PaintQuad,
      scale_factor: f32,
      content_mask: ContentMask<Pixels>,
      opacity: f32,
  ) { ... }
  ```
- Or create a lightweight `PaintContext` struct with just the scene and parameters
- Or render using normal GPUI elements but capture the scene before present

### Challenge 3: Text and Images Require Atlas

Glyphs and images are stored in the `BladeAtlas`. The atlas is shared across frames.

**Solution:** The refactored `draw_to_target` already uses `self.atlas`, so as long as we use the same renderer instance, sprites will work correctly.

### Challenge 4: Compositing the Textures Back

After rendering to textures, we need to draw those textures in the main window.

**Solutions:**
- Use existing `PolychromeSprite` primitive (requires registering texture in atlas)
- Add a new `TextureQuad` primitive type
- Create a helper that wraps a texture view as a sprite

---

## Implementation Plan

### Phase 1: Refactor BladeRenderer (Low Risk)

1. Extract `draw_to_target()` as private method
2. Reimplement `draw()` to call `draw_to_target()`
3. Verify all existing rendering still works
4. Add `draw_to_texture()` public method
5. Add `create_render_texture()` helper

### Phase 2: Expose Renderer Access

1. Add `renderer()` method to `PlatformWindow` trait
2. Implement for Wayland, X11, etc.
3. Add `Window::renderer()` or similar accessor

### Phase 3: Scene Building Utilities

1. Create standalone `paint_*_to_scene()` functions
2. Or create `PaintContext` struct for building scenes
3. Test with simple shapes first

### Phase 4: Texture Compositing

1. Investigate using `PolychromeSprite` for texture display
2. Or add `TextureQuad` primitive if needed
3. Integrate with infinite canvas

---

## File Changes Summary

| File | Changes |
|------|---------|
| `platform/blade/blade_renderer.rs` | Add `draw_to_target()`, `draw_to_texture()`, `create_render_texture()` |
| `platform.rs` | Add `renderer()` to `PlatformWindow` trait |
| `platform/linux/wayland/window.rs` | Implement `renderer()` |
| `platform/linux/x11/window.rs` | Implement `renderer()` |
| `window.rs` | Add accessor method (optional) |
| `scene.rs` | Possibly add `TextureQuad` primitive (Phase 4) |

---

## Alternative: Minimal External Approach

If modifying GPUI is too invasive, we could:

1. Create a standalone `OffscreenRenderer` that:
   - Creates its own GPU context (or borrows from main)
   - Has its own atlas (or shares)
   - Implements the same rendering logic
   
2. Use `TexturedSurface` as a template - it already works!

3. The tradeoff is code duplication vs cleaner separation

---

## References

- `vendor/zed/crates/gpui/src/platform/blade/blade_renderer.rs` - Main renderer
- `vendor/zed/crates/gpui/src/platform/linux/textured_surface/window.rs` - Offscreen proof-of-concept
- `vendor/zed/crates/gpui/src/scene.rs` - Scene and primitives
- `vendor/zed/crates/gpui/src/window.rs` - Window paint methods