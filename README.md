# Changeology

A modular Git diff visualization library extracted from [Zed](https://zed.dev/). This project aims to make Zed's powerful diff visualization functionality available as standalone components that can be used in other Rust applications.

## Overview

Changeology extracts Zed's diff visualization tools into separate, reusable components. The project is organized as a Rust workspace with multiple crates that can be used independently or together.

### Crates

- **[git](crates/git/README.md)**: A wrapper around `git2` that provides a simpler interface for working with Git repositories, including status information and file content retrieval.

- **[diff](crates/diff/README.md)**: A crate for calculating and representing differences between text documents, with support for hunks, line-level granularity, and context lines.

- **changeology** (Main application): Integrates the functionality of the other crates with a GPUI-based visualization interface.

## Features

- View unstaged changes in a Git repository
- View diffs between arbitrary text versions
- Hunk-based diff visualization
- Line-by-line change tracking
- Context-aware diff presentation
- Visual Git integration

## Dependencies

- [GPUI](https://www.gpui.rs/) - Zed's GPU-accelerated UI framework
- [git2](https://github.com/rust-lang/git2-rs) - Rust bindings to libgit2
- [ropey](https://github.com/cessen/ropey) - A rope data structure for text editing
- [similar](https://github.com/mitsuhiko/similar) - A diffing library for Rust

## Usage

- Ensure Rust is installed - [Rustup](https://rustup.rs/)
- Run the main application with `cargo run`
- Or use the individual crates in your own project (see each crate's README for details)

## Development

### Getting Started

```bash
# Clone the repository
git clone https://github.com/yourusername/changeology.git
cd changeology

# Run the application
cargo run

# Run the tests
cargo test
```

### Project Structure

- `crates/git/`: The Git wrapper crate
- `crates/diff/`: The diff calculation and representation crate
- `crates/changeology/`: The main application with GPUI integration

## Related Projects

- [Zed](https://github.com/zed-industries/zed) - The original source of the diff visualization functionality
- [git2-rs](https://github.com/rust-lang/git2-rs) - Rust bindings to libgit2
- [GPUI](https://www.gpui.rs/) - GPU-accelerated UI framework