use anyhow::Result;
use ropey::Rope;

use crate::diff_hunk::{DiffHunk, DiffHunkStatus};

/// Represents a diff between two buffers (text documents)
#[derive(Debug, Clone)]
pub struct BufferDiff {
    /// The old version of the text
    old_text: Rope,

    /// The new version of the text
    new_text: Rope,

    /// The hunks in this diff
    hunks: Vec<DiffHunk>,
}

/// An immutable snapshot of a buffer diff
#[derive(Debug, Clone)]
pub struct BufferDiffSnapshot {
    /// The hunks in this diff
    pub hunks: Vec<DiffHunk>,

    /// The number of lines in the old text
    pub old_line_count: usize,

    /// The number of lines in the new text
    pub new_line_count: usize,
}

impl BufferDiff {
    /// Create a new buffer diff between two texts
    pub fn new(old_text: &str, new_text: &str) -> Result<Self> {
        let old_rope = Rope::from_str(old_text);
        let new_rope = Rope::from_str(new_text);

        let mut diff = Self {
            old_text: old_rope,
            new_text: new_rope,
            hunks: Vec::new(),
        };

        // Compute the hunks
        diff.compute_hunks()?;

        Ok(diff)
    }

    /// Compute the hunks between the old and new text
    fn compute_hunks(&mut self) -> Result<()> {
        // We'll use similar crate to compute the diff
        let old_text_str = self.old_text.to_string();
        let new_text_str = self.new_text.to_string();

        // Get diff from similar crate
        let diff = similar::TextDiff::from_lines(&old_text_str, &new_text_str);

        // Special case: if both are empty
        if old_text_str.is_empty() && new_text_str.is_empty() {
            self.hunks
                .push(DiffHunk::new(DiffHunkStatus::Unchanged, 0, 0, 0, 0));
            return Ok(());
        }

        // Special case: if old is empty but new is not, this is an added file
        if old_text_str.is_empty() && !new_text_str.is_empty() {
            let new_line_count = self.new_text.len_lines().saturating_sub(1);
            if new_line_count == 0 {
                // Single line with no newline
                let new_line_count = 1;
                let mut hunk = DiffHunk::new(DiffHunkStatus::Added, 0, 0, 0, new_line_count);
                // Set all lines to NewOnly
                hunk.line_types = vec![crate::diff_hunk::DiffLineType::NewOnly; new_line_count];
                self.hunks.push(hunk);
            } else {
                let mut hunk = DiffHunk::new(DiffHunkStatus::Added, 0, 0, 0, new_line_count);
                // Set all lines to NewOnly
                hunk.line_types = vec![crate::diff_hunk::DiffLineType::NewOnly; new_line_count];
                self.hunks.push(hunk);
            }
            return Ok(());
        }

        // Special case: if new is empty but old is not, this is a deleted file
        if !old_text_str.is_empty() && new_text_str.is_empty() {
            let old_line_count = self.old_text.len_lines().saturating_sub(1);
            if old_line_count == 0 {
                // Single line with no newline
                let old_line_count = 1;
                let mut hunk = DiffHunk::new(DiffHunkStatus::Deleted, 0, old_line_count, 0, 0);
                // Set all lines to OldOnly
                hunk.line_types = vec![crate::diff_hunk::DiffLineType::OldOnly; old_line_count];
                self.hunks.push(hunk);
            } else {
                let mut hunk = DiffHunk::new(DiffHunkStatus::Deleted, 0, old_line_count, 0, 0);
                // Set all lines to OldOnly
                hunk.line_types = vec![crate::diff_hunk::DiffLineType::OldOnly; old_line_count];
                self.hunks.push(hunk);
            }
            return Ok(());
        }

        // If no changes, create a single unchanged hunk
        if !diff
            .iter_all_changes()
            .any(|c| c.tag() != similar::ChangeTag::Equal)
        {
            let old_line_count = self.old_text.len_lines().saturating_sub(1);
            let new_line_count = self.new_text.len_lines().saturating_sub(1);

            // Ensure we have at least 1 line if the text contains content
            let old_line_count = if old_line_count == 0 && !old_text_str.is_empty() {
                1
            } else {
                old_line_count
            };
            let new_line_count = if new_line_count == 0 && !new_text_str.is_empty() {
                1
            } else {
                new_line_count
            };

            let mut hunk = DiffHunk::new(
                DiffHunkStatus::Unchanged,
                0,
                old_line_count,
                0,
                new_line_count,
            );

            // Set correct line types
            hunk.line_types = vec![crate::diff_hunk::DiffLineType::Both; old_line_count];

            self.hunks.push(hunk);
            return Ok(());
        }

        // Process diffs to create hunks
        self.process_diffs(diff)?;

        Ok(())
    }

    /// Process the diffs to create hunks
    fn process_diffs<'a>(&mut self, diff: similar::TextDiff<'a, 'a, 'a, str>) -> Result<()> {
        // Keep track of unchanged lines for context
        let mut unchanged_lines: Vec<String> = Vec::new();
        let mut unchanged_start_old = 0;
        let mut unchanged_start_new = 0;

        // Track current position in both texts
        let mut old_pos = 0;
        let mut new_pos = 0;

        // Current changes being collected for a hunk
        let mut old_changes = Vec::new();
        let mut new_changes = Vec::new();

        // Process each change in the diff
        for change in diff.iter_all_changes() {
            match change.tag() {
                similar::ChangeTag::Equal => {
                    // If we had changes collected, create a hunk with some context
                    if !old_changes.is_empty() || !new_changes.is_empty() {
                        // Create a hunk with context from unchanged lines
                        let context_lines = 3; // Number of context lines before/after changes

                        // Add context before if available
                        let before_context = unchanged_lines.len().min(context_lines);
                        let context_start = unchanged_lines.len() - before_context;

                        // Include some unchanged lines before as context
                        let mut context_old_changes = Vec::new();
                        let mut context_new_changes = Vec::new();

                        // Add unchanged lines as context
                        for i in context_start..unchanged_lines.len() {
                            context_old_changes.push(unchanged_lines[i].clone());
                            context_new_changes.push(unchanged_lines[i].clone());
                        }

                        // Add actual changes
                        context_old_changes.extend(old_changes.clone());
                        context_new_changes.extend(new_changes.clone());

                        // Add the current line as context after if it's equal
                        context_old_changes.push(change.value().to_string());
                        context_new_changes.push(change.value().to_string());

                        // Calculate start positions with context
                        let old_start =
                            (unchanged_start_old + context_start).min(old_pos - old_changes.len());
                        let new_start =
                            (unchanged_start_new + context_start).min(new_pos - new_changes.len());

                        // Create the hunk with context
                        self.create_hunk_with_context(
                            old_start,
                            context_old_changes,
                            new_start,
                            context_new_changes,
                            before_context,
                            1,
                        )?;

                        // Reset collections
                        old_changes = Vec::new();
                        new_changes = Vec::new();
                        unchanged_lines = Vec::new();
                    }

                    // Track this unchanged line
                    if unchanged_lines.is_empty() {
                        unchanged_start_old = old_pos;
                        unchanged_start_new = new_pos;
                    }
                    unchanged_lines.push(change.value().to_string());

                    // Move positions forward
                    old_pos += 1;
                    new_pos += 1;
                }
                similar::ChangeTag::Delete => {
                    // Reset unchanged tracking on first change
                    if old_changes.is_empty() && new_changes.is_empty() {
                        // Keep only recent context lines
                        let context_lines = 3;
                        if unchanged_lines.len() > context_lines {
                            let keep = unchanged_lines.len() - context_lines;
                            unchanged_lines.drain(0..keep);
                            unchanged_start_old += keep;
                            unchanged_start_new += keep;
                        }
                    }

                    // Collect the deleted line
                    old_changes.push(change.value().to_string());
                    old_pos += 1;
                }
                similar::ChangeTag::Insert => {
                    // Reset unchanged tracking on first change
                    if old_changes.is_empty() && new_changes.is_empty() {
                        // Keep only recent context lines
                        let context_lines = 3;
                        if unchanged_lines.len() > context_lines {
                            let keep = unchanged_lines.len() - context_lines;
                            unchanged_lines.drain(0..keep);
                            unchanged_start_old += keep;
                            unchanged_start_new += keep;
                        }
                    }

                    // Collect the added line
                    new_changes.push(change.value().to_string());
                    new_pos += 1;
                }
            }
        }

        // If we have changes left, create a final hunk
        if !old_changes.is_empty() || !new_changes.is_empty() {
            // Create a hunk with context from unchanged lines
            let context_lines = 3;

            // Add context before if available
            let before_context = unchanged_lines.len().min(context_lines);
            let context_start = unchanged_lines.len() - before_context;

            // Include some unchanged lines before as context
            let mut context_old_changes = Vec::new();
            let mut context_new_changes = Vec::new();

            // Add unchanged lines as context
            for i in context_start..unchanged_lines.len() {
                context_old_changes.push(unchanged_lines[i].clone());
                context_new_changes.push(unchanged_lines[i].clone());
            }

            // Add actual changes
            context_old_changes.extend(old_changes.clone());
            context_new_changes.extend(new_changes.clone());

            // Calculate start positions with context
            let old_start = (unchanged_start_old + context_start).min(old_pos - old_changes.len());
            let new_start = (unchanged_start_new + context_start).min(new_pos - new_changes.len());

            // Create the hunk with context
            self.create_hunk_with_context(
                old_start,
                context_old_changes,
                new_start,
                context_new_changes,
                before_context,
                0,
            )?;
        }

        // If no hunks were created, create an unchanged hunk
        if self.hunks.is_empty() {
            let old_line_count = self.old_text.len_lines().saturating_sub(1);
            let new_line_count = self.new_text.len_lines().saturating_sub(1);

            self.hunks.push(DiffHunk::new(
                DiffHunkStatus::Unchanged,
                0,
                old_line_count,
                0,
                new_line_count,
            ));
        }

        Ok(())
    }

    /// Create a hunk from collected old and new changes with context lines
    fn create_hunk_with_context(
        &mut self,
        old_start: usize,
        old_changes: Vec<String>,
        new_start: usize,
        new_changes: Vec<String>,
        before_context: usize,
        after_context: usize,
    ) -> Result<()> {
        // Determine hunk status based on non-context changes
        let status = if old_changes.len() != new_changes.len() {
            // Different number of lines means there are adds/deletes
            if old_changes.len() > before_context + after_context
                && new_changes.len() > before_context + after_context
            {
                DiffHunkStatus::Modified
            } else if old_changes.len() <= before_context + after_context {
                DiffHunkStatus::Added
            } else {
                DiffHunkStatus::Deleted
            }
        } else {
            // Same number of lines - check if content differs
            let mut is_modified = false;

            for i in before_context..(old_changes.len() - after_context) {
                let j = i - before_context;
                if i < old_changes.len() && j + before_context < new_changes.len() {
                    if old_changes[i] != new_changes[j + before_context] {
                        is_modified = true;
                        break;
                    }
                }
            }

            if is_modified {
                DiffHunkStatus::Modified
            } else {
                DiffHunkStatus::Unchanged
            }
        };

        // Create the hunk
        let old_count = old_changes.len();
        let new_count = new_changes.len();

        let mut hunk = DiffHunk::new(status, old_start, old_count, new_start, new_count);

        // Set the line types based on the changes and context
        let mut line_types = Vec::new();

        // Process context lines at the beginning
        for _ in 0..before_context {
            line_types.push(crate::diff_hunk::DiffLineType::Both);
        }

        // For modified hunks, compare lines and mark appropriately
        if status == DiffHunkStatus::Modified {
            let old_content_start = before_context;
            let old_content_end = old_count - after_context;
            let new_content_start = before_context;
            let new_content_end = new_count - after_context;

            // Simple diff - if old and new have different lengths, mark extras as added/deleted
            let min_length =
                (old_content_end - old_content_start).min(new_content_end - new_content_start);

            // Compare common length, mark changes
            for i in 0..min_length {
                let old_idx = old_content_start + i;
                let new_idx = new_content_start + i;

                if old_idx < old_changes.len() && new_idx < new_changes.len() {
                    if old_changes[old_idx] == new_changes[new_idx] {
                        line_types.push(crate::diff_hunk::DiffLineType::Both);
                    } else {
                        // This is a modified line, mark old version
                        line_types.push(crate::diff_hunk::DiffLineType::OldOnly);
                        // Mark new version in next iteration
                        line_types.push(crate::diff_hunk::DiffLineType::NewOnly);
                    }
                }
            }

            // Add any remaining old lines
            for _ in old_content_start + min_length..old_content_end {
                line_types.push(crate::diff_hunk::DiffLineType::OldOnly);
            }

            // Add any remaining new lines
            for _ in new_content_start + min_length..new_content_end {
                line_types.push(crate::diff_hunk::DiffLineType::NewOnly);
            }
        } else if status == DiffHunkStatus::Added {
            // For added hunks, all non-context lines are NewOnly
            for _ in before_context..(new_count - after_context) {
                line_types.push(crate::diff_hunk::DiffLineType::NewOnly);
            }
        } else if status == DiffHunkStatus::Deleted {
            // For deleted hunks, all non-context lines are OldOnly
            for _ in before_context..(old_count - after_context) {
                line_types.push(crate::diff_hunk::DiffLineType::OldOnly);
            }
        }

        // Process context lines at the end
        for _ in 0..after_context {
            line_types.push(crate::diff_hunk::DiffLineType::Both);
        }

        // Set the line types on the hunk
        hunk.line_types = line_types;

        self.hunks.push(hunk);

        Ok(())
    }

    /// Get a snapshot of the current diff
    pub fn snapshot(&self) -> BufferDiffSnapshot {
        BufferDiffSnapshot {
            hunks: self.hunks.clone(),
            old_line_count: self.old_text.len_lines(),
            new_line_count: self.new_text.len_lines(),
        }
    }

    /// Get the old text
    pub fn old_text(&self) -> &Rope {
        &self.old_text
    }

    /// Get the new text
    pub fn new_text(&self) -> &Rope {
        &self.new_text
    }

    /// Get the hunks
    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    /// Get the number of hunks
    pub fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    /// Get a hunk by index
    pub fn hunk(&self, index: usize) -> Option<&DiffHunk> {
        self.hunks.get(index)
    }
}

impl BufferDiffSnapshot {
    /// Create a new empty diff snapshot
    pub fn empty() -> Self {
        Self {
            hunks: Vec::new(),
            old_line_count: 0,
            new_line_count: 0,
        }
    }

    /// Get the hunks
    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    /// Get the number of hunks
    pub fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    /// Get a hunk by index
    pub fn hunk(&self, index: usize) -> Option<&DiffHunk> {
        self.hunks.get(index)
    }

    /// Check if the diff has any changes
    pub fn has_changes(&self) -> bool {
        self.hunks.iter().any(|h| h.has_changes())
    }

    /// Get the number of added lines
    pub fn added_lines(&self) -> usize {
        self.hunks.iter().map(|h| h.added_lines()).sum()
    }

    /// Get the number of deleted lines
    pub fn deleted_lines(&self) -> usize {
        self.hunks.iter().map(|h| h.deleted_lines()).sum()
    }

    /// Get the number of unchanged lines
    pub fn unchanged_lines(&self) -> usize {
        self.hunks.iter().map(|h| h.unchanged_lines()).sum()
    }
}
