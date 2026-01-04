# Infinite Canvas in GPUI - Feasibility Research

## Overview

This document explores the feasibility of implementing an infinite canvas component in GPUI, suitable for adding to gpui-component. The goal is to create a pannable, zoomable canvas that can display and layout rectangular objects.

---

## Key Concepts from tldraw

tldraw provides an excellent reference implementation for infinite canvas. Key concepts:

### Camera System

- **Camera State**: `{x, y, z}` where z is zoom level
- **Two Coordinate Systems**:
  - **Screen Space**: Pixels relative to viewport top-left
  - **Page Space**: Coordinates on the infinite canvas
- **Coordinate Conversion**:
  - `screenToPage(point)` - Convert screen coords to canvas coords
  - `pageToScreen(point)` - Convert canvas coords to screen coords

### Camera Options (from tldraw)

| Option | Description |
|--------|-------------|
| `wheelBehavior` | `'pan'`, `'zoom'`, or `'none'` |
| `panSpeed` | Multiplier for pan speed (default 1.0) |
| `zoomSpeed` | Multiplier for zoom speed (default 1.0) |
| `zoomSteps` | Array of discrete zoom levels `[0.1, 0.25, 0.5, 1, 2, 4, 8]` |
| `isLocked` | Prevents camera movement |
| `constraints.bounds` | Limit canvas to specific bounds |
| `constraints.behavior` | `'free'`, `'inside'`, `'outside'`, `'fixed'`, `'contain'` |

### Viewport Concepts

```
┌─────────────────────────────────────────────────┐
│                 Infinite Canvas                  │
│                                                  │
│     ┌──────────────────────┐                    │
│     │      Viewport        │                    │
│     │   (visible area)     │                    │
│     │                      │                    │
│     │   ┌────┐  ┌────┐    │                    │
│     │   │Rect│  │Rect│    │                    │
│     │   └────┘  └────┘    │                    │
│     │                      │                    │
│     └──────────────────────┘                    │
│                                                  │
└─────────────────────────────────────────────────┘
```

---

## GPUI Building Blocks

### Existing Components in gpui-component

#### 1. Scroll System (`scroll/`)

The existing scroll system provides:
- `ScrollHandle` - Tracks scroll position
- `Scrollbar` - Visual scrollbar component
- `ScrollableElement` trait - Makes elements scrollable

**Limitation**: Traditional scrolling assumes finite content bounds. Infinite canvas needs unbounded panning.

#### 2. DockArea with Tiles (`dock/`)

The `tiles.rs` example shows:
- Free positioning of panels with `Bounds`
- Panels can be placed at arbitrary `(x, y)` positions
- Uses `DockItem::tiles(panels, bounds, ...)` pattern

```rust
// From tiles.rs example
let bounds = (0..PANELS)
    .map(|i| {
        let x = start_x + (panel_width + gap) * col as f32;
        let y = start_y + (panel_height + gap) * row as f32;
        Bounds::new(point(x, y), size(panel_width, panel_height))
    })
    .collect::<Vec<_>>();

DockItem::tiles(panels, bounds, dock_area, window, cx)
```

#### 3. Transform Support (`icon.rs`)

GPUI supports transforms on elements:
```rust
pub fn transform(mut self, transformation: gpui::Transformation) -> Self
```

This could be used for zoom transformations.

---

## Proposed Architecture

### Core Components

#### 1. `InfiniteCanvas` Component

```rust
#[derive(IntoElement)]
pub struct InfiniteCanvas<C: CanvasContent> {
    id: ElementId,
    camera: Camera,
    content: C,
    options: CanvasOptions,
}
```

#### 2. `Camera` State

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct Camera {
    /// Pan offset in canvas space
    pub offset: Point<Pixels>,
    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
}

impl Camera {
    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_point: Point<Pixels>) -> Point<Pixels> {
        Point::new(
            (screen_point.x - self.offset.x) / self.zoom,
            (screen_point.y - self.offset.y) / self.zoom,
        )
    }
    
    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_point: Point<Pixels>) -> Point<Pixels> {
        Point::new(
            canvas_point.x * self.zoom + self.offset.x,
            canvas_point.y * self.zoom + self.offset.y,
        )
    }
}
```

#### 3. `CanvasOptions`

```rust
pub struct CanvasOptions {
    /// Minimum zoom level
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
    /// Discrete zoom steps for zoom in/out actions
    pub zoom_steps: Vec<f32>,
    /// Pan speed multiplier
    pub pan_speed: f32,
    /// Zoom speed multiplier
    pub zoom_speed: f32,
    /// Whether to show grid
    pub show_grid: bool,
    /// Grid spacing in canvas units
    pub grid_size: f32,
}

impl Default for CanvasOptions {
    fn default() -> Self {
        Self {
            min_zoom: 0.1,
            max_zoom: 8.0,
            zoom_steps: vec![0.1, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0],
            pan_speed: 1.0,
            zoom_speed: 1.0,
            show_grid: true,
            grid_size: 20.0,
        }
    }
}
```

#### 4. `CanvasItem` - Rectangular Objects

```rust
/// A rectangular item on the canvas
#[derive(Clone, Debug)]
pub struct CanvasItem<D> {
    pub id: ItemId,
    pub bounds: Bounds<Pixels>,
    pub data: D,
}

impl<D> CanvasItem<D> {
    pub fn new(id: impl Into<ItemId>, bounds: Bounds<Pixels>, data: D) -> Self {
        Self {
            id: id.into(),
            bounds,
            data,
        }
    }
    
    pub fn position(&self) -> Point<Pixels> {
        self.bounds.origin
    }
    
    pub fn size(&self) -> Size<Pixels> {
        self.bounds.size
    }
}
```

---

## Layout Algorithms for Rectangular Objects

### 1. Grid Layout

Simple grid arrangement:

```rust
pub fn layout_grid(
    items: &mut [CanvasItem<impl Any>],
    columns: usize,
    cell_size: Size<Pixels>,
    gap: Pixels,
    start: Point<Pixels>,
) {
    for (i, item) in items.iter_mut().enumerate() {
        let row = i / columns;
        let col = i % columns;
        item.bounds.origin = Point::new(
            start.x + (cell_size.width + gap) * col as f32,
            start.y + (cell_size.height + gap) * row as f32,
        );
    }
}
```

### 2. Treemap Layout

Space-filling layout based on item weights:

```rust
pub fn layout_treemap(
    items: &mut [CanvasItem<impl HasWeight>],
    bounds: Bounds<Pixels>,
    direction: TreemapDirection,
) {
    // Squarified treemap algorithm
    // 1. Sort items by weight (descending)
    // 2. Recursively partition space
    // 3. Alternate horizontal/vertical splits
}

pub enum TreemapDirection {
    Horizontal,
    Vertical,
    Squarified, // Best aspect ratios
}
```

### 3. Force-Directed Layout

Physics-based layout for graph-like structures:

```rust
pub struct ForceLayoutOptions {
    pub repulsion: f32,      // Node repulsion strength
    pub attraction: f32,     // Edge attraction strength  
    pub damping: f32,        // Velocity damping
    pub iterations: usize,   // Simulation iterations
}

pub fn layout_force_directed(
    items: &mut [CanvasItem<impl Any>],
    edges: &[(usize, usize)],
    options: ForceLayoutOptions,
) {
    // 1. Initialize velocities
    // 2. For each iteration:
    //    a. Calculate repulsion forces between all nodes
    //    b. Calculate attraction forces along edges
    //    c. Apply damping
    //    d. Update positions
}
```

### 4. Hierarchical Tree Layout

For parent-child relationships (like file trees):

```rust
pub enum TreeLayoutStyle {
    /// Top-down tree
    TopDown,
    /// Left-to-right tree
    LeftToRight,
    /// Radial tree from center
    Radial,
    /// Cone tree (3D projection)
    ConeTree,
}

pub fn layout_tree(
    root: &mut TreeNode<CanvasItem<impl Any>>,
    style: TreeLayoutStyle,
    node_spacing: Size<Pixels>,
    level_spacing: Pixels,
) {
    match style {
        TreeLayoutStyle::TopDown => layout_tree_vertical(root, ...),
        TreeLayoutStyle::LeftToRight => layout_tree_horizontal(root, ...),
        TreeLayoutStyle::Radial => layout_tree_radial(root, ...),
        TreeLayoutStyle::ConeTree => layout_tree_cone(root, ...),
    }
}
```

### 5. Pack Layout

Circle packing / rectangle packing:

```rust
pub fn layout_pack(
    items: &mut [CanvasItem<impl Any>],
    container: Bounds<Pixels>,
    padding: Pixels,
) {
    // Binary tree bin packing algorithm
    // Good for variable-sized rectangles
}
```

---

## Input Handling

### Mouse Events

```rust
impl InfiniteCanvas {
    fn handle_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut Context) {
        match event.button {
            MouseButton::Left => self.start_drag(event.position),
            MouseButton::Middle => self.start_pan(event.position),
            MouseButton::Right => self.show_context_menu(event.position),
        }
    }
    
    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context) {
        if self.is_panning {
            let delta = event.position - self.last_mouse_pos;
            self.camera.offset += delta;
            cx.notify();
        }
    }
    
    fn handle_scroll(&mut self, event: &ScrollWheelEvent, cx: &mut Context) {
        // Zoom towards cursor position
        let cursor_canvas = self.camera.screen_to_canvas(event.position);
        
        let zoom_delta = event.delta.y * self.options.zoom_speed * 0.001;
        let new_zoom = (self.camera.zoom + zoom_delta)
            .clamp(self.options.min_zoom, self.options.max_zoom);
        
        // Adjust offset to zoom towards cursor
        self.camera.zoom = new_zoom;
        let new_cursor_screen = self.camera.canvas_to_screen(cursor_canvas);
        self.camera.offset += event.position - new_cursor_screen;
        
        cx.notify();
    }
}
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` + drag | Pan canvas |
| `Scroll` | Zoom in/out |
| `Cmd/Ctrl + 0` | Reset zoom to 100% |
| `Cmd/Ctrl + 1` | Zoom to fit |
| `Cmd/Ctrl + +` | Zoom in (step) |
| `Cmd/Ctrl + -` | Zoom out (step) |
| Arrow keys | Pan canvas |

---

## Rendering Strategy

### Viewport Culling

Only render items that intersect the viewport:

```rust
fn get_visible_items(&self) -> Vec<&CanvasItem<D>> {
    let viewport_canvas = Bounds::new(
        self.camera.screen_to_canvas(Point::zero()),
        Size::new(
            self.viewport_size.width / self.camera.zoom,
            self.viewport_size.height / self.camera.zoom,
        ),
    );
    
    self.items
        .iter()
        .filter(|item| item.bounds.intersects(&viewport_canvas))
        .collect()
}
```

### Layer Structure

```
┌─────────────────────────────────┐
│         Overlay Layer           │  <- UI, tooltips, selection
├─────────────────────────────────┤
│        Content Layer            │  <- Canvas items (transformed)
├─────────────────────────────────┤
│          Grid Layer             │  <- Background grid (transformed)
├─────────────────────────────────┤
│       Background Layer          │  <- Canvas background
└─────────────────────────────────┘
```

### Rendering Implementation

```rust
impl RenderOnce for InfiniteCanvas {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let camera = self.camera;
        let visible_items = self.get_visible_items();
        
        div()
            .id(self.id)
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(cx.theme().canvas_background)
            // Background grid
            .child(self.render_grid(window, cx))
            // Content layer with transform
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .style()
                    .transform(format!(
                        "translate({}px, {}px) scale({})",
                        camera.offset.x,
                        camera.offset.y,
                        camera.zoom
                    ))
                    .children(
                        visible_items.iter().map(|item| {
                            self.render_item(item, window, cx)
                        })
                    )
            )
            // Overlay layer
            .child(self.render_overlay(window, cx))
    }
}
```

---

## GPUI-Specific Considerations

### 1. Transform Support

GPUI has `Transformation` type but it's primarily for simple transforms. For canvas zoom/pan:

**Option A**: Use CSS transforms via style refinement
```rust
.style().transform(format!("scale({}) translate({}px, {}px)", ...))
```

**Option B**: Manually transform all coordinates before rendering
```rust
let screen_bounds = self.camera.canvas_to_screen_bounds(item.bounds);
div().absolute().left(screen_bounds.x).top(screen_bounds.y)...
```

**Recommendation**: Option B gives more control and avoids potential rendering issues.

### 2. Event Coordinate Handling

GPUI provides events in screen coordinates. Must convert to canvas coords:
```rust
fn on_click(&mut self, event: &ClickEvent, cx: &mut Context) {
    let canvas_pos = self.camera.screen_to_canvas(event.position);
    // Find item at canvas_pos
}
```

### 3. State Management

Use GPUI's entity system for canvas state:
```rust
struct CanvasState {
    camera: Camera,
    items: Vec<CanvasItem<D>>,
    selection: HashSet<ItemId>,
    drag_state: Option<DragState>,
}
```

### 4. Performance

- Use `uniform_list` for large numbers of similar items
- Implement spatial indexing (quadtree) for hit testing
- Batch similar items for rendering
- Use `cx.notify()` sparingly

---

## Implementation Plan

### Phase 1: Core Canvas
- [x] `Camera` struct with coordinate conversion
- [x] Basic `InfiniteCanvas` component
- [ ] Pan with middle mouse / space+drag ⚠️ **NEEDS FIX** - Requires stateful approach
- [ ] Zoom with scroll wheel ⚠️ **NEEDS FIX** - Event not triggering camera updates

### Phase 2: Items & Interaction
- [x] `CanvasItem` type
- [x] Render items with proper transforms (manual coordinate conversion)
- [x] Viewport culling
- [ ] Item selection

### Phase 3: Layout Algorithms
- [x] Grid layout
- [x] Tree layout (hierarchical - 4 orientations)
- [ ] Force-directed layout
- [ ] Treemap layout
- [x] Pack layout

### Phase 4: Polish
- [x] Background grid
- [ ] Minimap
- [ ] Zoom to fit
- [ ] Keyboard shortcuts
- [ ] Touch/trackpad gestures

### Current Issues
1. **State Management**: Mouse events in `paint()` can't mutate camera state
   - Need to refactor to use GPUI's stateful entity pattern
   - Or implement callback-based state management with parent view
2. **Zoom Not Working**: Scroll wheel events don't update camera
   - Callback fires but state doesn't persist
3. **Pan Not Implemented**: Middle-click drag needs stateful tracking
   - Requires drag start/end state and position tracking

---

## Open Questions

1. **Transform Method**: CSS transform vs manual coordinate conversion?
2. **Hit Testing**: Use GPUI's built-in or custom spatial index?
3. **Animation**: How to animate camera movements smoothly?
4. **Large Datasets**: Virtualization strategy for 10k+ items?
5. **Integration**: How to integrate with existing gpui-component theming?

---

## References

- [tldraw Editor API](https://tldraw.dev/docs/editor)
- [tldraw Camera Documentation](https://tldraw.dev/docs/editor#Camera)
- gpui-component `scroll/` module
- gpui-component `dock/tiles.rs` example
- [D3.js Zoom Behavior](https://github.com/d3/d3-zoom) (concept reference)
- [React Flow](https://reactflow.dev/) (concept reference)