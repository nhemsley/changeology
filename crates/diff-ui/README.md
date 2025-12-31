# Diff UI

A minimal GPUI application for displaying text diffs with colored backgrounds.

## Overview

This crate provides a simple visual diff viewer built with [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui), Zed's GPU-accelerated UI framework. It displays the differences between two text strings with colored backgrounds:

- **Green** for added lines
- **Red** for deleted lines  
- **Normal** for unchanged lines

This is a **focused subset** of Zed's diff visualization, extracting only the core text rendering functionality without the complexity of the full editor.

## Features

- ✅ Text diff display with colored backgrounds
- ✅ Scrolling support for long diffs
- ✅ Line prefixes (`+`, `-`, ` `)
- ✅ Dark and light theme support
- ✅ Monospace font rendering
- ✅ Integration with `buffer-diff` crate

## Not Included (Yet)

This is a minimal implementation focusing on the core diff rendering. Future phases may add:

- Line numbers and gutter
- File headers
- Hunk collapse/expand
- Syntax highlighting
- Git repository integration
- Keyboard navigation

## Usage

### Running the Demo

```bash
# From the project root
cargo run -p diff-ui
```

This opens a window displaying a sample diff.

### As a Library

```rust
use diff_ui::DiffTextView;

// Create a diff view from two text strings
let old_text = "line 1\nline 2\nline 3\n";
let new_text = "line 1\nline 2 modified\nline 3\nline 4\n";

let view = DiffTextView::new(old_text, new_text);
// `view` is a GPUI component that implements Render
```

### Custom Themes

```rust
use diff_ui::{DiffTextView, DiffTheme};

let view = DiffTextView::new(old_text, new_text)
    .with_theme(DiffTheme::light()); // Use light theme
```

## Architecture

```
DiffTextView
    │
    ├─> Uses buffer-diff crate to calculate hunks
    ├─> Converts hunks to display lines  
    └─> Renders with GPUI
           │
           └─> Each line gets colored background
```

### Key Components

- **DiffTextView** - Main component that renders the diff
- **DiffDisplayLine** - Represents a single line with style
- **DiffTheme** - Color definitions for dark/light themes

## Dependencies

- `gpui` - GPU-accelerated UI framework from Zed
- `buffer-diff` - Our diff calculation crate (workspace)
- `git` - Our Git wrapper crate (workspace, not used yet)
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

- **Total Lines**: ~450
- **Components**: 3 files
  - `main.rs` - Application entry (45 lines)
  - `diff_text_view.rs` - Core rendering (295 lines)
  - `theme.rs` - Colors (112 lines)

Compare to Zed's Editor component: ~20,000 lines.  
This is **98.5% simpler** by focusing only on diff display.

## Roadmap

### Phase 1: Basic Diff Display ✅ (Current)
- Text rendering with colored backgrounds
- Scrolling support
- Theme system

### Phase 2: Gutter (Planned)
- Line numbers
- Colored status bars
- Old/new line mapping

### Phase 3: Interactivity (Planned)
- Hunk navigation
- Expand/collapse hunks
- Stage/Restore buttons

### Phase 4: Git Integration (Planned)
- Load from repository
- HEAD vs working directory
- Staged vs unstaged

### Phase 5: File List (Planned)
- Multiple file view
- File tree navigation
- Status indicators

## Contributing

This is part of the Changeology project - a modular Git diff visualization library extracted from Zed.

See the main [Changeology README](../../README.md) for more information.

## License

Same as parent project.

## See Also

- [buffer-diff](../diff/) - Diff calculation crate
- [git](../git/) - Git repository wrapper
- [Zed Editor](https://github.com/zed-industries/zed) - Original source of inspiration
- [GPUI](https://www.gpui.rs/) - The UI framework we use
