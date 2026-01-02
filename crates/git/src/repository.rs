use anyhow::{anyhow, Context, Result};
use git2::{Diff, DiffOptions, Repository as Git2Repository, Sort};
use std::path::{Path, PathBuf};

use crate::status::{StatusEntry, StatusKind, StatusList};

/// Represents a git commit
#[derive(Debug, Clone)]
pub struct Commit {
    /// The commit's SHA-1 hash
    pub id: String,
    /// The commit's short hash (first 7 characters)
    pub short_id: String,
    /// The commit message
    pub message: String,
    /// The commit author name
    pub author_name: String,
    /// The commit author email
    pub author_email: String,
    /// The commit timestamp (seconds since epoch)
    pub time: i64,
    /// Parent commit IDs
    pub parent_ids: Vec<String>,
}

/// A wrapper around git2::Repository with additional functionality
pub struct Repository {
    /// The underlying git2 repository
    inner: Git2Repository,
    /// The repository's working directory
    work_dir: PathBuf,
}

impl Repository {
    /// Open a git repository at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let repo = Git2Repository::discover(path)
            .with_context(|| format!("Failed to discover git repository at {}", path.display()))?;

        let work_dir = repo
            .workdir()
            .ok_or_else(|| anyhow!("Repository has no working directory"))?
            .to_path_buf();

        Ok(Self {
            inner: repo,
            work_dir,
        })
    }

    /// Get the repository's working directory
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// Get the status of the repository
    pub fn status(&self) -> Result<StatusList> {
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let status = self.inner.statuses(Some(&mut opts))?;

        let mut entries = Vec::new();

        for entry in status.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            entries.push(StatusEntry {
                path,
                kind: StatusKind::from_git2_status(status),
            });
        }

        Ok(StatusList { entries })
    }

    /// Get the content of a file from the repository HEAD
    pub fn get_head_content(&self, path: &str) -> Result<Option<String>> {
        self.get_content_at_revision("HEAD", path)
    }

    /// Get the content of a file at a specific commit/revision
    pub fn get_content_at_revision(&self, revision: &str, path: &str) -> Result<Option<String>> {
        let obj = match self.inner.revparse_single(revision) {
            Ok(obj) => obj,
            Err(_) => return Ok(None),
        };

        let commit = obj.peel_to_commit()?;
        let tree = commit.tree()?;

        let entry = match tree.get_path(Path::new(path)) {
            Ok(entry) => entry,
            Err(_) => return Ok(None),
        };

        let blob = entry.to_object(&self.inner)?.peel_to_blob()?;
        let content = String::from_utf8_lossy(blob.content()).to_string();

        Ok(Some(content))
    }

    /// Get the content of a file from the working directory
    pub fn get_working_content(&self, path: &str) -> Result<Option<String>> {
        let full_path = self.work_dir.join(path);
        if !full_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file {}", full_path.display()))?;

        Ok(Some(content))
    }

    /// Get the content of a file from the index (staging area)
    pub fn get_index_content(&self, path: &str) -> Result<Option<String>> {
        let index = self.inner.index()?;

        let id = match index.get_path(Path::new(path), 0) {
            Some(entry) => entry.id,
            None => return Ok(None),
        };

        let blob = self.inner.find_blob(id)?;
        let content = String::from_utf8_lossy(blob.content()).to_string();

        Ok(Some(content))
    }

    /// Get the diff between two versions of a file
    pub fn diff_file(&self, path: &str, old_version: &str, new_version: &str) -> Result<Diff<'_>> {
        let old_oid = self.inner.revparse_single(old_version)?.id();
        let new_oid = self.inner.revparse_single(new_version)?.id();

        let old_tree = self
            .inner
            .find_tree(self.inner.find_commit(old_oid)?.tree_id())?;
        let new_tree = self
            .inner
            .find_tree(self.inner.find_commit(new_oid)?.tree_id())?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(path);

        let diff =
            self.inner
                .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut diff_opts))?;

        Ok(diff)
    }

    /// Get the diff between the index and the working directory for a file
    pub fn diff_index_to_workdir(&self, path: &str) -> Result<Diff<'_>> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(path);

        let diff = self
            .inner
            .diff_index_to_workdir(None, Some(&mut diff_opts))?;

        Ok(diff)
    }

    /// Get the diff between HEAD and the index for a file
    pub fn diff_head_to_index(&self, path: &str) -> Result<Diff<'_>> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(path);

        // Get HEAD commit and its tree
        let head_obj = self.inner.revparse_single("HEAD")?;
        let head_commit = head_obj.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let diff = self
            .inner
            .diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;

        Ok(diff)
    }

    /// Get the commit history, optionally limited to a maximum count
    pub fn log(&self, max_count: Option<usize>) -> Result<Vec<Commit>> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
        revwalk.push_head()?;

        let mut commits = Vec::new();
        let limit = max_count.unwrap_or(usize::MAX);

        for (i, oid_result) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid_result?;
            let commit = self.inner.find_commit(oid)?;

            let message = commit
                .message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string();

            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown").to_string();
            let author_email = author.email().unwrap_or("").to_string();

            let parent_ids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();

            commits.push(Commit {
                id: oid.to_string(),
                short_id: format!("{:.7}", oid),
                message,
                author_name,
                author_email,
                time: commit.time().seconds(),
                parent_ids,
            });
        }

        Ok(commits)
    }

    /// Get a specific commit by its ID (can be short or full hash)
    pub fn get_commit(&self, id: &str) -> Result<Commit> {
        let obj = self.inner.revparse_single(id)?;
        let commit = obj.peel_to_commit()?;
        let oid = commit.id();

        let message = commit
            .message()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();

        let parent_ids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();

        Ok(Commit {
            id: oid.to_string(),
            short_id: format!("{:.7}", oid),
            message,
            author_name,
            author_email,
            time: commit.time().seconds(),
            parent_ids,
        })
    }

    /// Get the files changed in a commit
    pub fn get_commit_files(&self, commit_id: &str) -> Result<Vec<String>> {
        let obj = self.inner.revparse_single(commit_id)?;
        let commit = obj.peel_to_commit()?;
        let commit_tree = commit.tree()?;

        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        let diff = self.inner.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            Some(&mut diff_opts),
        )?;

        let mut files = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_string_lossy().to_string());
                }
                true
            },
            None,
            None,
            None,
        )?;

        Ok(files)
    }
}
