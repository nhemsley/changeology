# Infinite Canvas Zoom Rendering Considerations

## The Problem

When implementing an infinite canvas with zoom capability, we face a fundamental rendering challenge:

**At zoom levels != 100%, how should child elements be rendered?**

This is more complex than a scrollable container because:
- **Scrollable div**: Content renders at 100%, viewport clips what's visible
- **Zoomed canvas**: Content needs to be *scaled*, not just clipped

## Approach 1: Scale During Render (Current Approach)

What we currently do:
1. Calculate screen-space bounds for each item: `screen_bounds = camera.canvas_to_screen_bounds(item.bounds)`
2. Draw primitives (quads) at the scaled size
3. Grid lines scale with zoom

**For primitives (rectangles, lines)**: This works perfectly. A 100x100 canvas item at 50% zoom becomes a 50x50 screen quad.

**For text/child elements**: This is where it gets complicated:
- GPUI lays out elements at pixel sizes
- If we tell a div it's 50x50px, text will be tiny/unreadable
- If we tell a div it's 100x100px (canvas size), it renders at the wrong screen position

## Approach 2: Render at Native Size, Then Scale (Texture-based)

How tools like Figma, Miro work:
1. Render each item's content to an offscreen texture at 100% (or fixed DPI)
2. Draw that texture onto the canvas, scaled by zoom factor
3. Cache textures for performance

**Pros:**
- Text remains crisp at its "native" size until you zoom past 100%
- Familiar mental model
- Can implement LOD (different detail levels at different zooms)

**Cons:**
- GPUI doesn't have general-purpose offscreen rendering
- `Surface` element exists but is macOS-only (CoreVideo pixel buffers)
- Memory usage for cached textures
- Complexity of texture management

## Approach 3: Re-layout at Screen Size (GPUI-native)

Work with GPUI's layout system:
1. During prepaint, calculate screen-space size for visible items
2. Use `layout_as_root(screen_size, ...)` to layout children at screen size
3. Child elements render at screen resolution

**Example:**
```rust
fn prepaint(&mut self, ...) {
    for item in visible_items {
        // Canvas item is 200x100 in canvas space
        // At 50% zoom, screen size is 100x50
        let screen_size = size(
            item.bounds.size.width * camera.zoom,
            item.bounds.size.height * camera.zoom,
        );
        
        let available_space = size(
            AvailableSpace::Definite(screen_size.width),
            AvailableSpace::Definite(screen_size.height),
        );
        
        // Child element layouts to 100x50 screen pixels
        child.layout_as_root(available_space, window, cx);
        child.prepaint_at(screen_origin, window, cx);
    }
}
```

**Pros:**
- Works with GPUI's existing system
- No offscreen rendering needed
- Children are true GPUI elements with full interactivity

**Cons:**
- Text size scales with zoom (may become unreadable at low zoom)
- Content may not fit when zoomed out (needs overflow handling)
- Layout recalculates on every zoom change

## Approach 4: Fixed-Size Items with Zoom-Independent Content

Hybrid approach:
1. Items have fixed screen-space sizes (e.g., always 200x100 screen pixels)
2. Only position changes with pan/zoom
3. Items get smaller spacing when zoomed out, but content stays readable

**This is NOT a true infinite canvas** - more like a node graph editor where nodes stay readable at any zoom.

## Approach 5: Semantic Zoom (LOD)

Different content at different zoom levels:
- **Zoomed out (< 50%)**: Show item as colored rectangle with label
- **Normal (50-150%)**: Show full content
- **Zoomed in (> 150%)**: Show additional detail

**Implementation:**
```rust
fn render_item_content(item: &CanvasItem, zoom: f32) -> AnyElement {
    if zoom < 0.5 {
        // Just a label
        div().child(item.title.clone()).into_any_element()
    } else if zoom < 1.5 {
        // Normal content
        div()
            .child(item.title.clone())
            .child(item.description.clone())
            .into_any_element()
    } else {
        // Detailed view
        full_item_detail_view(item).into_any_element()
    }
}
```

## GPUI Limitations

Based on code research:

1. **No CSS-like transforms**: GPUI doesn't have `transform: scale()` equivalent
2. **No general offscreen rendering**: `Surface` is macOS-only for video
3. **Layout is pixel-based**: Elements layout to specific pixel sizes
4. **`Path::scale()` exists**: Low-level paths can be scaled, but not elements

## Recommended Approach for Infinite Canvas

### Phase 1: Simple Scaling (Current + Children)
Use Approach 3 - let children layout at screen size:
- Children scale with zoom (text gets smaller)
- Simple to implement with `layout_as_root` + `prepaint_at`
- Works for node-graph style UIs

### Phase 2: Semantic Zoom (If Needed)
Add LOD support:
- Render callback receives zoom level
- Can return different content at different zooms
- Items show less detail when zoomed out

### Phase 3: Texture Caching (Future/If Needed)
If performance becomes an issue:
- Investigate GPUI's image caching
- Potentially render to image at fixed resolution
- This would require GPUI enhancements

## Key Insight

The fundamental question is: **What UX do we want?**

1. **Figma-style**: Content renders at fixed DPI, zooming is like a magnifying glass
   - Requires texture-based rendering (not easy in GPUI)

2. **Node-graph style**: Content scales with zoom, may need LOD for readability
   - Works with GPUI's layout system
   - Better for our use case (code visualization, architecture diagrams)

For the infinite canvas component, **Option 2 (node-graph style) is recommended** because:
- It works with GPUI's existing architecture
- It's appropriate for the intended use cases
- LOD can address readability at extreme zoom levels
- No platform-specific code needed

## Implementation Notes

When implementing children with scaling:

```rust
// In prepaint:
let screen_bounds = camera.canvas_to_screen_bounds(item.bounds);

// Give child the screen size to layout into
let available = size(
    AvailableSpace::Definite(screen_bounds.size.width),
    AvailableSpace::Definite(screen_bounds.size.height),
);

// Create child element (via callback or stored)
let mut child = render_item(item, camera.zoom, window, cx);

// Layout at screen size
child.layout_as_root(available, window, cx);

// Position at screen location (accounting for canvas origin offset)
let screen_origin = point(
    bounds.origin.x + screen_bounds.origin.x,
    bounds.origin.y + screen_bounds.origin.y,
);
child.prepaint_at(screen_origin, window, cx);

// Store for paint phase
prepared_items.push(PreparedItem { child, screen_bounds });
```

```rust
// In paint:
for item in &mut prepared_items {
    // Paint background
    window.paint_quad(fill(item.screen_bounds, bg_color));
    
    // Paint child content
    item.child.paint(window, cx);
}
```

## References

- `vendor/gpui/src/element.rs` - AnyElement::layout_as_root, prepaint_at
- `vendor/gpui-component/crates/ui/src/virtual_list.rs` - Dynamic child positioning
- `vendor/gpui/src/elements/anchored.rs` - Absolute positioning patterns
- `vendor/gpui/src/scene.rs` - Low-level rendering primitives