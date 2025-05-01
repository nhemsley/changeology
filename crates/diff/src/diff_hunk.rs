use derive_more::Display;
use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the status of a diff hunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DiffHunkStatus {
    /// The hunk represents added content (only exists in new version)
    #[display(fmt = "Added")]
    Added,

    /// The hunk represents deleted content (only exists in old version)
    #[display(fmt = "Deleted")]
    Deleted,

    /// The hunk represents modified content (exists in both versions but different)
    #[display(fmt = "Modified")]
    Modified,

    /// The hunk represents unchanged content (exists in both versions and identical)
    #[display(fmt = "Unchanged")]
    Unchanged,
}

/// Represents the secondary status of a diff hunk in the context of git
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DiffHunkSecondaryStatus {
    /// The hunk is staged (in the index)
    #[display(fmt = "Staged")]
    Staged,

    /// The hunk is unstaged (in the working directory)
    #[display(fmt = "Unstaged")]
    Unstaged,

    /// The hunk has no secondary status
    #[display(fmt = "None")]
    None,
}

/// Represents a range of lines in a diff
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DiffHunkRange {
    /// The starting line (0-based)
    pub start: usize,

    /// The number of lines
    pub count: usize,
}

impl DiffHunkRange {
    /// Create a new range from start and count
    pub fn new(start: usize, count: usize) -> Self {
        Self { start, count }
    }

    /// Create a range from a start and end (exclusive)
    pub fn from_range(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            count: range.end - range.start,
        }
    }

    /// Convert to a standard Range
    pub fn to_range(&self) -> Range<usize> {
        self.start..(self.start + self.count)
    }

    /// Get the end of the range (exclusive)
    pub fn end(&self) -> usize {
        self.start + self.count
    }

    /// Check if this range is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if this range contains the given line
    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line < self.end()
    }
}

/// Represents the type of a line in a diff hunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DiffLineType {
    /// Line only exists in old version (deleted)
    OldOnly,

    /// Line only exists in new version (added)
    NewOnly,

    /// Line exists in both versions (unchanged or part of modified hunk)
    Both,
}

/// Represents a hunk of changes between two versions of text
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DiffHunk {
    /// The primary status of the hunk
    pub status: DiffHunkStatus,

    /// The secondary status of the hunk (for git integration)
    pub secondary_status: DiffHunkSecondaryStatus,

    /// The range of lines in the old version
    pub old_range: DiffHunkRange,

    /// The range of lines in the new version
    pub new_range: DiffHunkRange,

    /// Line-by-line mapping of line types within this hunk
    pub line_types: Vec<DiffLineType>,
}

impl DiffHunk {
    /// Create a new diff hunk
    pub fn new(
        status: DiffHunkStatus,
        old_start: usize,
        old_count: usize,
        new_start: usize,
        new_count: usize,
    ) -> Self {
        // Initialize line_types based on status
        let line_types = match status {
            DiffHunkStatus::Added => {
                // All lines are new only
                vec![DiffLineType::NewOnly; new_count]
            }
            DiffHunkStatus::Deleted => {
                // All lines are old only
                vec![DiffLineType::OldOnly; old_count]
            }
            DiffHunkStatus::Modified => {
                // For modified hunks, we need to determine line-by-line later
                // This is a placeholder that will be filled in by the diff algorithm
                let total_lines = old_count.max(new_count);
                vec![DiffLineType::Both; total_lines]
            }
            DiffHunkStatus::Unchanged => {
                // All lines are both
                vec![DiffLineType::Both; old_count]
            }
        };

        Self {
            status,
            secondary_status: DiffHunkSecondaryStatus::None,
            old_range: DiffHunkRange::new(old_start, old_count),
            new_range: DiffHunkRange::new(new_start, new_count),
            line_types,
        }
    }

    /// Check if this hunk has any changes
    pub fn has_changes(&self) -> bool {
        self.status != DiffHunkStatus::Unchanged
    }

    /// Get the number of added lines in this hunk
    pub fn added_lines(&self) -> usize {
        self.line_types
            .iter()
            .filter(|&&t| t == DiffLineType::NewOnly)
            .count()
    }

    /// Get the number of deleted lines in this hunk
    pub fn deleted_lines(&self) -> usize {
        self.line_types
            .iter()
            .filter(|&&t| t == DiffLineType::OldOnly)
            .count()
    }

    /// Get the number of unchanged lines in this hunk
    pub fn unchanged_lines(&self) -> usize {
        self.line_types
            .iter()
            .filter(|&&t| t == DiffLineType::Both)
            .count()
    }

    /// Set the line type at the given index
    pub fn set_line_type(&mut self, index: usize, line_type: DiffLineType) {
        if index < self.line_types.len() {
            self.line_types[index] = line_type;
        }
    }

    /// Get the line type at the given index
    pub fn line_type(&self, index: usize) -> Option<DiffLineType> {
        self.line_types.get(index).copied()
    }

    /// Set the secondary status of the hunk
    pub fn set_secondary_status(&mut self, status: DiffHunkSecondaryStatus) {
        self.secondary_status = status;
    }
}
