# Git Diff Visualization Extraction Project

This document outlines a plan for extracting and reusing the git diff visualization functionality from Zed, focusing on two primary use cases:
1. Viewing unstaged changes
2. Viewing diffs between arbitrary text versions

## Project Structure

```
changeology/
├── Cargo.toml               # Workspace manifest
├── crates/
│   ├── changeology/         # Integration crate / public API
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs       # Main library entry point
│   │
│   ├── diff/                # Core diff calculation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── buffer_diff.rs   # Core diff algorithm (extracted from Zed)
│   │       ├── diff_hunk.rs     # Diff hunk representation and status
│   │       └── text_diff.rs     # Text-based diff calculation
│   │
│   ├── git/                 # Git integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repository.rs    # Git repository operations
│   │       └── status.rs        # Git status representation
│   │
│   ├── ui/                  # UI components
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── diff_view.rs     # Component for displaying diffs
│   │       ├── status_view.rs   # Component for displaying git status
│   │       └── styling.rs       # Styling for diff display
│   │
│   └── util/                # Utility functions
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── text.rs          # Text handling utilities
│
├── examples/                # Example applications
│   ├── git_status.rs        # Show git status example
│   └── file_diff.rs         # Compare two files example
│
├── tests/                   # Integration tests
│
└── vendor/                  # Vendored code from Zed and gitui
    ├── zed/                 # Relevant Zed code for reference
    └── gitui/               # Relevant gitui code for reference
```

## Core Components to Extract

### 1. From Zed's `buffer_diff` Crate

The heart of the diff calculation logic from `crates/buffer_diff/src/buffer_diff.rs`:

- `BufferDiff`: Main class for diff calculation
- `BufferDiffSnapshot`: Immutable snapshot of diffs
- `DiffHunk`: Represents a chunk of changes
- `DiffHunkStatus`: Enum for diff status (added/deleted/modified)
- `DiffHunkSecondaryStatus`: Enum for staged/unstaged status
- `compute_hunks()`: Core algorithm to calculate diff hunks

### 2. From Zed's `git_ui` Crate

UI rendering and interaction from `crates/git_ui/src/project_diff.rs`:

- `ProjectDiff`: Main component for diff visualization
- Logic for rendering diff hunks with appropriate styling
- Staging/unstaging interaction handlers

### 3. From Zed's `git` Crate

Git integration from `crates/git/src/repository.rs` and `crates/git/src/status.rs`:

- File status representation and detection
- Repository operations for fetching content

## Implementation Plan

### Phase 1: Core Diff Library

1. Extract and adapt the core diff calculation from `buffer_diff.rs`
2. Remove dependencies on Zed's entity system and UI framework
3. Create clean interfaces for text representation and diff calculation
4. Implement tests to ensure correctness

### Phase 2: Git Integration

1. Create a simplified git integration layer
2. Extract status representation from Zed's git crate
3. Implement functions to get unstaged/staged changes
4. Add git content retrieval for different versions (HEAD, index, working)

### Phase 3: UI Components

1. Design a framework-agnostic representation of diff styling
2. Extract the diff visualization logic from `project_diff.rs`
3. Create adaptable UI components for different UI frameworks
4. Implement interaction handlers for staging/unstaging

### Phase 4: Examples and Documentation

1. Create example applications for both use cases
2. Document the API and usage patterns
3. Add comprehensive tests

## Dependency Considerations

### Required External Dependencies

- **git2**: For Git operations via libgit2
- **derive_more**: For deriving traits
- A text/rope library (e.g., **ropey** or **xi-rope**)

### Optional Dependencies

- UI framework adapters (for your specific UI framework)
- Syntax highlighting (if needed)

## Adapting Zed's Architecture

### Key Adaptations Needed

1. **Replace Entity System**:
   - Replace Zed's entity-based state management with simpler references or Arc
   - Remove dependencies on Context<T> and App

2. **Text Representation**:
   - Replace Zed's Rope and Buffer with a simpler text representation
   - Use a standard Rope implementation or simple String for smaller diffs

3. **UI Abstraction**:
   - Create trait-based abstractions for rendering
   - Allow adapters for different UI frameworks

4. **State Management**:
   - Simplify the reactive model used in Zed
   - Use a simpler observer pattern or callback-based approach

## Challenges and Considerations

1. **Zed's Tight Integration**: The diff functionality is tightly integrated with Zed's architecture, requiring careful extraction to maintain functionality.

2. **Git Integration**: The git operations need to be simplified while maintaining functionality.

3. **UI Framework Independence**: Create abstractions that work with various UI frameworks.

4. **Performance**: Maintain the efficiency of Zed's diff calculation while simplifying the implementation.

## Reference Code

The most important files to study from Zed:

1. `crates/buffer_diff/src/buffer_diff.rs` - Core diff calculation
2. `crates/git_ui/src/project_diff.rs` - Diff visualization
3. `crates/git/src/repository.rs` - Git operations
4. `crates/git/src/status.rs` - Git status representation

Additionally, relevant code from gitui can provide inspiration for alternative implementations.

## Usage Examples

### Viewing Unstaged Changes

```rust
// Example API usage (not final)
let repo = Repository::open("path/to/repo")?;
let status = repo.status()?;

// Get unstaged changes for a file
let file_path = "src/main.rs";
let file_status = status.get_file_status(file_path)?;

if file_status.has_changes() {
    // Get content from different versions
    let head_content = repo.get_head_content(file_path)?;
    let working_content = repo.get_working_content(file_path)?;
    
    // Calculate diff
    let diff = BufferDiff::new(&working_content, &head_content)?;
    
    // Render diff (framework-specific)
    diff_view.render(&diff);
}
```

### Viewing Diffs Between Arbitrary Texts

```rust
// Example API usage (not final)
let text_a = "original text\nwith multiple\nlines";
let text_b = "original text\nwith different\nlines\nand more content";

// Calculate diff
let diff = BufferDiff::new(text_a, text_b)?;

// Render diff (framework-specific)
diff_view.render(&diff);
```