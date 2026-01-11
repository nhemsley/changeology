# Changeology Roadmap

## Vision

A visual git client that makes understanding and managing changes intuitive through an infinite canvas interface.

## Current State

- ✅ Three-panel sidebar (Changes, Staged, History)
- ✅ Directory watching with targeted refresh
- ✅ Commit history viewing with diff display
- ✅ Infinite canvas foundation (pan, zoom)

## Next Up

### Show Diff Canvas on Dirty File Selection
> Research: `research/show-diff-canvas-on-dirty-selected.md`

When a dirty (unstaged) file is selected in the sidebar, display its diff on the infinite canvas. This bridges the sidebar selection to the main visualization area.

### Canvas Layout & Focus
> Research: `research/canvas-layout-and-focus.md`

- Layout algorithms for positioning diff views on the canvas
- Focus system for navigating between canvas items
- Zoom-to-fit for selected items

## Future

### Staging Workflow
- Stage/unstage files from the canvas
- Partial staging (hunk-level)
- Visual drag-and-drop staging

### Commit Creation
- Commit message input
- Commit from within the app
- Amend last commit

### Branch Visualization
- Branch graph on canvas
- Branch switching
- Merge visualization

### Advanced Features
- Stash support
- Interactive rebase visualization
- Blame view
- File history timeline