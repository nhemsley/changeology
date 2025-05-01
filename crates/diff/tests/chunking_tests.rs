use diff::{BufferDiff, DiffHunkStatus};

#[test]
fn test_large_file_chunking() {
    // Create a large text file (over the chunking threshold)
    let mut old_text = String::new();
    let mut new_text = String::new();
    
    // Create content big enough to trigger chunking
    for i in 0..2000 {
        old_text.push_str(&format!("Line {} of the old text\n", i));
        
        // Make some deliberate changes to test diff accuracy
        if i % 100 == 0 {
            // Every 100th line is different
            new_text.push_str(&format!("Modified line {} of the new text\n", i));
        } else {
            new_text.push_str(&format!("Line {} of the old text\n", i));
        }
    }
    
    // Create a diff
    let buffer_diff = BufferDiff::new(&old_text, &new_text).unwrap();
    
    // Get a snapshot
    let snapshot = buffer_diff.snapshot();
    
    // Verify that we have the expected number of hunks (we should have changes)
    assert!(snapshot.has_changes());
    
    // We should have exactly 20 modified hunks (one per 100 lines)
    let modified_hunks = snapshot.hunks().iter()
        .filter(|h| h.status == DiffHunkStatus::Modified)
        .count();
    
    assert_eq!(modified_hunks, 20);
    
    // Verify total line counts
    assert_eq!(snapshot.old_line_count, 2001); // 2000 lines + final newline
    assert_eq!(snapshot.new_line_count, 2001); // 2000 lines + final newline
    
    // The number of changes should match our pattern (20 lines modified)
    let total_changes = snapshot.hunks().iter()
        .filter(|h| h.status != DiffHunkStatus::Unchanged)
        .fold(0, |acc, h| acc + h.line_types.len());
        
    assert!(total_changes >= 20); // At least one line per changed hunk
}

#[test]
fn test_merge_adjacent_hunks() {
    // Create text with changes that will generate adjacent hunks
    let old_text = "Line 1\nLine 2\nLine 3\nLine 4\n";
    
    // Modify lines 2 and 3 to ensure they're close enough to merge
    let new_text = "Line 1\nModified 2\nModified 3\nLine 4\n";
    
    // Create a diff
    let buffer_diff = BufferDiff::new(old_text, new_text).unwrap();
    
    // Get a snapshot
    let snapshot = buffer_diff.snapshot();
    
    // For this small file, with changes this close together, we should have a single hunk
    assert!(snapshot.hunk_count() == 1, 
        "Expected 1 merged hunk, got {} hunks", snapshot.hunk_count());
    
    // The hunk should contain both modified lines
    let hunk = &snapshot.hunks()[0];
    assert!(hunk.old_range.contains(1), "Hunk should contain line 2"); // Line 2
    assert!(hunk.old_range.contains(2), "Hunk should contain line 3"); // Line 3
}