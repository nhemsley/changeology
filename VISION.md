# Changeology Vision: Infinite Canvas Git Visualization

## The Big Idea

**Traditional Git visualization is constrained by linear thinking.** Commits in a list. Diffs in a pane. Files in a tree. These tools work, but they don't leverage the power of spatial reasoning that humans excel at.

**Changeology reimagines Git as a spatial, explorable universe.** Each file diff becomes a card on an infinite, zoomable canvas. Navigate commits like flowing through time. Explore branches as parallel dimensions. Arrange changes to reveal patterns and relationships.

## Core Concept: Render to Texture

### The Technical Foundation

Each file diff is rendered **once** to a GPU texture, then placed on the canvas as a moveable, zoomable card:

```
┌─────────────────────────────────────────────────────────┐
│  1. Load Git Changes                                    │
│     ↓                                                    │
│  2. For each file:                                      │
│     • Get old content (from commit parent)              │
│     • Get new content (from working tree or commit)     │
│     ↓                                                    │
│  3. Calculate Diff (buffer-diff crate)                  │
│     • Analyze line-by-line changes                      │
│     • Create hunk representation                        │
│     ↓                                                    │
│  4. Render to Texture (diff-ui crate)                   │
│     • Use Full Buffer mode (all lines at once)          │
│     • Capture as GPU texture                            │
│     • Cache the texture                                 │
│     ↓                                                    │
│  5. Place on Canvas                                     │
│     • Position at (x, y) coordinates                    │
│     • Apply zoom/pan transformations                    │
│     • User can drag, resize, rearrange                  │
└─────────────────────────────────────────────────────────┘
```

### Why This Works

**Performance**: Render once, display many times
- Texture is cached on GPU
- Zooming/panning is just transformation math
- No re-rendering needed until content changes

**Flexibility**: Textures can be:
- Scaled (zoom in/out)
- Translated (pan around)
- Arranged arbitrarily in 2D space
- Cached and reused

**Simplicity**: Clear separation of concerns
- `diff-ui` renders a single file diff
- Canvas manages layout and interaction
- No coupling between rendering and positioning

## The Two Rendering Modes

### Full Buffer Mode: Canvas Cards

**Purpose**: Render entire diff as a texture for canvas placement

```rust
let view = DiffTextView::new(&old_text, &new_text)
    .with_render_mode(RenderMode::FullBuffer);

// All lines rendered at once → capture as texture → place on canvas
```

**When**: 
- Creating diff cards for the canvas
- Initial load of changed files
- Switching commits/branches

**Characteristics**:
- Complete visual representation in single frame
- All lines present (no virtualization)
- Ready for texture capture
- Memory trade-off acceptable for typical file sizes

### Virtualized Mode: Full-Screen Inspection

**Purpose**: Interactive viewing when you click on a diff card

```rust
let view = DiffTextView::new(&old_text, &new_text)
    .with_render_mode(RenderMode::Virtualized);

// Only visible lines rendered → smooth scrolling
```

**When**:
- User clicks a diff card to inspect it
- Viewing very large files (10,000+ lines)
- Need smooth scrolling performance

**Characteristics**:
- Only renders visible lines (~50-100)
- Virtualized scrolling with `uniform_list`
- Minimal memory footprint
- 60fps smooth scrolling

## User Experience Flow

### 1. Opening a Repository

```
User: cargo run
  ↓
App: Opens current Git repository
  ↓
App: Detects changed files (unstaged)
  ↓
App: For each file, renders diff to texture
  ↓
Canvas: Displays diff cards in default layout
```

### 2. Navigating the Canvas

```
User Actions:
• Scroll/drag → Pan around the canvas
• Pinch/wheel → Zoom in/out
• Click card → Full-screen inspection (virtualized mode)
• Esc → Back to canvas
• Drag card → Rearrange spatial layout
```

### 3. Navigating Commits

```
User: Clicks "Prev Commit" button
  ↓
App: Loads parent commit
  ↓
App: Calculates new diffs (HEAD~ vs HEAD~~)
  ↓
App: Re-renders changed files to textures
  ↓
Canvas: Updates diff cards with smooth transition
```

### 4. Switching Branches

```
User: Selects branch from dropdown
  ↓
App: Checks out branch
  ↓
App: Calculates diff (HEAD vs branch)
  ↓
App: Renders all changed files
  ↓
Canvas: Shows branch diff spatially
```

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│  Layer 5: User Interface (GPUI)                         │
│  • Window management                                    │
│  • Input handling (mouse, keyboard, touch)              │
│  • Navigation controls                                  │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Infinite Canvas                               │
│  • Zoom/pan/drag logic                                  │
│  • Card positioning and layout                          │
│  • Texture rendering and caching                        │
│  • Spatial interaction                                  │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Diff Rendering (diff-ui crate)                │
│  • Single file diff visualization                       │
│  • Full Buffer mode → texture capture                   │
│  • Virtualized mode → interactive viewing               │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 2: Diff Calculation (buffer-diff crate)          │
│  • Line-by-line comparison                              │
│  • Hunk extraction                                      │
│  • Line type classification                             │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Git Access (git crate)                        │
│  • Repository operations                                │
│  • Commit/branch navigation                             │
│  • File content retrieval                               │
└─────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### Decision 1: One Diff View = One File

**Rationale**: Clear separation of concerns
- `diff-ui` is focused: renders a single file diff
- Canvas manages multiple files
- Easy to test, maintain, and reason about

**Alternative Considered**: Multi-file view in one component
- ❌ Too complex
- ❌ Tight coupling
- ❌ Harder to cache and optimize

### Decision 2: Render to Texture, Not Live DOM

**Rationale**: Performance and flexibility
- Render once, transform many times (zoom/pan is cheap)
- GPU handles scaling beautifully
- Can cache textures for instant navigation

**Alternative Considered**: Live DOM elements for each diff
- ❌ Re-renders on every zoom/pan
- ❌ Heavy memory with many files
- ❌ Jittery performance

### Decision 3: Two Rendering Modes

**Rationale**: Different use cases need different optimizations
- Canvas needs complete render (Full Buffer)
- Inspection needs efficiency (Virtualized)
- Both use same diff calculation

**Alternative Considered**: Only virtualized
- ❌ Can't easily capture to texture
- ❌ Complex to handle during zoom/pan

### Decision 4: Modular Crate Architecture

**Rationale**: Reusability and maintainability
- Each crate has clear, single purpose
- Can be used independently
- Easy to test in isolation
- Future: publish to crates.io

**Alternative Considered**: Monolithic app
- ❌ Hard to test
- ❌ Not reusable
- ❌ Tight coupling

## Future Possibilities

### Phase 3: Advanced Navigation

- **Commit Timeline**: Horizontal timeline showing commits
- **Branch Tree**: Visual branch/merge graph
- **Diff History**: See how a file evolved across commits
- **Search**: Find changes across all files

### Phase 4: Collaboration

- **Shared Canvas**: Multiple users on same canvas
- **Annotations**: Add notes/markers to diff cards
- **Comments**: Discuss specific changes
- **Reviews**: Visual code review workflow

### Phase 5: AI Integration

- **Smart Layout**: AI suggests optimal card arrangement
- **Change Summaries**: AI generates commit summaries
- **Pattern Detection**: Highlight related changes
- **Impact Analysis**: Show downstream effects

## Performance Targets

### Initial Load (100 files)
- Diff calculation: < 500ms
- Texture rendering: < 2s
- Total to interactive: < 3s

### Navigation
- Commit switch: < 1s
- Branch switch: < 2s
- Canvas pan/zoom: 60fps

### Memory
- Texture cache: ~10MB per file (typical)
- Max textures: ~1000 files = ~10GB (acceptable on modern hardware)
- LRU cache for older commits

## Success Metrics

### Usability
- Users can navigate commits faster than traditional tools
- Spatial layout helps understand change relationships
- Large changesets (100+ files) are manageable

### Performance
- 60fps canvas interaction
- Sub-second commit navigation
- Smooth zoom/pan regardless of file count

### Developer Experience
- Clear crate boundaries
- Easy to add new features
- Well-documented APIs
- Comprehensive tests

## Inspiration & Prior Art

### What We Learned From:

**Zed Editor**
- Fast, GPU-accelerated rendering
- Excellent diff visualization
- Clean component architecture

**Miro/FigJam**
- Infinite canvas UX patterns
- Zoom/pan interaction
- Card-based layouts

**Obsidian Canvas**
- Spatial knowledge management
- Simple, intuitive controls
- Note relationships

**GitKraken**
- Visual Git history
- Branch visualization
- Commit navigation

### What We're Building Differently:

- **Spatial not linear**: Leverage 2D space
- **Performance-first**: GPU acceleration, texture caching
- **Modular**: Reusable components
- **Rust**: Memory safe, blazing fast

## Conclusion

Changeology is more than a Git tool—it's a new way to think about code history. By embracing spatial reasoning and GPU acceleration, we can make understanding changes intuitive, fast, and even enjoyable.

The infinite canvas isn't just a gimmick; it's a fundamental rethinking of how we visualize and navigate code changes. When you can see all changes at once, arrange them meaningfully, and zoom between overview and detail, Git history transforms from a linear timeline into an explorable universe.

---

**Next Steps**: Complete GPUI render-to-texture integration and build the infinite canvas system.

**Vision Document Version**: 1.0  
**Last Updated**: 2024  
**Status**: Phase 1 Complete, Phase 2 In Progress