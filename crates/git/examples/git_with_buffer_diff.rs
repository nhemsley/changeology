use anyhow::Result;
use buffer_diff::TextDiff;
use git::Repository;
use std::env;

fn main() -> Result<()> {
    println!("=== Git + Buffer-Diff Integration Example ===\n");

    // Get repository path and file path from command line
    let repo_path = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let file_path = env::args()
        .nth(2)
        .expect("Usage: cargo run --example git_with_buffer_diff <repo_path> <file_path>");

    // Open the repository using the git crate
    let repo = Repository::open(&repo_path)?;
    println!("Repository: {}", repo.work_dir().display());
    println!("File: {}\n", file_path);

    // Get file status
    let status = repo.status()?;
    let file_statuses = status.get_file_status(&file_path);

    if file_statuses.is_empty() {
        println!("No changes detected for this file");
        return Ok(());
    }

    println!("File status: {:?}\n", file_statuses);

    // Get file contents from different versions
    let head_content = repo.get_head_content(&file_path)?;
    let index_content = repo.get_index_content(&file_path)?;
    let working_content = repo.get_working_content(&file_path)?;

    // Demo 1: Compare HEAD vs Working Directory
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("1. HEAD vs Working Directory (unstaged changes)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if let (Some(head), Some(working)) = (&head_content, &working_content) {
        // Use buffer-diff to calculate the diff
        let diff = TextDiff::diff(head, working)?;
        let snapshot = diff.snapshot();

        println!("Statistics:");
        println!("  Hunks: {}", snapshot.hunk_count());
        println!("  Added lines: {}", snapshot.added_lines());
        println!("  Deleted lines: {}", snapshot.deleted_lines());
        println!("  Unchanged lines: {}", snapshot.unchanged_lines());

        if snapshot.has_changes() {
            println!("\nUnified Diff:");
            println!("{}", TextDiff::unified_diff(head, working, 3));
        } else {
            println!("\nNo changes detected");
        }
    } else {
        println!("Cannot compare - file may be new or deleted");
    }

    // Demo 2: Compare HEAD vs Index (staged changes)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("2. HEAD vs Index (staged changes)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if let (Some(head), Some(index)) = (&head_content, &index_content) {
        let diff = TextDiff::diff(head, index)?;
        let snapshot = diff.snapshot();

        println!("Statistics:");
        println!("  Hunks: {}", snapshot.hunk_count());
        println!("  Added lines: {}", snapshot.added_lines());
        println!("  Deleted lines: {}", snapshot.deleted_lines());
        println!("  Unchanged lines: {}", snapshot.unchanged_lines());

        if snapshot.has_changes() {
            println!("\nUnified Diff:");
            println!("{}", TextDiff::unified_diff(head, index, 3));
        } else {
            println!("\nNo staged changes");
        }
    } else {
        println!("Cannot compare - file may be new or not staged");
    }

    // Demo 3: Compare Index vs Working Directory
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("3. Index vs Working Directory (unstaged changes after staging)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if let (Some(index), Some(working)) = (&index_content, &working_content) {
        let diff = TextDiff::diff(index, working)?;
        let snapshot = diff.snapshot();

        println!("Statistics:");
        println!("  Hunks: {}", snapshot.hunk_count());
        println!("  Added lines: {}", snapshot.added_lines());
        println!("  Deleted lines: {}", snapshot.deleted_lines());
        println!("  Unchanged lines: {}", snapshot.unchanged_lines());

        if snapshot.has_changes() {
            println!("\nUnified Diff:");
            println!("{}", TextDiff::unified_diff(index, working, 3));
        } else {
            println!("\nNo unstaged changes (working directory matches index)");
        }
    } else {
        println!("Cannot compare");
    }

    // Demo 4: Show detailed hunk information
    if let (Some(head), Some(working)) = (&head_content, &working_content) {
        let diff = TextDiff::diff(head, working)?;
        let snapshot = diff.snapshot();

        if snapshot.has_changes() {
            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("4. Detailed Hunk Information");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

            for (i, hunk) in snapshot.hunks().iter().enumerate() {
                println!("Hunk #{}: {}", i + 1, hunk.status);
                println!(
                    "  Old: lines {}-{} ({} lines)",
                    hunk.old_range.start,
                    hunk.old_range.end(),
                    hunk.old_range.count
                );
                println!(
                    "  New: lines {}-{} ({} lines)",
                    hunk.new_range.start,
                    hunk.new_range.end(),
                    hunk.new_range.count
                );
                println!(
                    "  Changes: +{} -{} ={} lines",
                    hunk.added_lines(),
                    hunk.deleted_lines(),
                    hunk.unchanged_lines()
                );
                println!();
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nThe git crate provides:");
    println!("  ✓ File content from HEAD, index, and working directory");
    println!("  ✓ Git status information");
    println!("  ✓ Repository operations");
    println!("\nThe buffer-diff crate provides:");
    println!("  ✓ Detailed diff calculation with hunks");
    println!("  ✓ Line-level change tracking");
    println!("  ✓ Unified diff formatting");
    println!("  ✓ Statistics and analysis");
    println!("\nTogether they enable:");
    println!("  ✓ Complete Git workflow analysis");
    println!("  ✓ Detailed change visualization");
    println!("  ✓ Flexible diff processing");

    Ok(())
}
