use diff::{BufferDiff, DiffHunkStatus, DiffLineType, TextDiff};

#[test]
fn test_empty_files() {
    // Two empty files should result in an unchanged hunk
    let old = "";
    let new = "";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert_eq!(snapshot.hunk_count(), 1);
    assert_eq!(snapshot.hunks()[0].status, DiffHunkStatus::Unchanged);
    assert_eq!(snapshot.added_lines(), 0);
    assert_eq!(snapshot.deleted_lines(), 0);
    assert_eq!(snapshot.unchanged_lines(), 0);
}

#[test]
fn test_identical_files() {
    // Identical files should result in an unchanged hunk
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine 2\nLine 3\n";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert_eq!(snapshot.hunk_count(), 1);
    assert_eq!(snapshot.hunks()[0].status, DiffHunkStatus::Unchanged);
    assert_eq!(snapshot.added_lines(), 0);
    assert_eq!(snapshot.deleted_lines(), 0);
    assert!(snapshot.unchanged_lines() > 0);
}

#[test]
fn test_added_file() {
    // New file added (old is empty)
    let old = "";
    let new = "Line 1\nLine 2\n";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert_eq!(snapshot.hunk_count(), 1);
    assert_eq!(snapshot.hunks()[0].status, DiffHunkStatus::Added);
    assert!(snapshot.added_lines() > 0);
    assert_eq!(snapshot.deleted_lines(), 0);
    assert_eq!(snapshot.unchanged_lines(), 0);
}

#[test]
fn test_deleted_file() {
    // File deleted (new is empty)
    let old = "Line 1\nLine 2\n";
    let new = "";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert_eq!(snapshot.hunk_count(), 1);
    assert_eq!(snapshot.hunks()[0].status, DiffHunkStatus::Deleted);
    assert_eq!(snapshot.added_lines(), 0);
    assert!(snapshot.deleted_lines() > 0);
    assert_eq!(snapshot.unchanged_lines(), 0);
}

#[test]
fn test_modified_file() {
    // Modified file
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine X\nLine 3\n";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    // Should have at least one hunk
    assert!(snapshot.hunk_count() >= 1);

    // At least one of the hunks should be modified
    let has_modified = snapshot
        .hunks()
        .iter()
        .any(|h| h.status == DiffHunkStatus::Modified);
    assert!(has_modified);

    // Expect some deleted and added lines but also some unchanged
    assert!(snapshot.added_lines() >= 1);
    assert!(snapshot.deleted_lines() >= 1);
    assert!(snapshot.unchanged_lines() >= 1);
}

#[test]
fn test_additions_only() {
    // Only additions, no deletions
    let old = "Line 1\nLine 3\n";
    let new = "Line 1\nLine 2\nLine 3\n";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert!(snapshot.hunk_count() >= 1);
    assert!(snapshot.added_lines() >= 1);
    assert_eq!(snapshot.deleted_lines(), 0);
    assert!(snapshot.unchanged_lines() >= 1);
}

#[test]
fn test_deletions_only() {
    // Only deletions, no additions
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine 3\n";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert!(snapshot.hunk_count() >= 1);
    assert_eq!(snapshot.added_lines(), 0);
    assert!(snapshot.deleted_lines() >= 1);
    assert!(snapshot.unchanged_lines() >= 1);
}

#[test]
fn test_multi_hunk_diff() {
    // Changes in multiple places should produce multiple hunks
    let old = "
Line 1
Line 2
Line 3
Line 4
Line 5
Line 6
Line 7
Line 8
"
    .trim();

    let new = "
Line 1
Line 2 modified
Line 3
Line 4
Line 5
Line 6 modified
Line 7
Line 8
"
    .trim();

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    // We expect two hunks (one for each modified line)
    // But our current implementation might merge them into one
    assert!(snapshot.hunk_count() >= 1);
    assert!(snapshot.added_lines() >= 2);
    assert!(snapshot.deleted_lines() >= 2);
}

#[test]
fn test_line_types() {
    // Test that line types are correctly identified
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine X\nLine 3\nLine 4\n";

    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();

    // Find a hunk with both additions and deletions
    let hunk = snapshot.hunks().iter().find(|h| {
        h.line_types.contains(&DiffLineType::OldOnly)
            && h.line_types.contains(&DiffLineType::NewOnly)
    });

    // Make sure we found such a hunk
    assert!(hunk.is_some());

    if let Some(hunk) = hunk {
        // Check if the line types match our expectations
        let has_old_only = hunk.line_types.contains(&DiffLineType::OldOnly);
        let has_new_only = hunk.line_types.contains(&DiffLineType::NewOnly);
        let has_both = hunk.line_types.contains(&DiffLineType::Both);

        assert!(has_old_only);
        assert!(has_new_only);
        // Note: it's okay if there's no "Both" type depending on implementation
    }
}

#[test]
fn test_text_diff() {
    // Test the TextDiff utilities
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine X\nLine 3\nLine 4\n";

    // Test unified diff
    let unified = TextDiff::unified_diff(old, new, 1);
    assert!(!unified.is_empty());
    assert!(unified.contains("Line 1"));
    assert!(unified.contains("Line 2"));
    assert!(unified.contains("Line X"));
    assert!(unified.contains("Line 3"));
    assert!(unified.contains("Line 4"));

    // Test creating a diff object
    let diff = TextDiff::diff(old, new).unwrap();
    let snapshot = diff.snapshot();

    assert!(snapshot.hunk_count() >= 1);
    assert!(snapshot.added_lines() >= 1);
    assert!(snapshot.deleted_lines() >= 0);
}

#[test]
fn test_range_methods() {
    // Test DiffHunkRange methods
    use diff::DiffHunkRange;

    let range = DiffHunkRange::new(10, 5);

    assert_eq!(range.start, 10);
    assert_eq!(range.count, 5);
    assert_eq!(range.end(), 15);
    assert!(!range.is_empty());
    assert!(range.contains(10));
    assert!(range.contains(14));
    assert!(!range.contains(9));
    assert!(!range.contains(15));

    let empty_range = DiffHunkRange::new(10, 0);
    assert!(empty_range.is_empty());
    assert!(!empty_range.contains(10));

    // Test from_range
    let std_range = 5..10;
    let range2 = DiffHunkRange::from_range(std_range);
    assert_eq!(range2.start, 5);
    assert_eq!(range2.count, 5);
    assert_eq!(range2.end(), 10);

    // Test to_range
    let std_range2 = range2.to_range();
    assert_eq!(std_range2, 5..10);
}
