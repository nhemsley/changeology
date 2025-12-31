use anyhow::Result;
use buffer_diff::{DiffGranularity, TextDiff};
use similar::Algorithm;

fn main() -> Result<()> {
    // Sample text with whitespace differences
    let text1 = "function calculateSum(a, b) {\n    return a+b;\n}";
    let text2 = "function  calculateSum(a,b){\n  return a + b;\n}";
    
    println!("=== Default diff (shows whitespace changes) ===");
    println!("{}", TextDiff::unified_diff(text1, text2, 1));
    
    println!("\n=== Ignoring whitespace ===");
    println!("{}", TextDiff::configure()
        .ignore_whitespace(true)
        .unified_diff(text1, text2));
    
    // Try different algorithms
    let complex_text1 = "This is a longer text with multiple paragraphs.\nIt contains several lines that will be changed.\nSome lines will remain the same.\nOthers will be modified extensively.";
    let complex_text2 = "This is a longer text with multiple sections.\nIt has several lines that have been modified.\nSome lines will remain the same.\nNew lines are also added here.\nAnd more content at the end.";
    
    println!("\n=== Myers algorithm (default) ===");
    println!("{}", TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .unified_diff(complex_text1, complex_text2));
    
    println!("\n=== Patience algorithm ===");
    println!("{}", TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .unified_diff(complex_text1, complex_text2));
    
    // Different levels of context
    println!("\n=== With 0 context lines ===");
    println!("{}", TextDiff::configure()
        .context_lines(0)
        .unified_diff(complex_text1, complex_text2));
    
    println!("\n=== With 2 context lines ===");
    println!("{}", TextDiff::configure()
        .context_lines(2)
        .unified_diff(complex_text1, complex_text2));
    
    // Combining multiple configuration options
    println!("\n=== Advanced configuration ===");
    println!("{}", TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .granularity(DiffGranularity::Word)
        .ignore_whitespace(true)
        .context_lines(1)
        .unified_diff(complex_text1, complex_text2));
    
    Ok(())
}