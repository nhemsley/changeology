use buffer_diff::{DiffHunk, DiffHunkSecondaryStatus, DiffHunkStatus, DiffLineType};

#[test]
fn test_diff_hunk_creation() {
    // Create a simple added hunk
    let hunk = DiffHunk::new(DiffHunkStatus::Added, 0, 0, 0, 3);

    assert_eq!(hunk.status, DiffHunkStatus::Added);
    assert_eq!(hunk.secondary_status, DiffHunkSecondaryStatus::None);
    assert_eq!(hunk.old_range.start, 0);
    assert_eq!(hunk.old_range.count, 0);
    assert_eq!(hunk.new_range.start, 0);
    assert_eq!(hunk.new_range.count, 3);

    // Line types should all be NewOnly for an added hunk
    assert!(hunk
        .line_types
        .iter()
        .all(|&lt| lt == DiffLineType::NewOnly));
    assert_eq!(hunk.line_types.len(), 3);
}

#[test]
fn test_diff_hunk_deleted() {
    // Create a simple deleted hunk
    let hunk = DiffHunk::new(DiffHunkStatus::Deleted, 5, 3, 5, 0);

    assert_eq!(hunk.status, DiffHunkStatus::Deleted);
    assert_eq!(hunk.old_range.start, 5);
    assert_eq!(hunk.old_range.count, 3);
    assert_eq!(hunk.new_range.start, 5);
    assert_eq!(hunk.new_range.count, 0);

    // Line types should all be OldOnly for a deleted hunk
    assert!(hunk
        .line_types
        .iter()
        .all(|&lt| lt == DiffLineType::OldOnly));
    assert_eq!(hunk.line_types.len(), 3);
}

#[test]
fn test_diff_hunk_modified() {
    // Create a simple modified hunk
    let hunk = DiffHunk::new(DiffHunkStatus::Modified, 5, 3, 5, 4);

    assert_eq!(hunk.status, DiffHunkStatus::Modified);
    assert_eq!(hunk.old_range.start, 5);
    assert_eq!(hunk.old_range.count, 3);
    assert_eq!(hunk.new_range.start, 5);
    assert_eq!(hunk.new_range.count, 4);

    // Line types for modified hunks should be Both by default
    assert!(hunk.line_types.iter().all(|&lt| lt == DiffLineType::Both));
    assert_eq!(hunk.line_types.len(), 4); // Takes the max of old and new
}

#[test]
fn test_diff_hunk_unchanged() {
    // Create a simple unchanged hunk
    let hunk = DiffHunk::new(DiffHunkStatus::Unchanged, 10, 5, 10, 5);

    assert_eq!(hunk.status, DiffHunkStatus::Unchanged);
    assert_eq!(hunk.old_range.start, 10);
    assert_eq!(hunk.old_range.count, 5);
    assert_eq!(hunk.new_range.start, 10);
    assert_eq!(hunk.new_range.count, 5);

    // Line types should all be Both for an unchanged hunk
    assert!(hunk.line_types.iter().all(|&lt| lt == DiffLineType::Both));
    assert_eq!(hunk.line_types.len(), 5);
}

#[test]
fn test_set_line_type() {
    // Create a hunk and modify its line types
    let mut hunk = DiffHunk::new(DiffHunkStatus::Modified, 0, 3, 0, 3);

    // Initial state: all Both
    assert!(hunk.line_types.iter().all(|&lt| lt == DiffLineType::Both));

    // Set specific line types
    hunk.set_line_type(0, DiffLineType::Both); // Line 1: unchanged
    hunk.set_line_type(1, DiffLineType::OldOnly); // Line 2: deleted
    hunk.set_line_type(2, DiffLineType::NewOnly); // Line 3: added

    // Check the line types are correct
    assert_eq!(hunk.line_type(0), Some(DiffLineType::Both));
    assert_eq!(hunk.line_type(1), Some(DiffLineType::OldOnly));
    assert_eq!(hunk.line_type(2), Some(DiffLineType::NewOnly));
    assert_eq!(hunk.line_type(3), None); // Out of bounds
}

#[test]
fn test_hunk_statistics() {
    // Create a hunk with mixed line types
    let mut hunk = DiffHunk::new(DiffHunkStatus::Modified, 0, 4, 0, 4);

    // Set mixed line types
    hunk.set_line_type(0, DiffLineType::Both); // Line 1: unchanged
    hunk.set_line_type(1, DiffLineType::OldOnly); // Line 2: deleted
    hunk.set_line_type(2, DiffLineType::NewOnly); // Line 3: added
    hunk.set_line_type(3, DiffLineType::Both); // Line 4: unchanged

    // Check statistics
    assert_eq!(hunk.unchanged_lines(), 2);
    assert_eq!(hunk.deleted_lines(), 1);
    assert_eq!(hunk.added_lines(), 1);
    assert!(hunk.has_changes());
}

#[test]
fn test_secondary_status() {
    // Create a hunk and set its secondary status
    let mut hunk = DiffHunk::new(DiffHunkStatus::Modified, 0, 3, 0, 3);

    // Initial state: None
    assert_eq!(hunk.secondary_status, DiffHunkSecondaryStatus::None);

    // Set to Staged
    hunk.set_secondary_status(DiffHunkSecondaryStatus::Staged);
    assert_eq!(hunk.secondary_status, DiffHunkSecondaryStatus::Staged);

    // Set to Unstaged
    hunk.set_secondary_status(DiffHunkSecondaryStatus::Unstaged);
    assert_eq!(hunk.secondary_status, DiffHunkSecondaryStatus::Unstaged);
}
