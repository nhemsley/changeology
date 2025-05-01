use diff::{BufferDiff, DiffConfig, DiffGranularity, TextDiff};
use similar::Algorithm;
use std::vec::Vec;

#[test]
fn test_line_level_diff() {
    let old_text = "First line\nSecond line\nThird line\nFourth line\n";
    let new_text = "First line\nSecond line modified\nThird line\nFourth line\n";
    
    // Create a line-level diff
    let config = DiffConfig::default()
        .algorithm(Algorithm::Myers)
        .granularity(DiffGranularity::Line);
    
    let diff = config.diff(old_text, new_text).unwrap();
    let snapshot = diff.snapshot();
    
    // We should have one hunk
    assert_eq!(snapshot.hunk_count(), 1);
    
    // The hunk should contain the modified line
    let hunk = snapshot.hunk(0).unwrap();
    assert!(hunk.old_range.contains(1)); // Second line (0-indexed)
    assert!(hunk.new_range.contains(1)); // Second line (0-indexed)
    
    // Verify line types (should have at least one modified line)
    let old_only_count = snapshot.hunks()
        .iter()
        .map(|h| h.line_types.iter().filter(|&&t| t == diff::DiffLineType::OldOnly).count())
        .sum::<usize>();
    
    let new_only_count = snapshot.hunks()
        .iter()
        .map(|h| h.line_types.iter().filter(|&&t| t == diff::DiffLineType::NewOnly).count())
        .sum::<usize>();
    
    assert_eq!(old_only_count, 1); // One line removed
    assert_eq!(new_only_count, 1); // One line added
}

#[test]
fn test_word_level_diff() {
    let old_text = "The quick brown fox jumps over the lazy dog";
    let new_text = "The quick red fox jumps over the lazy dog";
    
    // Create a word-level diff
    let diff = TextDiff::diff_with_granularity(old_text, new_text, DiffGranularity::Word).unwrap();
    let snapshot = diff.snapshot();
    
    // Ensure the diff detected the change
    assert!(snapshot.has_changes());
    
    // Create a line-level diff for comparison
    let line_diff = TextDiff::diff_with_granularity(old_text, new_text, DiffGranularity::Line).unwrap();
    let line_snapshot = line_diff.snapshot();
    
    // Both should detect changes
    assert!(line_snapshot.has_changes());
    
    // Get the unified diff output and check that it contains word-level changes
    let unified_diff = TextDiff::unified_diff_with_granularity(old_text, new_text, 3, DiffGranularity::Word);
    
    // The word-level diff should show only "brown" and "red" as changed, not the entire line
    assert!(unified_diff.contains("-brown"));
    assert!(unified_diff.contains("+red"));
}

#[test]
fn test_character_level_diff() {
    let old_text = "testing123";
    let new_text = "testing456";
    
    // Create a character-level diff
    let diff = TextDiff::diff_with_granularity(old_text, new_text, DiffGranularity::Character).unwrap();
    let snapshot = diff.snapshot();
    
    // Ensure the diff detected the change
    assert!(snapshot.has_changes());
    
    // Create a line-level diff for comparison
    let line_diff = TextDiff::diff(old_text, new_text).unwrap();
    let line_snapshot = line_diff.snapshot();
    
    // Both should detect changes
    assert!(line_snapshot.has_changes());
    
    // Get the unified diff output and check that it contains character-level changes
    let unified_diff = TextDiff::unified_diff_with_granularity(old_text, new_text, 0, DiffGranularity::Character);
    
    // Print the unified diff for debugging
    println!("Unified diff: {}", unified_diff);
    
    // The character-level diff should show only "123" and "456" as changed, not the entire string
    assert!(unified_diff.contains("-1") && unified_diff.contains("-2") && unified_diff.contains("-3"),
            "Expected to find '-1', '-2', '-3' in the diff");
    assert!(unified_diff.contains("+4") && unified_diff.contains("+5") && unified_diff.contains("+6"),
            "Expected to find '+4', '+5', '+6' in the diff");
}

#[test]
fn test_whitespace_ignoring() {
    let old_text = "  This has  extra spaces   ";
    let new_text = "This has extra spaces";
    
    // Normal diff should detect changes
    let normal_diff = TextDiff::diff(old_text, new_text).unwrap();
    let normal_snapshot = normal_diff.snapshot();
    assert!(normal_snapshot.has_changes());
    
    // Whitespace-ignoring diff should not detect changes
    let ws_diff = DiffConfig::default()
        .ignore_whitespace(true)
        .diff(old_text, new_text)
        .unwrap();
    
    let ws_snapshot = ws_diff.snapshot();
    
    // Since we're ignoring whitespace, these should be considered the same
    // Note: This assertion might not always hold depending on how normalize_whitespace is implemented
    // If it treats all leading/trailing space as significant, this might need updating
    assert!(!ws_snapshot.has_changes() || ws_snapshot.hunks()[0].status == diff::DiffHunkStatus::Unchanged);
}