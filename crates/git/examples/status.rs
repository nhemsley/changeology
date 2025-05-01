use anyhow::Result;
use git::Repository;
use std::env;

fn main() -> Result<()> {
    // Use current directory if no path provided
    let path = env::args().nth(1).unwrap_or_else(|| ".".to_string());

    // Open the repository
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
