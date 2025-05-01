# Git Crate

A wrapper around `git2` that provides a simpler interface for working with Git repositories.

## Features

- Simple wrapper around `git2::Repository`
- Get repository status information
- Retrieve file contents from different versions (HEAD, index, working directory)
- Generate diffs between different versions of files
- Filter status entries by type (added, modified, deleted, etc.)

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
git = { path = "../git" }
```

### Examples

#### Getting Repository Status

```rust
use git::Repository;
use std::env;

fn main() -> anyhow::Result<()> {
    // Open the repository at the given path (or current directory)
    let path = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let repo = Repository::open(&path)?;
    println!("Opened repository at: {}", repo.work_dir().display());
    
    // Get repository status
    let status = repo.status()?;
    
    // Print all entries
    println!("\nAll status entries:");
    for entry in &status.entries {
        println!("{}: {}", entry.kind, entry.path);
    }
    
    // Print categorized entries
    println!("\nAdded files:");
    for entry in status.added() {
        println!("  {}", entry.path);
    }
    
    println!("\nModified files:");
    for entry in status.modified() {
        println!("  {}", entry.path);
    }
    
    println!("\nDeleted files:");
    for entry in status.deleted() {
        println!("  {}", entry.path);
    }
    
    println!("\nUntracked files:");
    for entry in status.untracked() {
        println!("  {}", entry.path);
    }
    
    Ok(())
}
```

#### Getting File Diffs

```rust
use git::{Repository, DiffFormat};
use std::env;

fn main() -> anyhow::Result<()> {
    // Open repository and get file path from command line
    let repo_path = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let file_path = env::args().nth(2).expect("File path required");
    
    // Open the repository
    let repo = Repository::open(&repo_path)?;
    
    // Get status for the file
    let status = repo.status()?;
    let file_statuses = status.get_file_status(&file_path);
    
    if file_statuses.is_empty() {
        println!("\nFile '{}' has no changes or doesn't exist in git", file_path);
        return Ok(());
    }
    
    // Get contents based on file status
    let is_new_file = file_statuses
        .iter()
        .any(|s| s.kind == git::StatusKind::Untracked);
    
    // Get unstaged changes
    println!("\nUnstaged changes (index to working directory):");
    let diff_unstaged = repo.diff_index_to_workdir(&file_path)?;
    
    if diff_unstaged.deltas_count() == 0 {
        println!("No unstaged changes");
    } else {
        diff_unstaged.print(DiffFormat::Patch, |_, _, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("[Invalid UTF-8]");
            match line.origin() {
                '+' => print!("\x1b[32m{}\x1b[0m", content), // Green for additions
                '-' => print!("\x1b[31m{}\x1b[0m", content), // Red for deletions
                _ => print!("{}", content),
            }
            true
        })?;
    }
    
    // If not a new file, get staged changes too
    if !is_new_file {
        println!("\nStaged changes (HEAD to index):");
        match repo.diff_head_to_index(&file_path) {
            Ok(diff_staged) => {
                if diff_staged.deltas_count() == 0 {
                    println!("No staged changes");
                } else {
                    // Print colored diff
                    diff_staged.print(DiffFormat::Patch, |_, _, line| {
                        let content = std::str::from_utf8(line.content()).unwrap_or("[Invalid UTF-8]");
                        match line.origin() {
                            '+' => print!("\x1b[32m{}\x1b[0m", content),
                            '-' => print!("\x1b[31m{}\x1b[0m", content),
                            _ => print!("{}", content),
                        }
                        true
                    })?;
                }
            }
            Err(e) => {
                println!("Could not get staged changes: {}", e);
            }
        }
    }
    
    Ok(())
}
```

## API Reference

### Repository

The main struct for interacting with a Git repository.

```rust
pub struct Repository {
    inner: Git2Repository,
    work_dir: PathBuf,
}
```

Key methods:
- `open(path: &str) -> Result<Repository>`: Open a repository at the given path
- `status() -> Result<StatusList>`: Get the current status of the repository
- `get_head_content(path: &str) -> Result<Option<String>>`: Get file content from HEAD
- `get_index_content(path: &str) -> Result<Option<String>>`: Get file content from the index
- `get_working_content(path: &str) -> Result<Option<String>>`: Get file content from the working directory
- `diff_head_to_index(path: &str) -> Result<Diff>`: Get diff between HEAD and index
- `diff_index_to_workdir(path: &str) -> Result<Diff>`: Get diff between index and working directory

### StatusList

Represents the status of a Git repository.

```rust
pub struct StatusList {
    pub entries: Vec<StatusEntry>,
}
```

Key methods:
- `added() -> Vec<&StatusEntry>`: Get all added files
- `modified() -> Vec<&StatusEntry>`: Get all modified files
- `deleted() -> Vec<&StatusEntry>`: Get all deleted files
- `untracked() -> Vec<&StatusEntry>`: Get all untracked files
- `get_file_status(path: &str) -> Vec<&StatusEntry>`: Get status entries for a specific file

### StatusEntry

Represents a single status entry in a Git repository.

```rust
pub struct StatusEntry {
    pub path: String,
    pub kind: StatusKind,
}
```

### StatusKind

Enum representing the kind of status.

```rust
pub enum StatusKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
    Unknown,
}
```

## Running the Examples

```
cargo run --example status -- [path_to_repo]
cargo run --example diff -- [path_to_repo] [file_path]
```