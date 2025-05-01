use diff::BufferDiff;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: chunked_diff <old_file> <new_file> [output_file]");
        println!("Demonstrates chunking and parallel processing for large file diffs");
        return Ok(());
    }
    
    let old_file = &args[1];
    let new_file = &args[2];
    let output_file = args.get(3);
    
    println!("Reading files...");
    let old_content = read_file(old_file)?;
    let new_content = read_file(new_file)?;
    
    println!("File sizes:");
    println!("  Old: {} bytes, {} lines", old_content.len(), count_lines(&old_content));
    println!("  New: {} bytes, {} lines", new_content.len(), count_lines(&new_content));
    
    println!("Computing diff with chunking and parallelization...");
    let start = Instant::now();
    let buffer_diff = BufferDiff::new(&old_content, &new_content).unwrap();
    let snapshot = buffer_diff.snapshot();
    let elapsed = start.elapsed();
    
    println!("Diff completed in {:.2?}", elapsed);
    println!("Statistics:");
    println!("  Hunks: {}", snapshot.hunk_count());
    println!("  Added lines: {}", snapshot.added_lines());
    println!("  Deleted lines: {}", snapshot.deleted_lines());
    println!("  Unchanged lines: {}", snapshot.unchanged_lines());
    
    // Write output to file if requested
    if let Some(output_path) = output_file {
        println!("Writing diff to {}...", output_path);
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        
        for (i, hunk) in snapshot.hunks().iter().enumerate() {
            writeln!(writer, "Hunk #{} ({})", i + 1, hunk.status)?;
            writeln!(writer, "  Old range: lines {}-{}", 
                hunk.old_range.start + 1, 
                hunk.old_range.end())?;
            writeln!(writer, "  New range: lines {}-{}", 
                hunk.new_range.start + 1, 
                hunk.new_range.end())?;
            writeln!(writer, "  Lines: {} old, {} new", 
                hunk.old_range.count, 
                hunk.new_range.count)?;
            writeln!(writer)?;
        }
    }
    
    println!("Done!");
    Ok(())
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn count_lines(text: &str) -> usize {
    text.lines().count()
}