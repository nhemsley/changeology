use buffer_diff::{DiffConfig, DiffGranularity};
use similar::Algorithm;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: enhanced_diff <old_file> <new_file> [options]");
        println!("Options:");
        println!("  --word        Perform word-level diffing (default is line-level)");
        println!("  --char        Perform character-level diffing");
        println!("  --patience    Use the patience algorithm (default is Myers)");
        println!("  --ignore-ws   Ignore whitespace changes");
        println!("  --context N   Show N lines of context (default is 3)");
        return Ok(());
    }
    
    let old_file = &args[1];
    let new_file = &args[2];
    
    // Read the files
    let old_content = read_file(old_file)?;
    let new_content = read_file(new_file)?;
    
    // Parse options
    let mut granularity = DiffGranularity::Line;
    let mut algorithm = Algorithm::Myers;
    let mut ignore_whitespace = false;
    let mut context_lines = 3;
    
    for arg in &args[3..] {
        match arg.as_str() {
            "--word" => granularity = DiffGranularity::Word,
            "--char" => granularity = DiffGranularity::Character,
            "--patience" => algorithm = Algorithm::Patience,
            "--ignore-ws" => ignore_whitespace = true,
            _ => {
                if arg.starts_with("--context=") {
                    if let Some(val) = arg.strip_prefix("--context=") {
                        if let Ok(n) = val.parse::<usize>() {
                            context_lines = n;
                        }
                    }
                }
            }
        }
    }
    
    // Configure the diff
    let config = DiffConfig::default()
        .algorithm(algorithm)
        .granularity(granularity)
        .ignore_whitespace(ignore_whitespace)
        .context_lines(context_lines);
    
    // Generate the diff
    let diff_result = config.diff(&old_content, &new_content).unwrap();
    let snapshot = diff_result.snapshot();
    
    // Print diff information
    println!("Diff between {} and {}", old_file, new_file);
    println!("Configuration:");
    println!("  Granularity: {:?}", granularity);
    println!("  Algorithm: {:?}", algorithm);
    println!("  Ignore whitespace: {}", ignore_whitespace);
    println!("  Context lines: {}", context_lines);
    println!();
    
    // Print statistics
    println!("Statistics:");
    println!("  Hunks: {}", snapshot.hunk_count());
    println!("  Added lines: {}", snapshot.added_lines());
    println!("  Deleted lines: {}", snapshot.deleted_lines());
    println!("  Unchanged lines: {}", snapshot.unchanged_lines());
    println!();
    
    // Print unified diff
    println!("Unified diff:");
    println!("{}", config.unified_diff(&old_content, &new_content));
    
    Ok(())
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}