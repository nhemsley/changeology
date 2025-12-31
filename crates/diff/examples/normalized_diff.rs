use anyhow::Result;
use buffer_diff::{LineEndingMode, TextDiff};
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: normalized_diff <old_file> <new_file> [--mode=<mode>]");
        println!("Available modes: auto, unix, windows, macos, preserve");
        return Ok(());
    }
    
    let old_file = &args[1];
    let new_file = &args[2];
    
    // Parse mode option
    let mut mode = LineEndingMode::Auto;
    if args.len() > 3 {
        let mode_arg = &args[3];
        if mode_arg.starts_with("--mode=") {
            let mode_str = mode_arg.strip_prefix("--mode=").unwrap();
            mode = match mode_str {
                "unix" => LineEndingMode::Unix,
                "windows" => LineEndingMode::Windows,
                "macos" => LineEndingMode::MacOS,
                "preserve" => LineEndingMode::Preserve,
                _ => LineEndingMode::Auto,
            };
        }
    }
    
    // Read and compare files
    println!("Comparing {} and {} with line ending mode: {:?}", old_file, new_file, mode);
    
    // Read the file contents
    let old_content = std::fs::read_to_string(Path::new(old_file))?;
    let new_content = std::fs::read_to_string(Path::new(new_file))?;
    
    // Configure the diff with line ending mode and run the diff
    let diff = TextDiff::configure()
        .line_ending_mode(mode)
        .context_lines(3)
        .unified_diff(&old_content, &new_content);
    
    println!("\nUnified diff with normalized line endings:");
    println!("{}", diff);
    
    Ok(())
}