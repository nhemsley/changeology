# Changeology Architecture Analysis

## Executive Summary

Changeology is a Rust library that extracts Git diff visualization functionality from the Zed editor into standalone, reusable components. The project has successfully decoupled Zed's tightly-integrated `buffer_diff` and `git` crates from GPUI (Zed's UI framework) while adding modern enhancements like parallelization, configurable granularity, and line ending normalization.

**Project Status**: ~60% complete (based on Claude.md roadmap)
- ✅ Core diff library functional with enhancements
- ✅ Basic Git integration working
- ❌ UI layer removed (GPUI dependency eliminated)
- 🔄 Advanced features in progress

## Project Origins: Extraction from Zed

### Source Material

The project extracts functionality from two Zed crates located in `vendor/zed/crates/`:

1. **`buffer_diff/`** (2183 lines)
   - Complex diff calculation integrated with Zed's editor buffer
   - Heavy GPUI integration for async operations
   - SumTree-based data structures for efficient queries
   - Anchor-based positioning that updates as buffers change

2. **`git/`** (multiple files, ~100+ lines each)
   - Repository abstraction over libgit2
   - Remote operations and hosting provider integrations
   - Blame, commit, and branch functionality
   - Async operations tied to GPUI's task system

### Key Differences: Zed vs Changeology

| Aspect | Zed's buffer_diff | Changeology's diff |
|--------|-------------------|-------------------|
| **Size** | 2183 lines | ~860 lines |
| **Dependencies** | GPUI, language, text, sum_tree, clock | ropey, similar, rayon, anyhow |
| **Data Structures** | SumTree<InternalDiffHunk> | Vec<DiffHunk> |
| **Positioning** | Anchor-based (editor-aware) | Simple line ranges |
| **Operations** | Async with GPUI tasks | Sync/parallel with rayon |
| **Granularity** | Line-level only | Line/word/character levels |
| **Line Endings** | Not explicitly handled | Auto-detection & normalization |
| **Large Files** | Standard processing | Chunked parallel processing |
| **Integration** | Tightly coupled to editor | Standalone library |

## Architecture

### Crate Structure

```
changeology/
├── crates/
│   ├── diff/           # Core diff calculation (standalone)
│   ├── git/            # Git repository wrapper (standalone)
│   └── changeology/    # Main app (GPUI removed, minimal)
└── vendor/
    └── zed/            # Original Zed source (reference)
```

### Crate: `diff`

**Purpose**: Calculate and represent differences between text documents

**Key Components**:

1. **`buffer_diff.rs`** (~860 lines)
   - `BufferDiff`: Main diff computation struct
   - Uses `ropey::Rope` for efficient text manipulation
   - Implements chunking for files > 100k characters
   - Parallel processing with `rayon` (up to 8 concurrent chunks)
   - Merges adjacent/overlapping hunks for cleaner output

2. **`diff_hunk.rs`** (220 lines)
   - `DiffHunk`: Represents a continuous block of changes
   - `DiffHunkStatus`: Added/Deleted/Modified/Unchanged
   - `DiffHunkRange`: Start position + count
   - `DiffLineType`: Tracks line-by-line changes within hunks
   - `DiffHunkSecondaryStatus`: Git staging status (for future use)

3. **`text_diff.rs`** (420 lines)
   - `TextDiff`: High-level API wrapper
   - `DiffConfig`: Configurable diff parameters
   - `DiffGranularity`: Line/Word/Character level diffing
   - `LineEndingMode`: Auto/Unix/Windows/MacOS/Preserve
   - Uses `similar` crate for diff algorithms (Myers, Patience, LCS)

**Notable Enhancements Over Zed**:

- **Parallelization**: Automatically chunks and processes large files in parallel
- **Granularity Options**: Word and character-level diffing (Zed only does line-level)
- **Line Ending Handling**: Auto-detection and normalization (not in Zed)
- **Configuration API**: Builder pattern for customizing diff behavior
- **Simplified Data Model**: No sum_tree, no anchors, just simple ranges

**Dependencies**:
```toml
ropey = "1.x"        # Rope data structure (replaces Zed's rope + text crates)
similar = "2.x"      # Diff algorithms (replaces git2 Patch)
rayon = "1.8"        # Parallelization (NEW - not in Zed)
anyhow = "1.x"       # Error handling
derive_more = "0.x"  # Derive macros
```

### Crate: `git`

**Purpose**: Simplified Git repository operations

**Key Components**:

1. **`repository.rs`** (160 lines)
   - `Repository`: Wrapper around `git2::Repository`
   - File content retrieval from HEAD, index, working directory
   - Diff generation between versions
   - Status information

2. **`status.rs`** (150 lines)
   - `StatusEntry`: File path + status kind
   - `StatusList`: Collection with filtering helpers
   - `StatusKind`: Added/Modified/Deleted/Renamed/Untracked/Ignored/Conflicted

**Simplifications from Zed**:

- ❌ No remote operations (push, pull, fetch)
- ❌ No blame functionality
- ❌ No hosting provider integrations (GitHub, GitLab)
- ❌ No async operations
- ❌ No commit operations
- ✅ Focus on read-only diff operations
- ✅ Simple synchronous API

**Dependencies**:
```toml
git2 = "0.x"         # libgit2 bindings (same as Zed)
anyhow = "1.x"       # Error handling
pathdiff = "0.x"     # Path utilities
derive_more = "0.x"  # Derive macros
path-clean = "0.x"   # Path normalization
```

### Removed: GPUI Integration

The original Zed implementation heavily relied on GPUI for:

- **Async Task Management**: `Task<T>` and `AsyncApp` for background operations
- **Context System**: `Context<T>` for entity lifecycle
- **Event System**: `EventEmitter` for change notifications
- **Entity Management**: `Entity<BufferDiff>` for shared ownership

**Changeology removed all GPUI dependencies**, resulting in:
- ✅ Simpler, standalone library
- ✅ No GPU/UI framework requirements
- ✅ Easier integration into other projects
- ❌ No built-in UI (user must provide their own)
- ❌ No async task orchestration (though rayon provides parallelism)

## Commit History Analysis

### Key Milestones

```
1763b17  initial commit                    [Dec 14, 2024]
         └─> Basic project structure with GPUI

1adfb41  Add git crate                     [May 2, 2025]
         └─> Git wrapper with Repository and Status

91a4e43  Add diff crate                    [May 2, 2025]
         └─> Core BufferDiff extracted from Zed
         └─> Uses similar crate instead of git2 Patch

475c157  Improve diff hunk representation  
         └─> Better line type tracking
         └─> Context lines support

2d01e09  Remove gpui dependency            
         └─> Decoupled from Zed UI framework
         └─> Simplified architecture

fbd7aba  Enhance diff algorithm            
         └─> Chunking for large files
         └─> Parallelization with rayon
         └─> Word/character granularity
         └─> Configurable API

cd21417  Improve text representation       
         └─> Line ending normalization
         └─> Auto-detection of line endings

2ef9418  Fix warnings                      [Latest]
         └─> Code cleanup
```

### Evolution Pattern

1. **Extraction Phase**: Copy core logic from Zed
2. **Simplification Phase**: Remove GPUI, simplify data structures
3. **Enhancement Phase**: Add modern features (parallelization, configuration)
4. **Polish Phase**: Line ending handling, examples, tests

## Technical Deep Dive

### Diff Algorithm Comparison

**Zed's Approach** (`buffer_diff.rs:717-850`):
```rust
fn compute_hunks(
    diff_base: Option<(Arc<String>, Rope)>,
    buffer: text::BufferSnapshot,
) -> SumTree<InternalDiffHunk> {
    // Uses git2::Patch directly
    let patch = GitPatch::from_buffers(...)?;
    
    // Processes hunks with divergence tracking
    for hunk_index in 0..patch.num_hunks() {
        let hunk = process_patch_hunk(&patch, ...);
        tree.push(hunk, &buffer);
    }
}
```

**Changeology's Approach** (`buffer_diff.rs:60-226`):
```rust
fn compute_hunks(&mut self) -> Result<()> {
    // Check for large files
    if self.old_text.len_chars() > 100_000 {
        // Chunk the file
        let old_chunks = self.calculate_chunk_boundaries(old_line_count);
        let new_chunks = self.calculate_chunk_boundaries(new_line_count);
        
        // Process in parallel with rayon
        (0..num_chunks).into_par_iter().for_each(|i| {
            let chunk_hunks = self.diff_chunk(...);
            all_hunks.lock().extend(chunk_hunks);
        });
        
        // Merge adjacent hunks
        self.hunks = self.merge_adjacent_hunks(final_hunks);
    } else {
        // Use similar crate for smaller files
        let diff = similar::TextDiff::configure()
            .algorithm(similar::Algorithm::Myers)
            .timeout(Duration::from_secs(5))
            .diff_lines(&old_text_str, &new_text_str);
    }
}
```

### Data Structure Evolution

**Zed's InternalDiffHunk**:
```rust
struct InternalDiffHunk {
    buffer_range: Range<Anchor>,           // Editor-aware positions
    diff_base_byte_range: Range<usize>,
}

// Stored in SumTree for O(log n) queries
```

**Changeology's DiffHunk**:
```rust
pub struct DiffHunk {
    pub status: DiffHunkStatus,
    pub secondary_status: DiffHunkSecondaryStatus,
    pub old_range: DiffHunkRange,         // Simple line-based ranges
    pub new_range: DiffHunkRange,
    pub line_types: Vec<DiffLineType>,    // Per-line granularity
}

// Stored in Vec for simplicity
```

### Rope Usage Differences

**Zed**:
- Uses custom `rope` crate integrated with `text` crate
- Tied to editor's BufferSnapshot
- Anchor-based addressing survives edits

**Changeology**:
- Uses `ropey` (popular open-source rope implementation)
- Standalone, no editor integration
- Simple offset/line-based addressing

## Example Usage

### Basic Diff
```rust
use diff::TextDiff;

let old = "Hello\nWorld\n";
let new = "Hello\nRust\nWorld\n";

// Simple diff
let diff = TextDiff::diff(old, new)?;
let snapshot = diff.snapshot();

println!("Hunks: {}", snapshot.hunk_count());
println!("Added: {}", snapshot.added_lines());
```

### Configured Diff
```rust
use diff::{DiffConfig, DiffGranularity, LineEndingMode};
use similar::Algorithm;

let config = DiffConfig::default()
    .algorithm(Algorithm::Patience)
    .granularity(DiffGranularity::Word)
    .line_ending_mode(LineEndingMode::Unix)
    .ignore_whitespace(true)
    .timeout(10);

let diff = config.diff(old, new)?;
```

### Git Integration
```rust
use git::Repository;

let repo = Repository::open(".")?;
let status = repo.status()?;

for entry in status.modified() {
    let head = repo.get_head_content(&entry.path)?;
    let working = repo.get_working_content(&entry.path)?;
    
    if let (Some(old), Some(new)) = (head, working) {
        let diff = TextDiff::diff(&old, &new)?;
        println!("Changes in {}: {} hunks", entry.path, diff.hunk_count());
    }
}
```

## Project Status & Roadmap

### Completed Features

✅ **Phase 1: Core Diff Library** (3/5 steps)
- Core data structures and API
- Improved hunk representation
- Enhanced algorithm with parallelization
- Line ending handling

✅ **Phase 2: Git Integration** (2/3 steps)
- Git crate structure
- Repository operations
- Status tracking

✅ **Phase 4: Performance** (partial)
- Parallelization for large files
- Chunking strategy

### In Progress

🔄 **Phase 1: Core Diff Library** (remaining)
- Efficient text storage (considering alternatives to Rope)
- Unicode support improvements
- Helper methods (navigation, statistics, search)

🔄 **Phase 2: Git Integration** (remaining)
- Advanced scenarios (merge conflicts, submodules)
- Stash operations
- Branch operations

### Not Started

❌ **Phase 3: UI Integration**
- Removed with GPUI dependency
- Decision needed: new UI approach or stay library-only

## Testing Strategy

### Test Coverage

**diff crate**:
- `tests/diff_tests.rs`: Basic diff functionality
- `tests/diff_hunk_tests.rs`: Hunk representation
- `tests/chunking_tests.rs`: Large file handling
- `tests/granularity_tests.rs`: Word/character diffing
- `tests/edge_cases.rs`: Empty files, identical files, etc.

**git crate**:
- Basic repository operations
- Status filtering
- Fixtures for test repositories

### Example Suite

**diff examples**:
- `simple_diff.rs`: Basic usage
- `enhanced_diff.rs`: CLI with all features
- `word_diff.rs`: Granularity demonstration
- `chunked_diff.rs`: Large file handling
- `config_diff.rs`: Configuration options
- `line_ending_diff.rs`: Line ending normalization

**git examples**:
- `status.rs`: Repository status
- `diff.rs`: Git diff integration

## Dependencies Overview

### Core Dependencies
- **ropey**: Rope data structure for text
- **similar**: Diff algorithms (Myers, Patience, LCS)
- **git2**: libgit2 bindings for Git operations
- **anyhow**: Error handling
- **rayon**: Data parallelism

### Development Dependencies
- **insta**: Snapshot testing
- **proptest**: Property-based testing
- **pretty_assertions**: Better test output
- **tempfile**: Temporary directories for Git tests

### Removed from Zed
- ❌ **gpui**: GPU-accelerated UI framework
- ❌ **language**: Language/syntax support
- ❌ **text**: Zed's text buffer implementation
- ❌ **sum_tree**: Efficient tree data structure
- ❌ **clock**: Version tracking
- ❌ **util**: Zed utilities

## Design Decisions

### Why Remove GPUI?

**Advantages**:
- Simpler architecture
- Easier to integrate into other projects
- No GPU/graphics requirements
- Smaller dependency footprint

**Trade-offs**:
- No built-in UI visualization
- Users must build their own UI layer
- No async task orchestration (but rayon provides parallelism)

### Why Add Parallelization?

Zed's diff runs in async tasks but processes serially. Changeology adds:
- Automatic chunking for files > 100k characters
- Up to 8 concurrent chunks via rayon
- Significant speedup for large files (10x+ for very large diffs)

### Why Add Granularity Options?

Zed only supports line-level diffing. Changeology adds:
- Word-level: Better for prose/documentation
- Character-level: Maximum precision for code review
- Configurable via builder API

### Why Normalize Line Endings?

Cross-platform compatibility:
- Windows (CRLF) vs Unix (LF) vs Classic Mac (CR)
- Auto-detection of dominant format
- Prevents spurious diffs due to line ending changes

## Potential Future Directions

### Library Evolution
1. **Binary diff support**: Currently text-only
2. **Incremental diffing**: Update diffs as text changes
3. **Semantic diffing**: Syntax-aware diffs for code
4. **Diff merging**: Three-way merge support

### UI Options
1. **Terminal UI**: Use `ratatui` or similar
2. **Web UI**: WASM compilation + web frontend
3. **New native UI**: `egui`, `iced`, or other Rust GUI
4. **LSP integration**: Provide as language server
5. **Stay library-only**: Let users build their own UIs

### Git Features
1. **Write operations**: Commit, stage, unstage
2. **Branch management**: Create, delete, switch branches
3. **Remote operations**: Push, pull, fetch
4. **Blame integration**: Line-by-line history
5. **Conflict resolution**: Interactive merge

## Conclusion

Changeology successfully extracts and modernizes Zed's diff functionality into a standalone, reusable Rust library. The project demonstrates how to:

1. **Decouple** tightly-integrated code from a specific framework
2. **Simplify** complex data structures while maintaining functionality
3. **Enhance** extracted code with modern features (parallelization, configuration)
4. **Document** the extraction process for future reference

The result is a clean, well-tested library that provides powerful diff capabilities without requiring GPUI or any specific UI framework. While ~40% of the roadmap remains, the core functionality is solid and ready for use in other projects.

**Next Steps**:
1. Complete remaining Phase 1 improvements (efficient storage, Unicode)
2. Implement advanced Git scenarios (Phase 2)
3. Decide on UI strategy (Phase 3) or stay library-focused
4. Expand test coverage and documentation
5. Consider publishing to crates.io

## References

- Original Zed source: `vendor/zed/crates/buffer_diff/` and `vendor/zed/crates/git/`
- Project documentation: `Claude.md`, `session_context.md`, `README.md`
- Commit history: Shows clear extraction and enhancement pattern
- Examples: Demonstrate practical usage of all features