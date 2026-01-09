# Segfault Investigation: Changeology & Infinite Canvas

## Problem Summary

The changeology application and the `textured` example from infinite-canvas are experiencing intermittent segmentation faults. The `basic` example runs without issues, suggesting the problem is related to the textured rendering system.

## Observed Behavior

1. **Intermittent crashes**: Not every run crashes - sometimes the app starts successfully
2. **Both textured example and changeology affected**: Common code path is `TexturedCanvasItemsProvider`
3. **Basic example works fine**: No texture rendering involved
4. **Sometimes shows "Killed" instead of "Segmentation fault"**: Could indicate OOM or external signal

## Potential Causes

### 1. Async Texture Rendering Thread Safety

The `TexturedCanvasItemsProvider` spawns background threads for rendering:

```rust
// In textured_provider.rs
let thread_handle = std::thread::spawn(move || {
    // Rendering code using Application::textured()
});
```

**Possible issues:**
- Race conditions between render threads and main thread
- Accessing GPU resources from multiple threads
- Channel communication issues with `flume`

**Investigation steps:**
- Add thread-safe logging to track render lifecycle
- Check if crashes correlate with number of concurrent renders
- Try setting `max_concurrent_renders` to 1

### 2. GPUI's `Application::textured()` API

The texture rendering uses GPUI's headless rendering API:

```rust
Application::textured(size, move |window, cx| {
    // Render element to texture
})
```

**Possible issues:**
- This API may not be fully stable on all platforms
- GPU context creation/destruction timing
- Memory management of rendered textures

**Investigation steps:**
- Check GPUI source for known issues with `textured()`
- Test on different Linux display servers (X11 vs Wayland)
- Add error handling around texture creation

### 3. Image Processing and Memory

The downscaling code allocates image buffers:

```rust
if let Some(rgba_img) = image::RgbaImage::from_raw(new_w, new_h, scaled_pixels) {
    let frame = image::Frame::new(rgba_img);
    let render_img = RenderImage::new(smallvec::smallvec![frame]);
    // ...
}
```

**Possible issues:**
- Buffer size mismatches causing memory corruption
- Large allocations failing silently
- Use-after-free in image data

**Investigation steps:**
- Validate buffer sizes before allocation
- Add bounds checking on pixel data access
- Use AddressSanitizer to detect memory issues

### 4. GPU Resource Exhaustion

Creating many textures without proper cleanup:

**Possible issues:**
- GPU memory leak from unreleased textures
- Too many concurrent GPU operations
- Driver-specific issues

**Investigation steps:**
- Monitor GPU memory usage during runtime
- Ensure textures are properly dropped
- Add explicit cleanup on view destruction

### 5. Platform-Specific Issues (Linux)

The code has platform-specific paths:

```rust
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn start_render_job(&mut self, id: &str) {
    // Linux-specific implementation
}
```

**Possible issues:**
- X11/Wayland compatibility
- Mesa/GPU driver bugs
- Vulkan/OpenGL context issues

**Investigation steps:**
- Test with different GPU drivers
- Check `DISPLAY` and `WAYLAND_DISPLAY` environment variables
- Try forcing a specific graphics backend

## Debugging Approaches

### 1. Get a Stack Trace

```bash
# Enable core dumps
ulimit -c unlimited

# Run with GDB
cargo build --bin changeology
gdb -ex run -ex bt ./target/debug/changeology

# Or attach to running process
gdb -p $(pgrep changeology)
```

### 2. Use AddressSanitizer

```bash
# In Cargo.toml or via environment
RUSTFLAGS="-Z sanitizer=address" cargo run --bin changeology
```

### 3. Enable RUST_BACKTRACE

```bash
RUST_BACKTRACE=1 cargo run --bin changeology
RUST_BACKTRACE=full cargo run --bin changeology
```

### 4. Add Debug Logging

Add logging to track the crash location:

```rust
// In diff_canvas.rs render()
eprintln!("DiffCanvasView::render() start");
// ... code ...
eprintln!("DiffCanvasView::render() before provider.tick()");
if self.provider.tick() {
    eprintln!("DiffCanvasView::render() tick returned true");
    cx.notify();
}
eprintln!("DiffCanvasView::render() end");
```

### 5. Bisect the Problem

1. Comment out texture rendering, use placeholder divs
2. Disable downscaling (always use original image)
3. Reduce max_concurrent_renders to 1
4. Remove async rendering, make it synchronous
5. Test with minimal diff data

## Temporary Workarounds

### Option A: Disable Texture Rendering

Replace texture rendering with direct element rendering (slower but stable):

```rust
// Instead of using TexturedCanvasItemsProvider, render elements directly
fn render_diff_card_direct(diff: &FileDiff, cx: &mut Context<Self>) -> impl IntoElement {
    // Render the card directly without texture caching
}
```

### Option B: Lazy Texture Loading

Only request textures when items are visible and the view is stable:

```rust
// Add a delay before requesting textures
if self.frame_count > 10 {
    self.provider.request_texture(&item.id);
}
```

### Option C: Single-Threaded Rendering

Force synchronous, single-threaded texture rendering:

```rust
provider.set_max_concurrent_renders(1);
// And potentially add synchronization barriers
```

## Next Steps

1. **Immediate**: Get a proper stack trace using GDB
2. **Short-term**: Add defensive checks and logging to narrow down crash location
3. **Medium-term**: Review `Application::textured()` implementation in GPUI
4. **Long-term**: Consider alternative rendering strategies if the issue persists

## Related Files

- `crates/infinite-canvas/src/textured_provider.rs` - Main texture rendering logic
- `crates/changeology/src/diff_canvas.rs` - DiffCanvasView implementation
- `crates/infinite-canvas/examples/textured.rs` - Minimal reproduction case
- `vendor/zed/crates/gpui/src/` - GPUI source (check `textured()` implementation)

## Environment Information to Collect

When reporting or debugging:

```bash
# System info
uname -a
cat /etc/os-release

# GPU info
glxinfo | grep -i "renderer\|vendor\|version"
vulkaninfo --summary

# Rust version
rustc --version
cargo --version

# Display server
echo $XDG_SESSION_TYPE
echo $DISPLAY
echo $WAYLAND_DISPLAY
```
