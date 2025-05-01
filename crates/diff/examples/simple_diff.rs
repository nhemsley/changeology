use anyhow::Result;
use diff::{DiffLineType, TextDiff};

fn main() -> Result<()> {
    // Two sample texts to compare
    let text1 = "This is the first line.\nHere is the second line.\nAnd the third line.";
    let text2 = "This is the first line.\nThis is a completely different second line.\nAnd the third line.\nPlus a new fourth line.";

    // Generate a unified diff
    println!("Unified diff:");
    println!("{}", TextDiff::unified_diff(text1, text2, 1));

    // Generate a diff object
    let diff = TextDiff::diff(text1, text2)?;
    let snapshot = diff.snapshot();

    // Print diff statistics
    println!("\nDiff statistics:");
    println!("  Total hunks: {}", snapshot.hunk_count());
    println!("  Added lines: {}", snapshot.added_lines());
    println!("  Deleted lines: {}", snapshot.deleted_lines());
    println!("  Unchanged lines: {}", snapshot.unchanged_lines());

    // Print hunks with color-coded line types
    println!("\nHunks with line types:");
    for (i, hunk) in snapshot.hunks().iter().enumerate() {
        println!("Hunk {}:", i + 1);
        println!("  Status: {}", hunk.status);
        println!(
            "  Old range: {}:{}",
            hunk.old_range.start, hunk.old_range.count
        );
        println!(
            "  New range: {}:{}",
            hunk.new_range.start, hunk.new_range.count
        );

        // Print line types for this hunk
        println!("  Line types:");
        for (j, &line_type) in hunk.line_types.iter().enumerate() {
            match line_type {
                DiffLineType::OldOnly => println!("    Line {}: \x1b[31mDeleted\x1b[0m", j),
                DiffLineType::NewOnly => println!("    Line {}: \x1b[32mAdded\x1b[0m", j),
                DiffLineType::Both => println!("    Line {}: \x1b[37mUnchanged\x1b[0m", j),
            }
        }
    }

    // Show example of comparing different file versions
    println!("\nExample of comparing different versions of a file:");
    let file1 = r#"fn main() {
    println!("Hello, world!");
}
"#;

    let file2 = r#"fn main() {
    // Add a greeting with name
    let name = "Rust";
    println!("Hello, {}!", name);
}
"#;

    let diff2 = TextDiff::diff(file1, file2)?;
    let snapshot2 = diff2.snapshot();

    println!("Unified diff:");
    println!("{}", TextDiff::unified_diff(file1, file2, 1));

    println!("\nDiff statistics:");
    println!("  Total hunks: {}", snapshot2.hunk_count());
    println!("  Added lines: {}", snapshot2.added_lines());
    println!("  Deleted lines: {}", snapshot2.deleted_lines());
    println!("  Unchanged lines: {}", snapshot2.unchanged_lines());

    Ok(())
}
