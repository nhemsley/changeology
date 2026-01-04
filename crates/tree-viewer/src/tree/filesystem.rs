//! Filesystem tree implementation with lazy loading support

use crate::tree::{Node, NodeId, NodeKind, Tree};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Metadata for filesystem nodes
#[derive(Debug, Clone, Default)]
pub struct FileData {
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Last modified timestamp
    pub modified: Option<SystemTime>,
    /// File extension (if any)
    pub extension: Option<String>,
}

impl std::fmt::Display for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.size)
    }
}

/// State of a node's children - loaded or not yet loaded
#[derive(Debug, Clone)]
enum ChildrenState {
    /// Children have not been loaded yet
    NotLoaded,
    /// Children are currently being loaded
    Loading,
    /// Children have been loaded
    Loaded(Vec<NodeId>),
    /// Error occurred during loading
    Error(String),
}

/// Internal node storage with lazy loading state
#[derive(Debug, Clone)]
struct FsNode {
    /// The node data
    node: Node<FileData>,
    /// Full path on the filesystem
    full_path: PathBuf,
    /// Parent node ID
    parent: Option<NodeId>,
    /// Children loading state
    children: ChildrenState,
}

/// A filesystem tree with lazy loading support
///
/// This tree loads directory contents on-demand rather than loading the entire
/// filesystem hierarchy upfront. This makes it suitable for large directory trees.
///
/// # Example
///
/// ```ignore
/// let tree = FilesystemTree::new("/path/to/directory")?;
///
/// // Root is loaded immediately
/// let root = tree.root();
///
/// // Children are loaded on first access
/// for child in tree.children(root) {
///     println!("{}", tree.name(child).unwrap());
/// }
/// ```
pub struct FilesystemTree {
    /// Arena storage for nodes
    nodes: Vec<FsNode>,
    /// Root path for relative path calculations
    root_path: PathBuf,
    /// Cache of path -> NodeId for quick lookups
    path_cache: HashMap<PathBuf, NodeId>,
}

impl FilesystemTree {
    /// Create a new filesystem tree rooted at the given path
    ///
    /// The root directory is loaded immediately, but its children are loaded lazily.
    ///
    /// # Errors
    ///
    /// Returns an error if the path doesn't exist or isn't a directory.
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;

        if !metadata.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must be a directory",
            ));
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string();

        let root_node = FsNode {
            node: Node::container(
                name.clone(),
                FileData {
                    size: 0,
                    modified: metadata.modified().ok(),
                    extension: None,
                },
            ),
            full_path: path.to_path_buf(),
            parent: None,
            children: ChildrenState::NotLoaded,
        };

        let mut path_cache = HashMap::new();
        path_cache.insert(path.to_path_buf(), NodeId::ROOT);

        Ok(Self {
            nodes: vec![root_node],
            root_path: path.to_path_buf(),
            path_cache,
        })
    }

    /// Get the full filesystem path for a node
    pub fn full_path(&self, id: NodeId) -> Option<&Path> {
        self.nodes.get(id.get()).map(|n| n.full_path.as_path())
    }

    /// Ensure children are loaded for a node
    ///
    /// This is called automatically by `children()` but can be called explicitly
    /// to preload children.
    pub fn ensure_loaded(&mut self, id: NodeId) -> Result<(), String> {
        // Check if already loaded
        if let Some(node) = self.nodes.get(id.get()) {
            if matches!(node.children, ChildrenState::Loaded(_)) {
                return Ok(());
            }
            if !node.node.is_container() {
                return Ok(());
            }
        } else {
            return Err("Invalid node ID".to_string());
        }

        // Get the path (we need to do this before mut borrow)
        let path = self.nodes[id.get()].full_path.clone();

        // Mark as loading
        self.nodes[id.get()].children = ChildrenState::Loading;

        // Load children from filesystem
        match self.load_children(&path) {
            Ok(child_ids) => {
                // Update parent's children list
                for child_id in &child_ids {
                    self.nodes[child_id.get()].parent = Some(id);
                }
                self.nodes[id.get()].children = ChildrenState::Loaded(child_ids);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.nodes[id.get()].children = ChildrenState::Error(error_msg.clone());
                Err(error_msg)
            }
        }
    }

    /// Load children from the filesystem
    fn load_children(&mut self, path: &Path) -> std::io::Result<Vec<NodeId>> {
        let mut child_ids = Vec::new();

        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = entry.metadata()?;

            let name = entry.file_name().to_string_lossy().to_string();

            let kind = if metadata.is_dir() {
                NodeKind::Container
            } else {
                NodeKind::Leaf
            };

            let extension = if metadata.is_file() {
                entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            };

            let file_data = FileData {
                size: metadata.len(),
                modified: metadata.modified().ok(),
                extension,
            };

            let node = FsNode {
                node: Node::new(name, kind, file_data),
                full_path: entry_path.clone(),
                parent: None, // Will be set by caller
                children: ChildrenState::NotLoaded,
            };

            let node_id = NodeId::new(self.nodes.len());
            self.nodes.push(node);
            self.path_cache.insert(entry_path, node_id);
            child_ids.push(node_id);
        }

        // Sort children: directories first, then files, alphabetically within each group
        child_ids.sort_by(|&a, &b| {
            let node_a = &self.nodes[a.get()].node;
            let node_b = &self.nodes[b.get()].node;

            match (node_a.kind, node_b.kind) {
                (NodeKind::Container, NodeKind::Leaf) => std::cmp::Ordering::Less,
                (NodeKind::Leaf, NodeKind::Container) => std::cmp::Ordering::Greater,
                _ => node_a.name.cmp(&node_b.name),
            }
        });

        Ok(child_ids)
    }

    /// Reload children for a node, discarding any previously loaded data
    pub fn reload(&mut self, id: NodeId) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(id.get()) {
            if node.node.is_container() {
                node.children = ChildrenState::NotLoaded;
            }
        }
        self.ensure_loaded(id)
    }

    /// Check if a node's children have been loaded
    pub fn is_loaded(&self, id: NodeId) -> bool {
        self.nodes
            .get(id.get())
            .map(|n| matches!(n.children, ChildrenState::Loaded(_)))
            .unwrap_or(false)
    }

    /// Recursively load all children (use with caution on large trees!)
    pub fn load_recursive(&mut self, id: NodeId) -> Result<(), String> {
        self.ensure_loaded(id)?;

        let children: Vec<_> = if let Some(node) = self.nodes.get(id.get()) {
            if let ChildrenState::Loaded(ref children) = node.children {
                children.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for child in children {
            if self.is_container(child) {
                self.load_recursive(child)?;
            }
        }

        Ok(())
    }

    /// Get the relative path from the tree root
    pub fn relative_path(&self, id: NodeId) -> Option<PathBuf> {
        let full_path = self.full_path(id)?;
        full_path
            .strip_prefix(&self.root_path)
            .ok()
            .map(|p| p.to_path_buf())
    }
}

impl Tree for FilesystemTree {
    type NodeData = FileData;

    fn root(&self) -> NodeId {
        NodeId::ROOT
    }

    fn get(&self, id: NodeId) -> Option<&Node<FileData>> {
        self.nodes.get(id.get()).map(|n| &n.node)
    }

    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes.get(id.get())?.parent
    }

    fn children(&self, id: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        // We need to ensure children are loaded, but we can't modify self in an immutable method
        // So we return an empty iterator if not loaded
        // Users should call ensure_loaded() first or use the mutable API

        if let Some(node) = self.nodes.get(id.get()) {
            if let ChildrenState::Loaded(ref children) = node.children {
                return Box::new(children.iter().copied());
            }
        }

        Box::new(std::iter::empty())
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree() -> (TempDir, FilesystemTree) {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create test structure:
        // root/
        //   file1.txt
        //   dir1/
        //     file2.txt
        //     dir2/
        //       file3.txt

        fs::write(root.join("file1.txt"), "content1").unwrap();
        fs::create_dir(root.join("dir1")).unwrap();
        fs::write(root.join("dir1/file2.txt"), "content2").unwrap();
        fs::create_dir(root.join("dir1/dir2")).unwrap();
        fs::write(root.join("dir1/dir2/file3.txt"), "content3").unwrap();

        let tree = FilesystemTree::new(root).unwrap();
        (temp, tree)
    }

    #[test]
    fn test_filesystem_tree_creation() {
        let (_temp, tree) = create_test_tree();
        assert_eq!(tree.node_count(), 1); // Only root loaded initially
        assert!(tree.is_container(tree.root()));
    }

    #[test]
    fn test_lazy_loading() {
        let (_temp, mut tree) = create_test_tree();

        // Initially only root is loaded
        assert_eq!(tree.node_count(), 1);
        assert!(!tree.is_loaded(tree.root()));

        // Load root's children
        tree.ensure_loaded(tree.root()).unwrap();
        assert!(tree.is_loaded(tree.root()));

        // Should have root + its immediate children
        assert!(tree.node_count() > 1);

        let children: Vec<_> = tree.children(tree.root()).collect();
        assert_eq!(children.len(), 2); // file1.txt and dir1
    }

    #[test]
    fn test_recursive_loading() {
        let (_temp, mut tree) = create_test_tree();

        tree.load_recursive(tree.root()).unwrap();

        // Should have all nodes loaded: root, file1, dir1, file2, dir2, file3
        assert_eq!(tree.node_count(), 6);
    }

    #[test]
    fn test_path_operations() {
        let (_temp, mut tree) = create_test_tree();
        tree.load_recursive(tree.root()).unwrap();

        let root_path = tree.path(tree.root());
        assert_eq!(root_path.components().count(), 1);

        // Find dir1
        let children: Vec<_> = tree.children(tree.root()).collect();
        let dir1 = children
            .iter()
            .find(|&&id| tree.name(id).unwrap().starts_with("dir"))
            .copied()
            .unwrap();

        let dir1_path = tree.relative_path(dir1).unwrap();
        assert_eq!(dir1_path.to_str().unwrap(), "dir1");
    }
}
