# Changeology Project Plan

This document outlines the approach to extract Git diff visualization functionality from Zed into a standalone Rust library.

## Project Structure

The project is organized as a Rust workspace with multiple crates:

- **crates/changeology/** - Main application with GPUI integration
- **crates/diff/** - Core diff calculation and representation
- **crates/git/** - Git repository interaction

## Implementation Phases

### Phase 1: Core Diff Library

Implement the core functionality for calculating and representing diffs between text documents.

- [x] **Step 1:** Create the diff crate structure
  - [x] Core data structures for representing diffs
  - [x] APIs for computing and displaying diffs
  - [x] Example showcasing the functionality
  - [x] Comprehensive tests

- [x] **Step 2:** Improve diff hunk representation
  - [x] Fix line type tracking for accurate add/delete counting
  - [x] Handle empty files, added files, and deleted files correctly
  - [x] Implement context lines for better diff readability
  - [x] Enhance handling of multi-hunk diffs

- [x] **Step 3:** Enhance the diff algorithm
  - [x] Implement chunking and parallelization for large files using rayon
  - [x] Add word-level and character-level diffing options
  - [x] Create configurable diffing API (algorithm, timeout, whitespace handling)
  - [x] Add comprehensive examples demonstrating features
  - [ ] Support for binary files

- [ ] **Step 4:** Improve text representation
  - [ ] Better handling of different line ending types
  - [ ] Efficient storage and manipulation of text
  - [ ] Unicode support improvements

- [ ] **Step 5:** Add helper methods
  - [ ] Methods for navigating through hunks
  - [ ] Statistical analysis of diffs
  - [ ] Search functionality within diffs

### Phase 2: Git Integration

Implement Git integration to access repository information and file versions.

- [x] **Step 1:** Create the git crate structure
  - [x] Repository wrapper around git2
  - [x] Status representation and filtering
  - [x] Methods for accessing different versions of files

- [x] **Step 2:** Implement repository operations
  - [x] Get working directory and index status
  - [x] Access file content from different versions (HEAD, index, working)
  - [x] Generate diffs between versions

- [ ] **Step 3:** Handle advanced Git scenarios
  - [ ] Support for merge conflicts
  - [ ] Handling of submodules
  - [ ] Stash operations
  - [ ] Branch and remote operations

### Phase 3: UI Integration

Implement the UI components using GPUI.

- [ ] **Step 1:** Create basic application shell
  - [ ] Window and layout setup
  - [ ] Menu and command structure
  - [ ] Settings and preferences

- [ ] **Step 2:** Build diff visualization components
  - [ ] Side-by-side diff view
  - [ ] Inline diff view
  - [ ] Syntax highlighting integration
  - [ ] Interactive components (expand/collapse hunks, etc.)

- [ ] **Step 3:** Create file browser and navigation
  - [ ] File tree component
  - [ ] Status indicators
  - [ ] Search and filter

- [ ] **Step 4:** Implement Git operations UI
  - [ ] Stage/unstage functionality
  - [ ] Commit interface
  - [ ] Branch visualization
  - [ ] History view

### Phase 4: Performance Optimization

Optimize the performance of both the core library and the UI.

- [ ] **Step 1:** Profile and identify bottlenecks
  - [ ] Core diff algorithm
  - [ ] File loading and parsing
  - [ ] UI rendering

- [ ] **Step 2:** Implement optimizations
  - [ ] Parallelization of diff calculations
  - [ ] Caching strategies
  - [ ] Virtualized rendering for large diffs
  - [ ] Incremental diffing

- [ ] **Step 3:** Create benchmarks
  - [ ] Benchmark suite for core operations
  - [ ] UI rendering benchmarks
  - [ ] Regression testing

## Current Status

- **Phase 1 (Core Diff Library)**: 3/5 steps completed
- **Phase 2 (Git Integration)**: 2/3 steps completed 
- **Phase 3 (UI Integration)**: Not started (gpui dependency removed)
- **Phase 4 (Performance Optimization)**: Partially implemented (parallelization in diff)

## Next Steps

1. Work on Phase 1, Step 4: Improve text representation
2. Continue work on Phase 2, Step 3: Handle advanced Git scenarios
3. Decide on UI approach (Phase 3) - gpui removed as dependency

## Recent Accomplishments

1. Enhanced diff algorithm implementation:
   - Added chunking and parallelization for large files using rayon
   - Implemented word-level and character-level diffing options
   - Created configurable API with algorithm selection, timeout, whitespace handling
   - Added comprehensive examples demonstrating different features
   
2. Added utility for generating large test files:
   - Created generate_large_files.rs tool for test file generation with controlled modifications
   
3. Removed gpui dependency to simplify architecture