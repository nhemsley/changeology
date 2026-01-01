# Changeology

An infinite canvas Git visualization tool. Explore your code history spatially - navigate commits, branches, and file changes on a zoomable, pannable infinite canvas where each file diff is rendered as a moveable card.

## Vision

Traditional Git tools show changes linearly - commits in a list, diffs in a pane. **Changeology** reimagines Git visualization as a spatial experience:

```
┌────────────────────────────────────────────────────────────────┐
│  Changeology - Infinite Canvas                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  🔍 Zoom: 75%        📍 Commit: abc123f                  │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │                                                           │  │
│  │     ┌──────────────┐      ┌──────────────┐              │  │
│  │     │ src/main.rs  │      │ src/lib.rs   │              │  │
│  │     │              │      │              │              │  │
│  │     │ + 50 lines   │      │ + 102 lines  │              │  │
│  │     │ - 23 lines   │      │ - 45 lines   │              │  │
│  │     │              │      │              │              │  │
│  │     └──────────────┘      └──────────────┘              │  │
│  │                                                           │  │
│  │                    ┌──────────────┐                      │  │
│  │                    │ README.md    │                      │  │
│  │                    │              │                      │  │
│  │                    │ + 5 lines    │                      │  │
│  │                    │ - 2 lines    │                      │  │
│  │                    │              │                      │  │
│  │                    └──────────────┘                      │  │
│  │                                                           │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  [← Prev Commit] [Branch: main ▼] [Next Commit →]             │
└────────────────────────────────────────────────────────────────┘
```

**Each file diff is a card you can:**
- 🔍 Zoom to see details or pull back for overview
- 🎯 Click to inspect full-screen with virtualized scrolling
- 📍 Arrange spatially to understand relationships
- 🌊 Navigate through commits like flowing through time
- 🌳 Explore branches as parallel universes

## Architecture

Changeology is built as a modular Rust workspace with focused, reusable components:

### Crates

#### 1. **[git](crates/git/README.md)** - Git Repository Interface
A clean wrapper around `git2` providing:
- Repository status and file changes
- Commit/branch navigation
- File content retrieval at any commit
- Simple API for common Git operations

```rust
use git::Repository;

let repo = Repository::open(".")?;
let changed_files = repo.get_changed_files()?;
```

#### 2. **[buffer-diff](crates/diff/README.md)** - Diff Calculation Engine
Calculates differences between text documents with:
- Hunk-based diff representation
- Line-level granularity (Added/Deleted/Modified/Unchanged)
- Context-aware diff presentation
- Efficient algorithms for large files

```rust
use buffer_diff::TextDiff;

let diff = TextDiff::diff(&old_text, &new_text)?;
```

#### 3. **[diff-ui](crates/diff-ui/README.md)** - Single File Diff Renderer
GPU-accelerated rendering of individual file diffs:
- **Full Buffer Mode**: Renders entire diff for texture capture (canvas cards)
- **Virtualized Mode**: Efficient scrolling for full-screen inspection
- Color-coded lines (green=added, red=deleted)
- Optimized for both small and massive files (10,000+ lines)

```rust
use diff_ui::{DiffTextView, RenderMode};

// Render to texture for canvas
let view = DiffTextView::new(&old_text, &new_text)
    .with_render_mode(RenderMode::FullBuffer);
```

#### 4. **changeology** (Main Application) - Infinite Canvas Integration
Ties everything together:
- Infinite zoomable/pannable canvas powered by GPUI
- Renders each file diff to texture using `diff-ui`
- Places diff textures as cards on the canvas
- Git navigation (commits, branches, timeline)
- Spatial layout and interaction

## Features

### Current (Phase 1)
- ✅ Single file diff rendering with GPU acceleration
- ✅ Two rendering modes (virtualized & full buffer)
- ✅ Git repository wrapper with status/content retrieval
- ✅ Efficient diff calculation with hunk representation
- ✅ Dark/light themes
- ✅ Optimized for large files

### In Development (Phase 2)
- 🚧 GPUI render-to-texture integration
- 🚧 Infinite canvas system (zoom, pan, drag)
- 🚧 Commit navigation (prev/next)
- 🚧 Multiple file diff cards on canvas

### Planned (Phase 3+)
- 📋 Branch visualization and switching
- 📋 Commit timeline navigation
- 📋 Diff card arrangement and layout
- 📋 Line numbers and gutter
- 📋 Search across diffs
- 📋 Syntax highlighting
- 📋 File tree navigation
- 📋 Staged vs unstaged visualization

## Getting Started

### Prerequisites
- Rust 1.70+ ([Install Rustup](https://rustup.rs/))
- Git 2.0+

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/changeology.git
cd changeology

# Build all crates
cargo build

# Run the diff-ui demo (single file diff viewer)
cargo run -p diff-ui

# Run the main app (coming soon)
cargo run
```

### Quick Start: Using the Crates

```rust
use git::Repository;
use buffer_diff::TextDiff;
use diff_ui::{DiffTextView, RenderMode};

// 1. Open a Git repository
let repo = Repository::open(".")?;

// 2. Get changed files
let changed_files = repo.get_changed_files()?;

// 3. For each file, get old and new content
for file in changed_files {
    let old_content = repo.get_file_content_at_head(&file.path)?;
    let new_content = std::fs::read_to_string(&file.path)?;
    
    // 4. Create a diff view
    let diff_view = DiffTextView::new(&old_content, &new_content)
        .with_render_mode(RenderMode::FullBuffer);
    
    // 5. Render to texture and place on canvas (coming soon)
    // canvas.add_card(diff_view, position);
}
```

## Development

### Project Structure

```
changeology/
├── crates/
│   ├── git/              # Git repository wrapper
│   ├── diff/             # Diff calculation (buffer-diff)
│   ├── diff-ui/          # Single file diff renderer
│   └── changeology/      # Main canvas app (coming soon)
├── Cargo.toml            # Workspace configuration
└── README.md             # This file
```

### Running Tests

```bash
# Test all crates
cargo test

# Test a specific crate
cargo test -p git
cargo test -p buffer-diff
cargo test -p diff-ui
```

### Building for Release

```bash
cargo build --release
```

## Technology Stack

- **[GPUI](https://www.gpui.rs/)** - GPU-accelerated UI framework from Zed
  - Blazing fast rendering
  - Texture capture for canvas cards
  - Efficient virtualized lists
  
- **[git2-rs](https://github.com/rust-lang/git2-rs)** - Rust bindings to libgit2
  - Repository access
  - Commit/branch navigation
  
- **[similar](https://github.com/mitsuhiko/similar)** - Text diffing library
  - Fast diff algorithms
  - Line-based comparison
  
- **[ropey](https://github.com/cessen/ropey)** - Rope data structure
  - Efficient text editing
  - Large file handling

## Design Philosophy

### Focused Components
Each crate does ONE thing well:
- `git` → Git operations
- `buffer-diff` → Diff calculation
- `diff-ui` → Single file rendering
- `changeology` → Canvas integration

### Spatial Not Linear
Traditional Git UIs are constrained by linear layouts. Changeology embraces 2D space:
- Arrange diffs to show relationships
- Zoom out for overview, zoom in for details
- Navigate naturally, not just up/down

### Performance First
Built for real-world codebases:
- Virtualized rendering for large files
- Efficient diff algorithms
- GPU acceleration
- Smart texture caching

### Extracted from Zed
We learned from [Zed](https://zed.dev/)'s excellent diff visualization and extracted the core concepts into standalone, modular components.

## Inspiration

- **[Zed Editor](https://zed.dev/)** - Fast, collaborative editor with great Git integration
- **[Miro](https://miro.com/)** / **[FigJam](https://www.figma.com/figjam/)** - Infinite canvas collaboration
- **[Obsidian Canvas](https://obsidian.md/)** - Spatial knowledge management
- **[GitKraken](https://www.gitkraken.com/)** - Visual Git client
- **[Sourcetree](https://www.sourcetreeapp.com/)** - Git visualization

## Contributing

Changeology is in active development. Contributions welcome!

### Areas to Contribute
- 🎨 Canvas rendering and interaction
- 🔧 GPUI texture integration
- 📝 Documentation and examples
- 🐛 Bug fixes and testing
- 💡 Feature ideas and design

## Roadmap

### Milestone 1: Core Diff Rendering ✅
- [x] Git repository wrapper
- [x] Diff calculation engine
- [x] Single file diff renderer
- [x] Two rendering modes

### Milestone 2: Canvas Foundation 🚧
- [ ] GPUI render-to-texture
- [ ] Infinite canvas implementation
- [ ] Zoom and pan controls
- [ ] Diff card placement

### Milestone 3: Git Navigation 📋
- [ ] Commit switching (prev/next)
- [ ] Branch visualization
- [ ] Timeline navigation
- [ ] Staged vs unstaged

### Milestone 4: Polish 📋
- [ ] Line numbers and gutter
- [ ] Syntax highlighting
- [ ] Search and filtering
- [ ] Performance optimization
- [ ] Custom themes

## License

[License TBD]

## See Also

- [diff-ui README](crates/diff-ui/README.md) - Single file diff renderer
- [buffer-diff README](crates/diff/README.md) - Diff calculation engine
- [git README](crates/git/README.md) - Git repository wrapper
- [GPUI Documentation](https://www.gpui.rs/)
- [Zed Editor](https://github.com/zed-industries/zed)

---

**Status**: 🚧 Active Development - Phase 1 (Core Diff Rendering) Complete, Phase 2 (Canvas) In Progress

Built with ❤️ and Rust 🦀