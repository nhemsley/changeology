# GPUI Child Elements Lifecycle Research

## Overview

This document researches how GPUI handles child elements through the layout → prepaint → paint lifecycle, with the goal of understanding how to implement full child element support in the infinite canvas component.

## The Three-Phase Element Lifecycle

Every GPUI element goes through three phases:

### 1. `request_layout` Phase
- **Purpose**: Request layout from Taffy (the layout engine) and initialize element state
- **Input**: Global element ID, inspector ID, window, app context
- **Output**: `(LayoutId, RequestLayoutState)`
- **Key operations**:
  - Call `child.request_layout(window, cx)` for each child
  - Request layout from window with style and child layout IDs
  - Return state needed for prepaint

### 2. `prepaint` Phase  
- **Purpose**: Commit bounds to current frame, set up hitboxes, handle autoscroll
- **Input**: Global ID, inspector ID, bounds (from layout), request_layout state, window, cx
- **Output**: `PrepaintState`
- **Key operations**:
  - Get child bounds via `window.layout_bounds(child_layout_id)`
  - Call `child.prepaint(window, cx)` for each child
  - Set up hitboxes and content masks

### 3. `paint` Phase
- **Purpose**: Actually render the element to the screen
- **Input**: Global ID, inspector ID, bounds, request_layout state, prepaint state, window, cx
- **Output**: None
- **Key operations**:
  - Call `child.paint(window, cx)` for each child
  - Use `window.paint_quad()` for low-level drawing

## Key Types

### `AnyElement`
A type-erased wrapper around any element. Key methods:

```rust
impl AnyElement {
    // Request layout for this element
    pub fn request_layout(&mut self, window: &mut Window, cx: &mut App) -> LayoutId;
    
    // Prepaint this element
    pub fn prepaint(&mut self, window: &mut Window, cx: &mut App) -> Option<FocusHandle>;
    
    // Paint this element  
    pub fn paint(&mut self, window: &mut Window, cx: &mut App);
    
    // Layout as root with available space, returns computed size
    pub fn layout_as_root(
        &mut self,
        available_space: Size<AvailableSpace>,
        window: &mut Window,
        cx: &mut App,
    ) -> Size<Pixels>;
    
    // Prepaint at a specific absolute origin
    pub fn prepaint_at(
        &mut self,
        origin: Point<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<FocusHandle>;
    
    // Layout and prepaint at origin
    pub fn prepaint_as_root(
        &mut self,
        origin: Point<Pixels>,
        available_space: Size<AvailableSpace>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<FocusHandle>;
}
```

### `LayoutId`
An opaque identifier returned by the layout engine. Used to query computed bounds.

### State Types
Each element defines two associated types:
- `RequestLayoutState`: State passed from request_layout to prepaint/paint
- `PrepaintState`: State passed from prepaint to paint

## Pattern 1: Div (Standard Child Handling)

The `Div` element is the canonical example of handling children.

### Structure
```rust
pub struct Div {
    interactivity: Interactivity,
    children: SmallVec<[StackSafe<AnyElement>; 2]>,
    // ...
}

pub struct DivFrameState {
    child_layout_ids: SmallVec<[LayoutId; 2]>,
}
```

### request_layout Implementation
```rust
fn request_layout(&mut self, ...) -> (LayoutId, DivFrameState) {
    let mut child_layout_ids = SmallVec::new();
    
    let layout_id = self.interactivity.request_layout(
        global_id, inspector_id, window, cx,
        |style, window, cx| {
            window.with_text_style(style.text_style().cloned(), |window| {
                // Request layout for each child
                child_layout_ids = self.children
                    .iter_mut()
                    .map(|child| child.request_layout(window, cx))
                    .collect();
                    
                // Request layout with child IDs
                window.request_layout(style, child_layout_ids.iter().copied(), cx)
            })
        },
    );

    (layout_id, DivFrameState { child_layout_ids })
}
```

### prepaint Implementation
```rust
fn prepaint(&mut self, bounds: Bounds<Pixels>, request_layout: &mut DivFrameState, ...) {
    // Calculate content size from child bounds
    for child_layout_id in &request_layout.child_layout_ids {
        let child_bounds = window.layout_bounds(*child_layout_id);
        // Track min/max for content size
    }
    
    self.interactivity.prepaint(
        ..., bounds, content_size, window, cx,
        |style, scroll_offset, hitbox, window, cx| {
            // Apply scroll offset when prepainting children
            window.with_element_offset(scroll_offset, |window| {
                for child in &mut self.children {
                    child.prepaint(window, cx);
                }
            });
            hitbox
        },
    )
}
```

### paint Implementation
```rust
fn paint(&mut self, ...) {
    self.interactivity.paint(
        ..., bounds, hitbox, window, cx,
        |style, window, cx| {
            for child in &mut self.children {
                child.paint(window, cx);
            }
        },
    )
}
```

## Pattern 2: VirtualList (Dynamic Children at Specific Positions)

The `VirtualList` shows how to render children at computed positions, similar to what the infinite canvas needs.

### Key Insight: `layout_as_root` + `prepaint_at`

For elements that need to position children at arbitrary locations (not via flexbox):

```rust
fn prepaint(&mut self, ..., layout: &mut VirtualListFrameState, ...) {
    // Calculate which items are visible
    let visible_range = first_visible..last_visible;
    
    // Create child elements on demand
    let items = (self.render_items)(visible_range.clone(), window, cx);
    
    window.with_content_mask(Some(content_mask), |window| {
        for (mut item, ix) in items.into_iter().zip(visible_range) {
            // Calculate item origin based on scroll and item position
            let item_origin = content_bounds.origin 
                + point(scroll_offset.x, item_origins[ix] + scroll_offset.y);
            
            let available_space = size(
                AvailableSpace::Definite(content_bounds.size.width),
                AvailableSpace::Definite(item_sizes[ix]),
            );

            // Layout the item as a root element
            item.layout_as_root(available_space, window, cx);
            
            // Prepaint at the calculated position
            item.prepaint_at(item_origin, window, cx);
            
            // Store for painting later
            layout.items.push(item);
        }
    });
}

fn paint(&mut self, layout: &mut VirtualListFrameState, ...) {
    for item in &mut layout.items {
        item.paint(window, cx);
    }
}
```

## Pattern 3: Anchored (Absolute Positioning)

The `Anchored` element positions children at absolute window coordinates.

### Key Methods Used

```rust
fn prepaint(&mut self, bounds: Bounds<Pixels>, request_layout: &mut AnchoredState, ...) {
    // Children were laid out normally in request_layout
    // Now compute where they should actually appear
    let offset = desired.origin - bounds.origin;
    
    // Use element offset to position children
    window.with_element_offset(offset, |window| {
        for child in &mut self.children {
            child.prepaint(window, cx);
        }
    });
}
```

## Pattern 4: Deferred (Delayed Rendering)

The `Deferred` element participates in layout but defers painting.

```rust
fn request_layout(&mut self, ...) -> (LayoutId, ()) {
    // Child participates in normal layout
    let layout_id = self.child.as_mut().unwrap().request_layout(window, cx);
    (layout_id, ())
}

fn prepaint(&mut self, ...) {
    let child = self.child.take().unwrap();
    let element_offset = window.element_offset();
    // Defer draw to later (after all normal elements)
    window.defer_draw(child, element_offset, self.priority)
}

fn paint(&mut self, ...) {
    // Nothing - painting happens via defer_draw
}
```

## Key Window Methods

### Layout Methods
- `window.request_layout(style, child_layout_ids, cx)` - Request layout with children
- `window.request_measured_layout(style, measure_fn)` - Request layout with custom measurement
- `window.layout_bounds(layout_id)` - Get computed bounds for a layout ID
- `window.compute_layout(layout_id, available_space, cx)` - Force layout computation

### Offset/Position Methods
- `window.with_element_offset(offset, |window| ...)` - Apply offset to nested elements
- `window.with_absolute_element_offset(origin, |window| ...)` - Set absolute position
- `window.element_offset()` - Get current element offset

### Content Mask
- `window.with_content_mask(Some(ContentMask { bounds }), |window| ...)` - Clip rendering

### Deferred Drawing  
- `window.defer_draw(element, offset, priority)` - Defer element painting

## Implications for Infinite Canvas

### Option A: Children in Layout Phase

Store children in `CanvasItem` and handle them through normal layout:

```rust
struct CanvasItem {
    id: ItemId,
    bounds: Bounds<Pixels>,  // Position in canvas space
    child: Option<AnyElement>,
    // ...
}

struct CanvasFrameState {
    item_layout_ids: Vec<LayoutId>,
    items_with_children: Vec<(usize, AnyElement)>,  // Index + prepared element
}
```

**Pros**: Follows GPUI patterns exactly
**Cons**: Children would participate in layout engine (may conflict with canvas positioning)

### Option B: Dynamic Children via `layout_as_root` (Recommended)

Like VirtualList, create and position children during prepaint:

```rust
struct CanvasFrameState {
    prepared_items: Vec<PreparedCanvasItem>,
}

struct PreparedCanvasItem {
    screen_bounds: Bounds<Pixels>,
    child: Option<AnyElement>,
}

fn prepaint(&mut self, ...) {
    for item in visible_items {
        let screen_bounds = camera.canvas_to_screen_bounds(item.bounds);
        
        if let Some(mut child) = create_child_for_item(item) {
            let available_space = size(
                AvailableSpace::Definite(screen_bounds.size.width),
                AvailableSpace::Definite(screen_bounds.size.height),
            );
            
            // Layout child with item's screen size
            child.layout_as_root(available_space, window, cx);
            
            // Position at screen location
            child.prepaint_at(screen_bounds.origin, window, cx);
            
            frame_state.prepared_items.push(PreparedCanvasItem {
                screen_bounds,
                child: Some(child),
            });
        }
    }
}

fn paint(&mut self, frame_state: &mut CanvasFrameState, ...) {
    // Paint backgrounds first
    for item in &frame_state.prepared_items {
        paint_item_background(item.screen_bounds, window);
    }
    
    // Then paint children
    for item in &mut frame_state.prepared_items {
        if let Some(child) = &mut item.child {
            child.paint(window, cx);
        }
    }
}
```

### Option C: Render Callback (Simplest)

Provide a callback that creates elements:

```rust
pub fn render_item_with<F>(mut self, renderer: F) -> Self
where
    F: Fn(&CanvasItem<D>, &Camera, &mut Window, &mut App) -> AnyElement + 'static
```

The canvas calls this during prepaint for visible items, then uses `layout_as_root` + `prepaint_at`.

## Recommended Implementation

Use **Option B** combined with **Option C**:

1. Store a render callback in `InfiniteCanvas`
2. During prepaint, for each visible item:
   - Call the render callback to create an `AnyElement`
   - Call `layout_as_root(available_space, ...)` to compute layout
   - Call `prepaint_at(screen_origin, ...)` to position it
   - Store the prepared element in frame state
3. During paint:
   - Paint item backgrounds
   - Call `paint(window, cx)` on each prepared child element

This approach:
- Follows VirtualList patterns (proven to work)
- Supports arbitrary GPUI elements as children
- Properly handles the three-phase lifecycle
- Allows for efficient viewport culling
- Scales with zoom (elements re-layout when size changes)

## Code References

- `vendor/gpui/src/elements/div.rs` - Standard child handling (lines 1370-1534)
- `vendor/gpui/src/elements/anchored.rs` - Absolute positioning
- `vendor/gpui/src/elements/deferred.rs` - Deferred rendering
- `vendor/gpui/src/element.rs` - AnyElement and Element trait (lines 574-651)
- `vendor/gpui-component/crates/ui/src/virtual_list.rs` - Dynamic children (lines 343-752)