use anyhow::Result;
use diff::{DiffGranularity, TextDiff};

fn main() -> Result<()> {
    // Sample texts with word-level differences
    let text1 = "This is the first paragraph with some words.\nHere is another line with minor changes.\nThis line is unchanged.";
    let text2 = "This is the first paragraph with different words.\nHere is another sentence with major changes.\nThis line is unchanged.";
    
    println!("=== Line-level diff (default) ===");
    println!("{}", TextDiff::unified_diff(text1, text2, 1));
    
    println!("\n=== Word-level diff ===");
    println!("{}", TextDiff::unified_diff_with_granularity(text1, text2, 1, DiffGranularity::Word));
    
    println!("\n=== Character-level diff ===");
    println!("{}", TextDiff::unified_diff_with_granularity(text1, text2, 1, DiffGranularity::Character));
    
    // Example with code
    let code1 = "function calculateTotal(items) {\n    let sum = 0;\n    for (let i = 0; i < items.length; i++) {\n        sum += items[i].price;\n    }\n    return sum;\n}";
    let code2 = "function calculateTotal(items) {\n    let sum = 0;\n    for (let i = 0; i < items.length; i++) {\n        sum += items[i].price * items[i].quantity;\n    }\n    return sum;\n}";
    
    println!("\n=== Code diff (line-level) ===");
    println!("{}", TextDiff::unified_diff(code1, code2, 1));
    
    println!("\n=== Code diff (word-level) ===");
    println!("{}", TextDiff::unified_diff_with_granularity(code1, code2, 1, DiffGranularity::Word));
    
    Ok(())
}