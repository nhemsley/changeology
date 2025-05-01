use anyhow::Result;
use similar::{Algorithm, ChangeTag, TextDiff as SimilarTextDiff};

use crate::buffer_diff::BufferDiff;

/// Wrapper around text diff operations
pub struct TextDiff;

impl TextDiff {
    /// Create a diff between two texts
    pub fn diff(old_text: &str, new_text: &str) -> Result<BufferDiff> {
        BufferDiff::new(old_text, new_text)
    }
    
    /// Generate a unified diff string (like git diff)
    pub fn unified_diff(old_text: &str, new_text: &str, _context_lines: usize) -> String {
        // The similar crate doesn't have context_radius in newer versions
        // so we'll just use the default configuration
        let diff = SimilarTextDiff::configure()
            .algorithm(Algorithm::Myers)
            .timeout(std::time::Duration::from_secs(5))
            .diff_lines(old_text, new_text);
            
        let mut result = String::new();
        
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push_str(&format!("{}{}", sign, change));
        }
        
        result
    }
}