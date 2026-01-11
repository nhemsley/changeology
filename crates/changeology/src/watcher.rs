//! Simple file system watcher for repository changes
//!
//! Watches the repository directory and notifies when files change.

use log::{debug, info, trace, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Identifies different data sources in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataSourceKind {
    /// Unstaged/dirty files in the working directory
    DirtyFiles,
    /// Staged files (git index)
    StagedFiles,
    /// Commit history
    History,
    /// All data sources
    All,
}

/// A simple file watcher that monitors a directory for changes
pub struct RepoWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
}

impl RepoWatcher {
    /// Create a new watcher for the given repository path
    pub fn new(repo_path: &Path) -> anyhow::Result<Self> {
        info!("Creating RepoWatcher for: {:?}", repo_path);
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        // Watch the .git directory for index/ref changes
        let git_dir = repo_path.join(".git");
        if git_dir.exists() {
            info!("Watching .git directory: {:?}", git_dir);
            watcher.watch(&git_dir, RecursiveMode::Recursive)?;
        } else {
            warn!("No .git directory found at: {:?}", git_dir);
        }

        // Watch the working directory for file changes (non-recursive to avoid .git)
        info!("Watching working directory: {:?}", repo_path);
        watcher.watch(repo_path, RecursiveMode::NonRecursive)?;

        info!("RepoWatcher initialized successfully");
        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Check for pending changes and return which data sources need refreshing
    pub fn poll_changes(&self) -> Option<DataSourceKind> {
        let mut result: Option<DataSourceKind> = None;

        // Drain all pending events
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Ok(event) => {
                    // Filter out Access events - we only care about actual changes
                    if matches!(event.kind, EventKind::Access(_)) {
                        trace!("Ignoring access event: {:?}", event);
                        continue;
                    }

                    trace!("Received fs event: {:?}", event);
                    // Log the paths that triggered the event
                    for path in &event.paths {
                        debug!("File event {:?}: {}", event.kind, path.display());
                    }
                    let kind = Self::classify_event(&event);
                    debug!("Classified event as: {:?}", kind);
                    result = Some(Self::merge_kinds(result, kind));
                }
                Err(e) => {
                    warn!("File watcher error: {:?}", e);
                }
            }
        }

        if let Some(ref kind) = result {
            info!("poll_changes returning: {:?}", kind);
        }

        result
    }

    /// Classify a file system event into which data source it affects
    fn classify_event(event: &Event) -> DataSourceKind {
        for path in &event.paths {
            let path_str = path.to_string_lossy();
            trace!("Classifying path: {}", path_str);

            // .git/index changes -> staged files
            if path_str.contains(".git/index") {
                return DataSourceKind::StagedFiles;
            }

            // .git/refs or .git/HEAD changes -> history
            if path_str.contains(".git/refs")
                || path_str.contains(".git/HEAD")
                || path_str.contains(".git/logs")
            {
                return DataSourceKind::History;
            }

            // Other .git changes -> could be anything
            if path_str.contains(".git") {
                return DataSourceKind::All;
            }
        }

        // Working directory changes -> dirty files
        DataSourceKind::DirtyFiles
    }

    /// Merge two data source kinds, preferring All if there's a conflict
    fn merge_kinds(current: Option<DataSourceKind>, new: DataSourceKind) -> DataSourceKind {
        match current {
            None => new,
            Some(DataSourceKind::All) => DataSourceKind::All,
            Some(current_kind) if current_kind == new => current_kind,
            Some(_) => DataSourceKind::All,
        }
    }
}
