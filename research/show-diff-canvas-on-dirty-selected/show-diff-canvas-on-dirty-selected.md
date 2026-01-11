# Show Diff Canvas on Dirty File Selection

## Goal

When a user selects a dirty (unstaged) file in the CHANGES panel, display its diff on the infinite canvas.

## Current State

- Dirty files are listed in the sidebar (`dirty_files: Vec<StatusEntry>`)
- Selection tracking exists (`selected_dirty_file: Option<usize>`)
- Clicking a dirty file updates `selected_dirty_file` and calls `cx.notify()`
- Diff canvas exists and works for commit diffs (`DiffCanvasView`)
- `load_commit_diffs()` shows how to load and display diffs

## What Needs to Happen

### 1. Get Diff for Dirty File

For a dirty file, we need to diff:
- **Old**: The file content at HEAD (last committed version)
- **New**: The file content in the working directory (current state)

```rust
// Pseudocode
let old_content = repo.get_content_at_revision("HEAD", &file_path)?;
let new_content = std::fs::read_to_string(&working_dir.join(&file_path))?;
let diff = DiffConfig::default().diff(&old_content, &new_content)?;
```

### 2. Wire Up Selection to Diff Loading

In the dirty file click handler, after setting `selected_dirty_file`:

```rust
.on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
    this.selected_dirty_file = Some(i);
    this.load_dirty_file_diff(i, cx);  // <-- Add this
    cx.notify();
}))
```

### 3. Implement `load_dirty_file_diff`

Similar to `load_commit_diffs` but for working directory changes:

```rust
fn load_dirty_file_diff(&mut self, file_index: usize, cx: &mut Context<Self>) {
    let Some(entry) = self.dirty_files.get(file_index) else { return };
    let Some(repo) = &self.repository else { return };
    
    let file_path = &entry.path;
    
    // Get HEAD version
    let old_content = repo
        .get_content_at_revision("HEAD", file_path)
        .ok()
        .flatten()
        .unwrap_or_default();
    
    // Get working directory version
    let new_content = self.cwd
        .as_ref()
        .and_then(|cwd| std::fs::read_to_string(cwd.join(file_path)).ok())
        .unwrap_or_default();
    
    // Compute diff
    let config = DiffConfig::default();
    if let Ok(buffer_diff) = config.diff(&old_content, &new_content) {
        let diffs = vec![FileDiff {
            path: file_path.clone(),
            old_content,
            new_content,
            buffer_diff,
        }];
        
        self.diff_canvas.update(cx, |canvas, cx| {
            canvas.set_diffs(diffs, None, cx);  // None = no commit info
        });
    }
}
```

### 4. Handle New Files

New files (untracked) have no HEAD version:
- `old_content` should be empty string
- Entire file shows as additions

### 5. Handle Deleted Files

Deleted files have no working directory version:
- `new_content` should be empty string
- Entire file shows as deletions

### 6. Clear Canvas When Nothing Selected

When `selected_dirty_file` becomes `None`, clear the canvas or show empty state.

## Open Questions

- [ ] Should selecting a staged file also show its diff?
- [ ] What happens when both dirty and staged files are selected?
- [ ] Should we show multiple diffs if multiple files are selected in future?

## Files to Modify

- `crates/changeology/src/app.rs` - Add `load_dirty_file_diff`, wire up click handler
- `crates/git/src/lib.rs` - May need `get_content_at_revision("HEAD", path)` if not exists

## Testing

1. Make a change to a tracked file
2. Select it in CHANGES panel
3. Verify diff appears on canvas
4. Create a new file, select it, verify shows as all additions
5. Delete a tracked file, select it, verify shows as all deletions