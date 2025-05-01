use derive_more::{Display, From};
use git2::Status as Git2Status;

/// Represents the status of a file in a git repository
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatus {
    /// Path to the file, relative to the repository root
    pub path: String,
    /// Whether the file is staged
    pub is_staged: bool,
    /// Whether the file is unstaged
    pub is_unstaged: bool,
    /// Whether the file is untracked
    pub is_untracked: bool,
    /// Whether the file is ignored
    pub is_ignored: bool,
    /// Whether the file is conflicted
    pub is_conflicted: bool,
}

/// Represents the status kind of a file in more detail
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, From)]
pub enum StatusKind {
    /// The file is new in the index
    #[display(fmt = "Added")]
    Added,
    /// The file has been modified in the index
    #[display(fmt = "Modified")]
    Modified,
    /// The file has been deleted in the index
    #[display(fmt = "Deleted")]
    Deleted,
    /// The file has been renamed in the index
    #[display(fmt = "Renamed")]
    Renamed,
    /// The file has been copied in the index
    #[display(fmt = "Copied")]
    Copied,
    /// The file is untracked in the working directory
    #[display(fmt = "Untracked")]
    Untracked,
    /// The file is ignored in the working directory
    #[display(fmt = "Ignored")]
    Ignored,
    /// The file is in a conflicted state
    #[display(fmt = "Conflicted")]
    Conflicted,
    /// The status is unknown
    #[display(fmt = "Unknown")]
    Unknown,
}

impl StatusKind {
    /// Convert from git2::Status to StatusKind
    pub fn from_git2_status(status: Git2Status) -> Self {
        if status.is_index_new() {
            return StatusKind::Added;
        }
        if status.is_index_modified() {
            return StatusKind::Modified;
        }
        if status.is_index_deleted() {
            return StatusKind::Deleted;
        }
        if status.is_index_renamed() {
            return StatusKind::Renamed;
        }
        if status.is_index_typechange() {
            return StatusKind::Modified;
        }
        if status.is_wt_new() {
            return StatusKind::Untracked;
        }
        if status.is_wt_modified() {
            return StatusKind::Modified;
        }
        if status.is_wt_deleted() {
            return StatusKind::Deleted;
        }
        if status.is_wt_renamed() {
            return StatusKind::Renamed;
        }
        if status.is_wt_typechange() {
            return StatusKind::Modified;
        }
        if status.is_ignored() {
            return StatusKind::Ignored;
        }
        if status.is_conflicted() {
            return StatusKind::Conflicted;
        }
        
        StatusKind::Unknown
    }
}

/// Entry in a status list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// Path to the file, relative to the repository root
    pub path: String,
    /// The status kind of the file
    pub kind: StatusKind,
}

/// List of status entries for a repository
#[derive(Debug, Clone)]
pub struct StatusList {
    /// The list of status entries
    pub entries: Vec<StatusEntry>,
}

impl StatusList {
    /// Get a filtered list of entries that match the given predicate
    pub fn filter<F>(&self, predicate: F) -> Vec<&StatusEntry>
    where
        F: Fn(&StatusEntry) -> bool,
    {
        self.entries.iter().filter(|e| predicate(e)).collect()
    }
    
    /// Get all status entries for a specific file path
    pub fn get_file_status(&self, path: &str) -> Vec<&StatusEntry> {
        self.filter(|e| e.path == path)
    }
    
    /// Get all added files
    pub fn added(&self) -> Vec<&StatusEntry> {
        self.filter(|e| e.kind == StatusKind::Added)
    }
    
    /// Get all modified files
    pub fn modified(&self) -> Vec<&StatusEntry> {
        self.filter(|e| e.kind == StatusKind::Modified)
    }
    
    /// Get all deleted files
    pub fn deleted(&self) -> Vec<&StatusEntry> {
        self.filter(|e| e.kind == StatusKind::Deleted)
    }
    
    /// Get all renamed files
    pub fn renamed(&self) -> Vec<&StatusEntry> {
        self.filter(|e| e.kind == StatusKind::Renamed)
    }
    
    /// Get all untracked files
    pub fn untracked(&self) -> Vec<&StatusEntry> {
        self.filter(|e| e.kind == StatusKind::Untracked)
    }
}