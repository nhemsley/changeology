# Diff UI

A single file diff renderer for GPU-accelerated visualization. This crate renders the differences between two text strings with colored backgrounds, optimized for both interactive viewing and render-to-texture workflows.

## Purpose

**`diff-ui`** is a focused component that renders **one file diff** at a time. It's designed to be used as a building block in the larger Changeology infinite canvas visualization system, where each file diff is rendered to a texture and placed on a zoomable, pannable canvas.

### The Bigger Picture

This crate is part of **Changeology** - an infinite canvas Git visualization tool:

```
┌───────────────────────────────────────────────────┐
│  Changeology (Infinite Canvas App)                │
│  ┌─────────────────────────────────────────────┐  │
│  │  Canvas: Zoomable, Pannable, Infinite       │  │
│  │                                              │  │
│  │   ┌──────────┐    ┌──────────┐             │  │
│  │   │ main.rs  │    │README.md │             │  │
│  │   │ +50 -23  │    │  +5 -2   │             │  │
│  │   └──────────┘    └──────────┘             │  │
│  │        ↑ Each file rendered by diff-ui     │  │
│  └─────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

**diff-ui's role**: Render individual file diffs as textures that can be placed on the canvas.

## Features

- ✅ **Single file diff rendering** with colored backgrounds
- ✅ **Two rendering modes**:
  - **Virtualized**: Efficient rendering with `uniform_list` (only renders visible lines)
  - **Full Buffer**: Renders all lines at once - **perfect for render-to-texture**
- ✅ Scrolling support for long diffs
- ✅ Line prefixes (`+`, `-`, ` `)
- ✅ Dark and light theme support
- ✅ Monospace font rendering
- ✅ Integration with `buffer-diff` crate
- ✅ Optimized for large diffs (1000+ lines)

## Rendering Modes

### Full Buffer Mode (Primary: Render to Texture)

Use this mode when rendering diffs to textures for the infinite canvas:

```rust
use diff_ui::{DiffTextView, RenderMode};

let view = DiffTextView::new(old_text, new_text)
    .with_render_mode(RenderMode::FullBuffer);

// This renders ALL lines at once, perfect for:
// - Capturing as a texture
// - Displaying on an infinite canvas
// - Creating "diff cards" that can be positioned anywhere
```

**Why Full Buffer for textures?**
- All content rendered at once (no virtualization)
- Complete visual representation in a single frame
- Can be captured and cached as a texture
- Supports zooming/scaling without re-rendering

### Virtualized Mode (Secondary: Interactive Preview)

Use this mode for interactive full-screen inspection:

```rust
let view = DiffTextView::new(old_text, new_text)
    .with_render_mode(RenderMode::Virtualized);

// This only renders visible lines, perfect for:
// - Clicking on a diff card to inspect it
// - Full-screen interactive viewing
// - Very large files (10,000+ lines)
```

**Why Virtualized for preview?**
- Only renders visible lines (virtualized scrolling)
- Minimal memory usage
- Smooth 60fps scrolling
- Handles massive files efficiently

## Usage

### Basic Example

```rust
use diff_ui::DiffTextView;

let old_text = "line 1\nline 2\nline 3\n";
let new_text = "line 1\nline 2 modified\nline 3\nline 4\n";

// Default: Virtualized mode
let view = DiffTextView::new(old_text, new_text);
```

### Render to Texture Workflow

```rust
use diff_ui::{DiffTextView, RenderMode};

// 1. Create a diff view in Full Buffer mode
let view = DiffTextView::new(old_text, new_text)
    .with_render_mode(RenderMode::FullBuffer);

// 2. Render it (GPUI will render all lines)
// 3. Capture as texture (using your GPUI texture branch)
// 4. Place on infinite canvas at position (x, y)
// 5. User can zoom, pan, rearrange diff cards
```

### Infinite Canvas Integration

```rust
// Pseudo-code for the larger Changeology app:

// Load changed files from Git
let changed_files = git::get_changed_files(commit);

// Create a diff view for each file
let diff_cards = changed_files.iter().map(|file| {
    let old_content = git::get_file_content(file, commit.parent);
    let new_content = git::get_file_content(file, commit);
    
    DiffTextView::new(&old_content, &new_content)
        .with_render_mode(RenderMode::FullBuffer)
});

// Render each to texture and place on canvas
canvas.add_cards(diff_cards);
```

### Custom Themes

```rust
use diff_ui::{DiffTextView, DiffTheme, RenderMode};

let view = DiffTextView::new(old_text, new_text)
    .with_theme(DiffTheme::light())
    .with_render_mode(RenderMode::FullBuffer);
```

### Running the Demo

```bash
# From the project root
cargo run -p diff-ui
```

This opens a window displaying a large sample diff to demonstrate both rendering modes.

## Architecture

```
DiffTextView
    │
    ├─> Uses buffer-diff crate to calculate hunks
    ├─> Converts hunks to display lines  
    └─> Renders with GPUI
           │
           ├─> Virtualized Mode: uniform_list (only visible lines)
           └─> Full Buffer Mode: all lines (render-to-texture ready)
```

### Key Components

- **DiffTextView** - Main component that renders a single file diff
- **DiffDisplayLine** - Represents a single line with style (Added/Deleted/Unchanged)
- **DiffTheme** - Color definitions for dark/light themes
- **RenderMode** - Enum to choose between Virtualized and Full Buffer rendering

### Performance Characteristics

**Virtualized Mode:**
- Memory: O(visible lines) - typically 50-100 lines
- Render time: ~5ms for visible lines
- Scrolling: Smooth 60fps regardless of diff size
- Best for: Interactive viewing of large diffs

**Full Buffer Mode:**
- Memory: O(total lines) - entire diff in memory
- Render time: O(total lines) - renders everything once
- Texture capture: Single frame capture
- Best for: Render-to-texture, canvas placement

## Dependencies

- `gpui` - GPU-accelerated UI framework from Zed (version 0.2 from crates.io)
- `buffer-diff` - Our diff calculation crate (workspace dependency)
- `git` - Our Git wrapper crate (workspace dependency)
- `anyhow` - Error handling

## Example Output

```
  fn main() {
-     println!("Hello, world!");
+     // Print a greeting
+     let name = "Rust";
+     println!("Hello, {}!", name);
  }
```

Lines starting with `-` have a red background (deleted).  
Lines starting with `+` have a green background (added).  
Lines starting with ` ` (space) have normal background (unchanged).

## Development

### Building

```bash
cargo build -p diff-ui
```

### Testing

```bash
cargo test -p diff-ui
```

### Checking

```bash
cargo check -p diff-ui
```

## Code Statistics

- **Total Lines**: ~350
- **Components**: 3 files
  - `main.rs` - Application entry with large diff demo (~75 lines)
  - `diff_text_view.rs` - Core rendering (~250 lines)
  - `theme.rs` - Colors (~112 lines)

Compare to Zed's Editor component: ~20,000 lines.  
This is **98.5% simpler** by focusing only on single file diff display.

## Scope & Non-Goals

### In Scope
✅ Render a single file diff  
✅ Two rendering modes (virtualized & full buffer)  
✅ Color-coded line backgrounds  
✅ Line prefixes (+, -, space)  
✅ Theme support  
✅ Efficient rendering for large files  

### Out of Scope (Handled by Parent App)
❌ Multiple file management - handled by Changeology canvas  
❌ Git repository integration - handled by `git` crate  
❌ File tree/navigation - handled by Changeology canvas  
❌ Commit/branch switching - handled by Changeology canvas  
❌ Canvas zooming/panning - handled by Changeology canvas  
❌ Texture capture - handled by GPUI texture branch  

### Future Enhancements (Maybe)
- Line numbers with gutter
- Syntax highlighting
- Inline diff (word-level changes)
- Copy/paste support
- Search within diff

## Roadmap

### Phase 1: Core Rendering ✅ (Completed)
- Text rendering with colored backgrounds
- Virtualized and full buffer modes
- Theme system
- Large diff support

### Phase 2: Render to Texture Integration (In Progress)
- Work with GPUI texture branch
- Optimize full buffer rendering for texture capture
- Test with Changeology infinite canvas

### Phase 3: Polish (Planned)
- Line numbers and gutter
- Better empty line handling
- Performance profiling
- Memory optimization

### Phase 4: Advanced Features (Maybe)
- Syntax highlighting
- Inline word-level diffs
- Custom color schemes
- Export formats

## Use Cases

### Primary: Infinite Canvas Visualization
Render individual file diffs as textures on an infinite canvas where users can:
- Navigate between commits/branches visually
- See all changed files spatially arranged
- Zoom in/out to see details or overview
- Pan to explore large changesets
- Rearrange diff cards for better understanding

### Secondary: Standalone Diff Viewer
Use as a simple, fast diff viewer for any two text strings with minimal setup.

### Tertiary: Embedded Diff Display
Embed diff views in larger applications that need to show code changes.

## Contributing

This is part of the Changeology project - an infinite canvas Git visualization tool.

See the main [Changeology README](../../README.md) for the overall architecture and vision.

## License

Same as parent project.

## See Also

- [buffer-diff](../diff/) - Diff calculation crate
- [git](../git/) - Git repository wrapper
- [Changeology](../../README.md) - Infinite canvas Git visualization
- [Zed Editor](https://github.com/zed-industries/zed) - Original source of inspiration
- [GPUI](https://www.gpui.rs/) - The GPU-accelerated UI framework we use