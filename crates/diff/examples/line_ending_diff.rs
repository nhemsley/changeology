use anyhow::Result;
use diff::{DiffConfig, LineEndingMode, TextDiff};
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

fn main() -> Result<()> {
    // Example text with mixed line endings
    let unix_text = "Line 1\nLine 2\nLine 3\n";
    let windows_text = "Line 1\r\nLine 2\r\nLine 3\r\n";
    let mac_text = "Line 1\rLine 2\rLine 3\r";
    let mixed_text = "Line 1\nLine 2\r\nLine 3\r";

    println!("=== Demonstrating line ending handling ===\n");
    
    // Example 1: Mixed line endings with Preserve mode
    println!("1. Diffing mixed line endings with Preserve mode:");
    let preserve_diff = TextDiff::configure()
        .line_ending_mode(LineEndingMode::Preserve)
        .unified_diff(unix_text, mixed_text);
    println!("{}", preserve_diff);
    
    // Example 2: Mixed line endings with Unix normalization
    println!("\n2. Diffing mixed line endings with Unix normalization:");
    let unix_diff = TextDiff::configure()
        .line_ending_mode(LineEndingMode::Unix)
        .unified_diff(unix_text, mixed_text);
    println!("{}", unix_diff);
    
    // Example 3: Auto-detection of line endings
    println!("\n3. Auto-detection with predominantly Windows line endings:");
    let windows_mixed = "Line 1\r\nLine 2\r\nLine 3\r\nLine 4\nLine 5\r";
    let auto_diff = TextDiff::configure()
        .line_ending_mode(LineEndingMode::Auto)
        .unified_diff(windows_text, windows_mixed);
    println!("{}", auto_diff);
    
    // Example 4: Different files with fundamentally different content
    println!("\n4. Diffing different content with normalized line endings:");
    let content1 = "First line\r\nSecond line\r\nThird line";
    let content2 = "First line\nModified second\nThird line\nAdded fourth";
    
    println!("Original diff (with mixed line endings):");
    let original_diff = TextDiff::configure()
        .line_ending_mode(LineEndingMode::Preserve)
        .unified_diff(content1, content2);
    println!("{}", original_diff);
    
    println!("\nNormalized diff (unix line endings):");
    let normalized_diff = TextDiff::configure()
        .line_ending_mode(LineEndingMode::Unix)
        .unified_diff(content1, content2);
    println!("{}", normalized_diff);

    // Create demo files with different line endings
    if env::args().len() > 1 && env::args().nth(1).unwrap() == "--create-files" {
        println!("\nCreating demo files with different line endings...");
        create_demo_files()?;
    }

    Ok(())
}

/// Create example files with different line endings for testing
fn create_demo_files() -> Result<()> {
    // Create directory if it doesn't exist
    let dir_path = Path::new("test_files");
    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path)?;
    }
    
    // Unix-style line endings (LF)
    let unix_path = dir_path.join("unix_endings.txt");
    write_file(
        &unix_path,
        "This file uses Unix-style line endings (LF: \\n).\n\
         These are common on Linux and macOS systems.\n\
         Many text editors automatically handle these.\n"
    )?;
    
    // Windows-style line endings (CRLF)
    let windows_path = dir_path.join("windows_endings.txt");
    write_file(
        &windows_path,
        "This file uses Windows-style line endings (CRLF: \\r\\n).\r\n\
         These are common on Windows operating systems.\r\n\
         Text editors typically show just a newline character.\r\n"
    )?;
    
    // Classic Mac OS line endings (CR)
    let mac_path = dir_path.join("mac_endings.txt");
    write_file(
        &mac_path,
        "This file uses Classic Mac OS line endings (CR: \\r).\r\
         These were used in Mac OS 9 and earlier.\r\
         Modern macOS uses Unix-style line endings.\r"
    )?;
    
    // Mixed line endings
    let mixed_path = dir_path.join("mixed_endings.txt");
    write_file(
        &mixed_path,
        "This file uses Unix-style line endings (LF: \\n).\n\
         This line uses Windows-style line endings (CRLF: \\r\\n).\r\n\
         This line uses Classic Mac OS line endings (CR: \\r).\r\
         Back to Unix-style for this line.\n"
    )?;
    
    println!("Demo files created in the test_files directory:");
    println!("  - {}", unix_path.display());
    println!("  - {}", windows_path.display());
    println!("  - {}", mac_path.display());
    println!("  - {}", mixed_path.display());
    
    Ok(())
}

/// Write content to a file, ensuring raw bytes are preserved
fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}