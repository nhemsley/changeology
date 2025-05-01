use diff::{BufferDiff, DiffHunkStatus, TextDiff};

#[test]
fn test_newlines_at_end() {
    // Test handling of files with and without trailing newlines
    
    // Case 1: Both with trailing newlines
    let old1 = "Line 1\nLine 2\n";
    let new1 = "Line 1\nLine X\n";
    
    let diff1 = BufferDiff::new(old1, new1).unwrap();
    
    // Case 2: Old with trailing newline, new without
    let old2 = "Line 1\nLine 2\n";
    let new2 = "Line 1\nLine X";
    
    let diff2 = BufferDiff::new(old2, new2).unwrap();
    
    // Case 3: Old without trailing newline, new with
    let old3 = "Line 1\nLine 2";
    let new3 = "Line 1\nLine X\n";
    
    let diff3 = BufferDiff::new(old3, new3).unwrap();
    
    // Case 4: Neither with trailing newlines
    let old4 = "Line 1\nLine 2";
    let new4 = "Line 1\nLine X";
    
    let diff4 = BufferDiff::new(old4, new4).unwrap();
    
    // All should produce valid diffs without crashing
    assert!(diff1.snapshot().hunk_count() > 0);
    assert!(diff2.snapshot().hunk_count() > 0);
    assert!(diff3.snapshot().hunk_count() > 0);
    assert!(diff4.snapshot().hunk_count() > 0);
}

#[test]
fn test_very_large_diff() {
    // Create large strings (but not too large for testing)
    let mut old = String::new();
    let mut new = String::new();
    
    // 1000 lines
    for i in 0..1000 {
        old.push_str(&format!("Line {} of old text\n", i));
        
        // Make every 10th line different
        if i % 10 == 0 {
            new.push_str(&format!("MODIFIED Line {} of new text\n", i));
        } else {
            new.push_str(&format!("Line {} of old text\n", i));
        }
    }
    
    // Should handle large files without issues
    let diff = BufferDiff::new(&old, &new).unwrap();
    let snapshot = diff.snapshot();
    
    // Should find some changes
    assert!(snapshot.hunk_count() > 0);
    assert!(snapshot.added_lines() > 0);
    assert!(snapshot.deleted_lines() > 0);
}

#[test]
fn test_unicode_text() {
    // Test with unicode text
    let old = "Line 1\nLine 2 🚀\nLine 3 😊\n";
    let new = "Line 1\nLine 2 🚀\nLine 3 🎉\n";
    
    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();
    
    // Should handle unicode correctly
    assert!(snapshot.hunk_count() > 0);
    assert!(snapshot.has_changes());
    
    // Unicode should also work in unified diff
    let unified = TextDiff::unified_diff(old, new, 1);
    assert!(unified.contains("😊"));
    assert!(unified.contains("🎉"));
}

#[test]
fn test_empty_hunk_snapshot() {
    // Test the empty() constructor for BufferDiffSnapshot
    use diff::BufferDiffSnapshot;
    
    let snapshot = BufferDiffSnapshot::empty();
    
    assert_eq!(snapshot.hunk_count(), 0);
    assert_eq!(snapshot.added_lines(), 0);
    assert_eq!(snapshot.deleted_lines(), 0);
    assert_eq!(snapshot.unchanged_lines(), 0);
    assert!(!snapshot.has_changes());
}

#[test]
fn test_diff_with_only_whitespace_changes() {
    // Test diffs with only whitespace changes
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine  2\nLine 3\n";  // Extra space in Line 2
    
    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot = diff.snapshot();
    
    // Whitespace changes should be detected
    assert!(snapshot.hunk_count() > 0);
    assert!(snapshot.has_changes());
}

#[test]
fn test_snapshot_clone() {
    // Test that snapshot cloning works correctly
    let old = "Line 1\nLine 2\nLine 3\n";
    let new = "Line 1\nLine X\nLine 3\n";
    
    let diff = BufferDiff::new(old, new).unwrap();
    let snapshot1 = diff.snapshot();
    
    // Clone the snapshot
    let snapshot2 = snapshot1.clone();
    
    // Both should be identical
    assert_eq!(snapshot1.hunk_count(), snapshot2.hunk_count());
    assert_eq!(snapshot1.added_lines(), snapshot2.added_lines());
    assert_eq!(snapshot1.deleted_lines(), snapshot2.deleted_lines());
    assert_eq!(snapshot1.unchanged_lines(), snapshot2.unchanged_lines());
}