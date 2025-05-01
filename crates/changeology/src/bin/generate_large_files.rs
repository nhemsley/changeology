use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::distributions::Alphanumeric;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Set default values
    let mut file1_path = String::from("large_file1.txt");
    let mut file2_path = String::from("large_file2.txt");
    let mut line_count = 5000;
    
    // Parse arguments if provided
    if args.len() > 1 {
        file1_path = args[1].clone();
    }
    if args.len() > 2 {
        file2_path = args[2].clone();
    }
    if args.len() > 3 {
        if let Ok(count) = args[3].parse::<usize>() {
            line_count = count;
        }
    }
    
    // Display information
    println!("Generating large files for diff testing:");
    println!("  Original file: {}", file1_path);
    println!("  Modified file: {}", file2_path);
    println!("  Line count: {}", line_count);
    
    // Use a seeded RNG for reproducibility
    let mut rng = StdRng::seed_from_u64(42);
    
    // Step 1: Generate the original large file
    generate_large_file(&file1_path, line_count, &mut rng)?;
    
    // Step 2: Generate the modified version
    generate_modified_file(&file1_path, &file2_path, &mut rng)?;
    
    println!("Done! Files generated successfully.");
    Ok(())
}

/// Generate a large file with random content
fn generate_large_file<P: AsRef<Path>>(file_path: P, line_count: usize, rng: &mut StdRng) -> io::Result<()> {
    println!("Generating original file with {} lines...", line_count);
    
    let file = File::create(file_path)?;
    let mut writer = io::BufWriter::new(file);
    
    for i in 0..line_count {
        // Create a random line with varying length (20-70 chars)
        let line_length = rng.gen_range(20..70);
        let random_content: String = (0..line_length)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();
        
        writeln!(writer, "Line {}: {}", i + 1, random_content)?;
    }
    
    writer.flush()?;
    Ok(())
}

/// Generate a modified version of a large file
fn generate_modified_file<P: AsRef<Path>>(src_path: P, dst_path: P, rng: &mut StdRng) -> io::Result<()> {
    println!("Creating modified version...");
    
    // Read the original file
    let file = File::open(&src_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    
    // Create the modified file
    let file = File::create(dst_path)?;
    let mut writer = io::BufWriter::new(file);
    
    // Calculate how many modifications to make (about 5% of lines)
    let total_lines = lines.len();
    let num_changes = total_lines / 20;
    
    println!("Making approximately {} modifications...", num_changes);
    
    // Create a set of line numbers to modify
    let mut lines_to_modify = Vec::new();
    for _ in 0..num_changes {
        lines_to_modify.push(rng.gen_range(0..total_lines));
    }
    
    // Process each line
    let mut i = 0;
    while i < total_lines {
        if lines_to_modify.contains(&i) {
            // Decide action: 0=modify, 1=delete, 2=insert
            let action = rng.gen_range(0..3);
            
            match action {
                0 => { // Modify the line
                    let line_length = rng.gen_range(20..70);
                    let random_content: String = (0..line_length)
                        .map(|_| rng.sample(Alphanumeric) as char)
                        .collect();
                    
                    writeln!(writer, "Modified line {}: {}", i + 1, random_content)?;
                },
                1 => { // Delete the line (do nothing)
                    continue;
                },
                2 => { // Add a new line before this one
                    let line_length = rng.gen_range(20..70);
                    let random_content: String = (0..line_length)
                        .map(|_| rng.sample(Alphanumeric) as char)
                        .collect();
                    
                    writeln!(writer, "Added line: {}", random_content)?;
                    writeln!(writer, "{}", lines[i])?;
                },
                _ => unreachable!(),
            }
        } else {
            // Keep the line unchanged
            writeln!(writer, "{}", lines[i])?;
        }
        
        i += 1;
    }
    
    writer.flush()?;
    Ok(())
}