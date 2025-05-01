use anyhow::Result;
use similar::{Algorithm, ChangeTag, TextDiff as SimilarTextDiff};
use std::time::Duration;

use crate::buffer_diff::BufferDiff;

/// Granularity for diff operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffGranularity {
    /// Diff by lines (default)
    Line,
    /// Diff by words (more detailed)
    Word,
    /// Diff by characters (highest detail)
    Character,
}

/// Configuration for diff operations
#[derive(Debug, Clone)]
pub struct DiffConfig {
    /// The algorithm to use for diffing
    pub algorithm: Algorithm,
    /// The granularity of the diff
    pub granularity: DiffGranularity,
    /// The timeout for diffing operations (in seconds)
    pub timeout_seconds: u64,
    /// The number of context lines to include
    pub context_lines: usize,
    /// Whether to ignore whitespace changes
    pub ignore_whitespace: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Myers, // Myers is usually the best default
            granularity: DiffGranularity::Line, // Line-level diffing by default
            timeout_seconds: 5, // 5 second timeout
            context_lines: 3, // Default context lines
            ignore_whitespace: false, // Don't ignore whitespace by default
        }
    }
}

impl DiffConfig {
    /// Set the diff algorithm
    pub fn algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }
    
    /// Set the diff granularity
    pub fn granularity(mut self, granularity: DiffGranularity) -> Self {
        self.granularity = granularity;
        self
    }
    
    /// Set the timeout in seconds
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
    
    /// Set the number of context lines
    pub fn context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }
    
    /// Set whether to ignore whitespace
    pub fn ignore_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_whitespace = ignore;
        self
    }
    
    /// Create a diff between two texts using this configuration
    pub fn diff(&self, old_text: &str, new_text: &str) -> Result<BufferDiff> {
        // Apply whitespace handling if needed
        let (old_processed, new_processed) = if self.ignore_whitespace {
            (self.normalize_whitespace(old_text), self.normalize_whitespace(new_text))
        } else {
            (old_text.to_string(), new_text.to_string())
        };
        
        // Delegate to the appropriate diff method based on granularity
        match self.granularity {
            DiffGranularity::Line => TextDiff::diff(&old_processed, &new_processed),
            DiffGranularity::Word => TextDiff::diff_words(&old_processed, &new_processed),
            DiffGranularity::Character => TextDiff::diff_chars(&old_processed, &new_processed),
        }
    }
    
    /// Generate a unified diff string using this configuration
    pub fn unified_diff(&self, old_text: &str, new_text: &str) -> String {
        // Apply whitespace handling if needed
        let (old_processed, new_processed) = if self.ignore_whitespace {
            (self.normalize_whitespace(old_text), self.normalize_whitespace(new_text))
        } else {
            (old_text.to_string(), new_text.to_string())
        };
        
        // Apply the granularity based on configuration
        let diff = match self.granularity {
            DiffGranularity::Line => SimilarTextDiff::configure()
                .algorithm(self.algorithm)
                .timeout(Duration::from_secs(self.timeout_seconds))
                .diff_lines(&old_processed, &new_processed),
            DiffGranularity::Word => SimilarTextDiff::configure()
                .algorithm(self.algorithm)
                .timeout(Duration::from_secs(self.timeout_seconds))
                .diff_words(&old_processed, &new_processed),
            DiffGranularity::Character => SimilarTextDiff::configure()
                .algorithm(self.algorithm)
                .timeout(Duration::from_secs(self.timeout_seconds))
                .diff_chars(&old_processed, &new_processed),
        };
        
        // Generate the unified diff
        let mut result = String::new();
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            
            // Add the sign and the value
            result.push_str(sign);
            result.push_str(change.value());
            result.push('\n');
        }
        
        result
    }
    
    /// Normalize whitespace in a string (for ignore_whitespace option)
    fn normalize_whitespace(&self, text: &str) -> String {
        // Replace all consecutive whitespace with a single space and trim
        let mut result = String::new();
        let mut in_whitespace = false;
        
        for c in text.chars() {
            if c.is_whitespace() {
                if !in_whitespace {
                    result.push(' ');
                    in_whitespace = true;
                }
            } else {
                result.push(c);
                in_whitespace = false;
            }
        }
        
        // Trim leading and trailing whitespace
        result.trim().to_string()
    }
}

/// Wrapper around text diff operations
pub struct TextDiff;

impl TextDiff {
    /// Create a new diff configuration with default settings
    pub fn configure() -> DiffConfig {
        DiffConfig::default()
    }
    
    /// Create a diff between two texts using default configuration
    pub fn diff(old_text: &str, new_text: &str) -> Result<BufferDiff> {
        BufferDiff::new(old_text, new_text)
    }

    /// Create a diff with the specified granularity using default config for other settings
    pub fn diff_with_granularity(old_text: &str, new_text: &str, granularity: DiffGranularity) -> Result<BufferDiff> {
        Self::configure().granularity(granularity).diff(old_text, new_text)
    }
    
    /// Create a diff between two texts, at the word level
    fn diff_words(old_text: &str, new_text: &str) -> Result<BufferDiff> {
        // Convert to lines first to maintain structure
        let old_lines: Vec<&str> = old_text.lines().collect();
        let new_lines: Vec<&str> = new_text.lines().collect();
        
        // Process each line pair with word-level diffing
        let mut processed_old = Vec::new();
        let mut processed_new = Vec::new();
        
        // We'll compare corresponding lines and expand them with word markers
        let max_lines = old_lines.len().max(new_lines.len());
        
        for i in 0..max_lines {
            if i < old_lines.len() && i < new_lines.len() {
                // If both lines exist, do word-level diff
                let old_line = old_lines[i];
                let new_line = new_lines[i];
                
                if old_line == new_line {
                    // Lines are identical, keep as is
                    processed_old.push(old_line.to_string());
                    processed_new.push(new_line.to_string());
                } else {
                    // Lines differ, expand to word-level diff
                    Self::expand_line_to_words(old_line, new_line, &mut processed_old, &mut processed_new);
                }
            } else if i < old_lines.len() {
                // Only old line exists
                processed_old.push(old_lines[i].to_string());
            } else if i < new_lines.len() {
                // Only new line exists
                processed_new.push(new_lines[i].to_string());
            }
        }
        
        // Rejoin the processed lines
        let processed_old_text = processed_old.join("\n");
        let processed_new_text = processed_new.join("\n");
        
        // Create diff using the processed texts
        BufferDiff::new(&processed_old_text, &processed_new_text)
    }
    
    /// Expand a line to word-level differences
    fn expand_line_to_words(old_line: &str, new_line: &str, processed_old: &mut Vec<String>, processed_new: &mut Vec<String>) {
        // Use similar to do word-level diffing for this line
        let line_diff = SimilarTextDiff::configure()
            .algorithm(Algorithm::Myers)
            .timeout(Duration::from_secs(1))
            .diff_words(old_line, new_line);
        
        let mut old_expanded = String::new();
        let mut new_expanded = String::new();
        
        // Process each word change
        for change in line_diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    // Same words in both lines
                    old_expanded.push_str(change.value());
                    new_expanded.push_str(change.value());
                }
                ChangeTag::Delete => {
                    // Word exists only in old line
                    old_expanded.push_str(change.value());
                }
                ChangeTag::Insert => {
                    // Word exists only in new line
                    new_expanded.push_str(change.value());
                }
            }
        }
        
        // Add the expanded lines
        processed_old.push(old_expanded);
        processed_new.push(new_expanded);
    }
    
    /// Create a diff between two texts, at the character level
    fn diff_chars(old_text: &str, new_text: &str) -> Result<BufferDiff> {
        // For character level diffing, we'll use similar directly to avoid excessive line expansion
        let diff = SimilarTextDiff::configure()
            .algorithm(Algorithm::Myers)
            .timeout(Duration::from_secs(5))
            .diff_chars(old_text, new_text);
        
        // Convert back to lines for our BufferDiff
        let mut processed_old = String::new();
        let mut processed_new = String::new();
        
        // Process each change
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    processed_old.push_str(change.value());
                    processed_new.push_str(change.value());
                }
                ChangeTag::Delete => {
                    processed_old.push_str(change.value());
                }
                ChangeTag::Insert => {
                    processed_new.push_str(change.value());
                }
            }
        }
        
        // Create diff using the processed texts
        BufferDiff::new(&processed_old, &processed_new)
    }

    /// Generate a unified diff string (like git diff) with default settings
    pub fn unified_diff(old_text: &str, new_text: &str, context_lines: usize) -> String {
        Self::configure()
            .context_lines(context_lines)
            .unified_diff(old_text, new_text)
    }
    
    /// Generate a unified diff string with specified granularity
    pub fn unified_diff_with_granularity(old_text: &str, new_text: &str, context_lines: usize, granularity: DiffGranularity) -> String {
        Self::configure()
            .granularity(granularity)
            .context_lines(context_lines)
            .unified_diff(old_text, new_text)
    }
}

