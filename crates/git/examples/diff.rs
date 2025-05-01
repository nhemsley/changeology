use anyhow::Result;
use git::Repository;
use git2::DiffFormat;
use std::env;

fn main() -> Result<()> {
    // Get repository path and file path from command line
    let repo_path = env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let file_path = env::args().nth(2).expect("File path required");

    // Open the repository
    let repo = Repository::open(&repo_path)?;
    println!("Opened repository at: {}", repo.work_dir().display());

    // Get status for the file
    let status = repo.status()?;
    let file_statuses = status.get_file_status(&file_path);

    if file_statuses.is_empty() {
        println!(
            "\nFile '{}' has no changes or doesn't exist in git",
            file_path
        );
        return Ok(());
    }

    println!("\nFile status: {:?}", file_statuses);

    // Get contents based on file status
    let is_new_file = file_statuses
        .iter()
        .any(|s| s.kind == git::StatusKind::Untracked);
    let head_content = if !is_new_file {
        repo.get_head_content(&file_path)?
    } else {
        None
    };
    let working_content = repo.get_working_content(&file_path)?;

    // Print contents if available
    match (head_content, working_content) {
        (Some(head), Some(working)) => {
            println!("\nFile exists in HEAD and working directory");

            if head == working {
                println!("Content is identical");
            } else {
                println!("\nHEAD content:");
                println!("{}\n", head);
                println!("Working directory content:");
                println!("{}", working);
            }
        }
        (None, Some(content)) => {
            println!("\nFile is new (not in HEAD)");
            println!("Working directory content:");
            println!("{}", content);
        }
        (Some(_), None) => {
            println!("\nFile has been deleted from working directory");
        }
        (None, None) => {
            println!("\nFile doesn't exist in HEAD or working directory");
        }
    }

    // Try using git2's diff functionality for both staged and unstaged changes
    println!("\nUnstaged changes (index to working directory):");
    let diff_unstaged = repo.diff_index_to_workdir(&file_path)?;
    if diff_unstaged.deltas().len() == 0 {
        println!("No unstaged changes");
    } else {
        diff_unstaged.print(DiffFormat::Patch, |_, _, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("[Invalid UTF-8]");
            match line.origin() {
                '+' => print!("\x1b[32m{}\x1b[0m", content),
                '-' => print!("\x1b[31m{}\x1b[0m", content),
                _ => print!("{}", content),
            }
            true
        })?;
    }

    // Only try to get staged changes if the file isn't new/untracked
    if !is_new_file {
        println!("\nStaged changes (HEAD to index):");
        match repo.diff_head_to_index(&file_path) {
            Ok(diff_staged) => {
                if diff_staged.deltas().len() == 0 {
                    println!("No staged changes");
                } else {
                    diff_staged.print(DiffFormat::Patch, |_, _, line| {
                        let content =
                            std::str::from_utf8(line.content()).unwrap_or("[Invalid UTF-8]");
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
    } else {
        println!("\nNo staged changes (file is untracked)");
    }

    Ok(())
}
