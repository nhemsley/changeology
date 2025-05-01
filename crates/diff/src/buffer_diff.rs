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

        // If no changes, create a single unchanged hunk
        if !diff
            .iter_all_changes()
            .any(|c| c.tag() != similar::ChangeTag::Equal)
        {
            let old_line_count = self.old_text.len_lines().saturating_sub(1);
            let new_line_count = self.new_text.len_lines().saturating_sub(1);

            self.hunks.push(DiffHunk::new(
                DiffHunkStatus::Unchanged,
                0,
                old_line_count,
                0,
                new_line_count,
            ));
            return Ok(());
        }

        // Track current position in both texts
        let mut old_pos = 0;
        let mut new_pos = 0;

        // Current hunk being built
        let mut current_hunk: Option<DiffHunk> = None;

        // Process each change in the diff
        for change in diff.iter_all_changes() {
            match change.tag() {
                similar::ChangeTag::Equal => {
                    // If we had a hunk in progress, finalize it
                    if let Some(hunk) = current_hunk.take() {
                        self.hunks.push(hunk);
                    }

                    // Move positions forward
                    old_pos += 1;
                    new_pos += 1;
                }
                similar::ChangeTag::Delete => {
                    if current_hunk.is_none() {
                        // Start a new hunk
                        if new_pos < self.new_text.len_lines() {
                            // There are still lines in the new text, so this is a modified hunk
                            current_hunk = Some(DiffHunk::new(
                                DiffHunkStatus::Modified,
                                old_pos,
                                1,
                                new_pos,
                                0,
                            ));
                        } else {
                            // No more lines in new text, this is a deletion hunk
                            current_hunk = Some(DiffHunk::new(
                                DiffHunkStatus::Deleted,
                                old_pos,
                                1,
                                new_pos,
                                0,
                            ));
                        }
                    } else {
                        // Extend the current hunk
                        if let Some(hunk) = &mut current_hunk {
                            hunk.old_range.count += 1;
                            // Add a deleted line to the hunk
                            hunk.line_types
                                .push(crate::diff_hunk::DiffLineType::OldOnly);
                        }
                    }

                    // Move old position forward
                    old_pos += 1;
                }
                similar::ChangeTag::Insert => {
                    if current_hunk.is_none() {
                        // Start a new hunk
                        if old_pos > 0 {
                            // There were lines in the old text, so this is a modified hunk
                            current_hunk = Some(DiffHunk::new(
                                DiffHunkStatus::Modified,
                                old_pos,
                                0,
                                new_pos,
                                1,
                            ));
                        } else {
                            // No lines in old text, this is an addition hunk
                            current_hunk =
                                Some(DiffHunk::new(DiffHunkStatus::Added, old_pos, 0, new_pos, 1));
                        }
                    } else {
                        // Extend the current hunk
                        if let Some(hunk) = &mut current_hunk {
                            hunk.new_range.count += 1;
                            // Add a new line to the hunk
                            hunk.line_types
                                .push(crate::diff_hunk::DiffLineType::NewOnly);
                        }
                    }

                    // Move new position forward
                    new_pos += 1;
                }
            }
        }

        // If we have a hunk in progress, finalize it
        if let Some(hunk) = current_hunk.take() {
            self.hunks.push(hunk);
        }

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
