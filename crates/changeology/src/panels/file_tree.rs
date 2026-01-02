//! File tree panel utilities
//!
//! Provides helpers for building tree structures from git status
//! and rendering file trees with appropriate icons and colors.

use git::{StatusKind, StatusList};
use gpui::*;
use gpui_component::{tree::TreeItem, ActiveTheme, IconName};
use std::collections::HashMap;

/// Get the appropriate icon for a file or folder
pub fn get_file_icon(is_folder: bool, is_expanded: bool) -> IconName {
    if is_folder {
        if is_expanded {
            IconName::FolderOpen
        } else {
            IconName::Folder
        }
    } else {
        IconName::File
    }
}

/// Get the appropriate icon based on file extension
/// Note: Using File icon for all types since gpui-component has limited icon set
pub fn get_file_type_icon(_path: &str) -> IconName {
    // gpui-component 0.5.0 only has basic icons (File, Folder, FolderOpen)
    // All files use the same icon
    IconName::File
}

/// Get color based on git status
pub fn status_color(kind: StatusKind, cx: &App) -> Hsla {
    match kind {
        StatusKind::Added => cx.theme().green,
        StatusKind::Modified => cx.theme().yellow,
        StatusKind::Deleted => cx.theme().red,
        StatusKind::Renamed => cx.theme().blue,
        StatusKind::Untracked => cx.theme().muted_foreground,
        _ => cx.theme().foreground,
    }
}

/// Get a status indicator character for display
pub fn status_indicator(kind: StatusKind) -> &'static str {
    match kind {
        StatusKind::Added => "A",
        StatusKind::Modified => "M",
        StatusKind::Deleted => "D",
        StatusKind::Renamed => "R",
        StatusKind::Copied => "C",
        StatusKind::Untracked => "?",
        StatusKind::Ignored => "!",
        StatusKind::Conflicted => "C",
        StatusKind::Unknown => "",
    }
}

/// Build tree items from git status as a flat list
pub fn build_flat_tree(status: &StatusList) -> Vec<TreeItem> {
    status
        .entries
        .iter()
        .map(|entry| {
            let filename = entry.path.split('/').last().unwrap_or(&entry.path);
            // Clone the strings to avoid lifetime issues
            TreeItem::new(entry.path.clone(), filename.to_string())
        })
        .collect()
}

/// Directory node for building nested tree structure
struct DirNode {
    name: String,
    path: String,
    children: HashMap<String, DirNode>,
    files: Vec<(String, String)>, // (full_path, filename)
}

impl DirNode {
    fn new(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            children: HashMap::new(),
            files: Vec::new(),
        }
    }

    fn into_tree_item(self) -> TreeItem {
        let mut item = TreeItem::new(self.path, self.name).expanded(true);

        // Add subdirectories first (sorted)
        let mut dirs: Vec<_> = self.children.into_values().collect();
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        for dir in dirs {
            item = item.child(dir.into_tree_item());
        }

        // Add files (sorted)
        let mut files = self.files;
        files.sort_by(|a, b| a.1.cmp(&b.1));
        for (path, name) in files {
            item = item.child(TreeItem::new(path, name));
        }

        item
    }
}

/// Build tree items with directory hierarchy
pub fn build_nested_tree(status: &StatusList) -> Vec<TreeItem> {
    let mut root_dirs: HashMap<String, DirNode> = HashMap::new();
    let mut root_files: Vec<(String, String)> = Vec::new();

    for entry in &status.entries {
        let parts: Vec<&str> = entry.path.split('/').collect();

        if parts.len() == 1 {
            // Root level file
            root_files.push((entry.path.clone(), parts[0].to_string()));
        } else {
            // File in subdirectory
            let dir_name = parts[0];
            let dir_node = root_dirs
                .entry(dir_name.to_string())
                .or_insert_with(|| DirNode::new(dir_name, dir_name));

            // Navigate/create nested directories
            let mut current = dir_node;
            for (i, part) in parts[1..parts.len() - 1].iter().enumerate() {
                let nested_path = parts[0..=i + 1].join("/");
                current = current
                    .children
                    .entry(part.to_string())
                    .or_insert_with(|| DirNode::new(part, &nested_path));
            }

            // Add the file to the deepest directory
            let filename = parts.last().unwrap().to_string();
            current.files.push((entry.path.clone(), filename));
        }
    }

    // Build result: directories first, then root files
    let mut result = Vec::new();

    // Add directories (sorted)
    let mut dirs: Vec<_> = root_dirs.into_values().collect();
    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    for dir in dirs {
        result.push(dir.into_tree_item());
    }

    // Add root files (sorted)
    root_files.sort_by(|a, b| a.1.cmp(&b.1));
    for (path, name) in root_files {
        result.push(TreeItem::new(path, name));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use git::StatusEntry;

    fn make_status(paths: &[&str]) -> StatusList {
        StatusList {
            entries: paths
                .iter()
                .map(|p| StatusEntry {
                    path: p.to_string(),
                    kind: StatusKind::Modified,
                })
                .collect(),
        }
    }

    #[test]
    fn test_flat_tree() {
        let status = make_status(&["file1.rs", "src/main.rs", "src/lib.rs"]);
        let items = build_flat_tree(&status);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].label, "file1.rs");
        assert_eq!(items[1].label, "main.rs");
        assert_eq!(items[2].label, "lib.rs");
    }

    #[test]
    fn test_nested_tree() {
        let status = make_status(&[
            "Cargo.toml",
            "src/main.rs",
            "src/lib.rs",
            "src/util/helpers.rs",
        ]);
        let items = build_nested_tree(&status);

        // Should have: src/ directory, Cargo.toml file
        assert_eq!(items.len(), 2);
    }
}
