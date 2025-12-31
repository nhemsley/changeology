# Buffer-Diff Crate

A crate for calculating and representing differences between text documents.

## Features

- Calculate diffs between text documents
- Represent diffs as hunks with line-level granularity
- Generate unified diffs for display
- Track added, deleted, and unchanged lines
- Supports multi-hunk diffs for large files
- Includes context lines for better readability

## Usage

SOme change
Add the crate to your `Cargo.toml`:

```toml
[dependencies]
buffer-diff = { path = "../diff" }
anyhow = "1.0"
```

Some chane 3
### Examples

#### Creating and Displaying a Text Diff

```rust
use anyhow::Result;
use buffer_diff::{DiffLineType, TextDiff};

fn main() -> Result<()> {
    // Two sample texts to compare
    let text1 = "This is the first line.\nHere is the second line.\nAnd the third line.";
    let text2 = "This is the first line.\nThis is a completely different second line.\nAnd the third line.\nPlus a new fourth line.";

    // Another
    // Generate a unified diff
    println!("Unified diff:");
    println!("{}", TextDiff::unified_diff(text1, text2, 1));

    // Generate a diff object
    let diff = TextDiff::diff(text1, text2)?;
    let snapshot = diff.snapshot();

    // Print diff statistics
    println!("\nDiff statistics:");
    println!("  Total hunks: {}", snapshot.hunk_count());
    println!("  Added lines: {}", snapshot.added_lines());
    println!("  Deleted lines: {}", snapshot.deleted_lines());
    println!("  Unchanged lines: {}", snapshot.unchanged_lines());

    // Print hunks with line types
    println!("\nHunks with line types:");
    for (i, hunk) in snapshot.hunks().iter().enumerate() {
        println!("Hunk {}:", i + 1);
        println!("  Status: {}", hunk.status);
        println!("  Old range: {}:{}", hunk.old_range.start, hunk.old_range.count);
        println!("  New range: {}:{}", hunk.new_range.start, hunk.new_range.count);

        // Print line types for this hunk
        println!("  Line types:");
        for (j, &line_type) in hunk.line_types.iter().enumerate() {
            match line_type {
                DiffLineType::OldOnly => println!("    Line {}: Deleted", j),
                DiffLineType::NewOnly => println!("    Line {}: Added", j),
                DiffLineType::Both => println!("    Line {}: Unchanged", j),
            }
        }
    }
    
    Ok(())
}
```

#### Working with BufferDiff Directly

For more control over the diff process, you can use `BufferDiff` directly:

```rust
use buffer_diff::{BufferDiff, DiffHunkStatus};
use anyhow::Result;

fn main() -> Result<()> {
    let old_text = "Line 1\nLine 2\nLine 3\n";
    let new_text = "Line 1\nLine X\nLine 3\n";
    
    let diff = BufferDiff::new(old_text, new_text)?;
    let snapshot = diff.snapshot();
    
    // Check if any changes
    if snapshot.has_changes() {
        println!("Files are different!");
        
        // Print statistics
        println!("Added lines: {}", snapshot.added_lines());
        println!("Deleted lines: {}", snapshot.deleted_lines());
        println!("Unchanged lines: {}", snapshot.unchanged_lines());
        
        // Process each hunk
        for (i, hunk) in snapshot.hunks().iter().enumerate() {
            println!("Hunk {}:", i + 1);
            
            match hunk.status {
                DiffHunkStatus::Added => println!("  New content added"),
                DiffHunkStatus::Deleted => println!("  Content deleted"),
                DiffHunkStatus::Modified => println!("  Content modified"),
                DiffHunkStatus::Unchanged => println!("  Content unchanged"),
            }
            
            // Print line ranges
            println!("  Old range: {}:{}", hunk.old_range.start, hunk.old_range.count);
            println!("  New range: {}:{}", hunk.new_range.start, hunk.new_range.count);
        }
    } else {
        println!("Files are identical!");
    }
    
    Ok(())
}
```

## API Reference

### TextDiff

A high-level interface for working with text diffs.

Key static methods:
- `diff(old_text: &str, new_text: &str) -> Result<BufferDiff>`: Create a diff between two texts
- `unified_diff(old_text: &str, new_text: &str, context_lines: usize) -> String`: Generate a unified diff string

### BufferDiff

The core diff implementation that handles comparing text documents.

```rust
pub struct BufferDiff {
    old_text: Rope,
    new_text: Rope,
    hunks: Vec<DiffHunk>,
}
```

Key methods:
- `new(old_text: &str, new_text: &str) -> Result<BufferDiff>`: Create a new buffer diff
- `snapshot(&self) -> BufferDiffSnapshot`: Get an immutable snapshot of the diff
- `hunks(&self) -> &[DiffHunk]`: Get all hunks in the diff
- `hunk_count(&self) -> usize`: Get the number of hunks
- `hunk(&self, index: usize) -> Option<&DiffHunk>`: Get a specific hunk by index

### BufferDiffSnapshot

An immutable snapshot of a diff for analysis.

```rust
pub struct BufferDiffSnapshot {
    pub hunks: Vec<DiffHunk>,
    pub old_line_count: usize,
    pub new_line_count: usize,
}
```

Key methods:
- `empty() -> BufferDiffSnapshot`: Create an empty diff snapshot
- `has_changes(&self) -> bool`: Check if the diff has any changes
- `added_lines(&self) -> usize`: Get the number of added lines
- `deleted_lines(&self) -> usize`: Get the number of deleted lines
- `unchanged_lines(&self) -> usize`: Get the number of unchanged lines

### DiffHunk

Represents a hunk of changes in a diff.

```rust
pub struct DiffHunk {
    pub status: DiffHunkStatus,
    pub secondary_status: DiffHunkSecondaryStatus,
    pub old_range: DiffHunkRange,
    pub new_range: DiffHunkRange,
    pub line_types: Vec<DiffLineType>,
}
```

Key methods:
- `new(status: DiffHunkStatus, old_start: usize, old_count: usize, new_start: usize, new_count: usize) -> DiffHunk`: Create a new diff hunk
- `has_changes(&self) -> bool`: Check if the hunk has any changes
- `added_lines(&self) -> usize`: Get the number of added lines in the hunk
- `deleted_lines(&self) -> usize`: Get the number of deleted lines in the hunk
- `unchanged_lines(&self) -> usize`: Get the number of unchanged lines in the hunk

### DiffHunkStatus

Enum representing the status of a diff hunk.

```rust
pub enum DiffHunkStatus {
    Added,
    Deleted,
    Modified,
    Unchanged,
}
```

### DiffLineType

Enum representing the type of a line in a diff hunk.

```rust
pub enum DiffLineType {
    OldOnly,   // Line only exists in old version (deleted)
    NewOnly,   // Line only exists in new version (added)
    Both,      // Line exists in both versions (unchanged)
}
```

## Running the Example

```
cargo run --example simple_diff
```
