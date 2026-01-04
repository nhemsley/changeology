//! CLI example that loads a directory and displays its full hierarchy
//!
//! Usage:
//!   cargo run --example list_directory [path]
//!
//! If no path is provided, uses the current directory.

use std::env;
use tree_viewer::tree::prelude::*;

fn main() {
    // Get path from command line args or use current directory
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "." };

    println!("Loading directory: {}", path);
    println!();

    // Create the filesystem tree
    let mut tree = match FilesystemTree::new(path) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("Error loading directory: {}", e);
            std::process::exit(1);
        }
    };

    // Load the entire tree recursively
    println!("Loading directory tree...");
    if let Err(e) = tree.load_recursive(tree.root()) {
        eprintln!("Error loading tree: {}", e);
        std::process::exit(1);
    }

    println!();
    println!("Tree loaded successfully!");
    println!("Total nodes: {}", tree.node_count());
    println!("Files: {}", tree.leaves().len());
    println!("Directories: {}", tree.containers().len());
    println!();
    println!("Directory Structure:");
    println!("═══════════════════════════════");
    println!();

    // Walk the tree and display it
    for id in tree.walk(TraversalOrder::PreOrder) {
        let node = tree.get(id).unwrap();
        let depth = tree.depth(id);
        let indent = "  ".repeat(depth);

        // Choose icon based on node kind
        let icon = match node.kind {
            NodeKind::Container => "📁",
            NodeKind::Leaf => "📄",
        };

        // Display with size info for files
        if node.kind == NodeKind::Leaf {
            let size = format_size(node.data.size);
            println!("{}{} {} ({})", indent, icon, node.name, size);
        } else {
            // For directories, show child count
            let child_count = tree.child_count(id);
            println!("{}{} {} ({} items)", indent, icon, node.name, child_count);
        }
    }

    println!();
    println!("═══════════════════════════════");
    println!("Summary:");
    println!("  Total items: {}", tree.node_count());
    println!("  Directories: {}", tree.containers().len());
    println!("  Files: {}", tree.leaves().len());

    // Calculate total size
    let total_size: u64 = tree
        .leaves()
        .iter()
        .filter_map(|&id| tree.get(id))
        .map(|n| n.data.size)
        .sum();
    println!("  Total size: {}", format_size(total_size));
}

/// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
