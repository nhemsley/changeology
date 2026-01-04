# Tree Trait Design

## Overview

This document outlines the design for a `Tree` trait that cleanly maps to filesystem-like hierarchical structures. The goal is to create an abstraction that can represent directories, files, and subdirectories in a way that's both ergonomic and efficient for 3D visualization.

## Core Concepts

### Filesystem Mapping

| Filesystem Concept | Tree Abstraction |
|-------------------|------------------|
| Directory | Container node (has children) |
| File | Leaf node (no children) |
| Subdirectory | Nested container node |
| Path | Sequence of node names from root |
| Name | Node's identifier within parent |

### Design Goals

1. **Generic** - Works with any hierarchical data, not just filesystems
2. **Efficient** - Arena-based storage for cache-friendly traversal
3. **Ergonomic** - Clean API for common operations
4. **Visualization-friendly** - Easy to map to 3D scene graph
5. **Lazy-capable** - Support for lazy loading of large trees

---

## Proposed API

### Core Types

```rust
use std::path::PathBuf;

/// Unique identifier for a node within a tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub const ROOT: NodeId = NodeId(0);
}

/// The type of a node in the tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A container node (directory) - can have children
    Container,
    /// A leaf node (file) - cannot have children
    Leaf,
}

/// A single node in the tree
#[derive(Debug, Clone)]
pub struct Node<D> {
    /// The node's name (not full path)
    pub name: String,
    /// Whether this is a file or directory
    pub kind: NodeKind,
    /// User-defined data associated with this node
    pub data: D,
}
```

### The Tree Trait

```rust
/// A hierarchical tree structure that maps to filesystem concepts
pub trait Tree {
    /// User-defined data stored at each node
    type NodeData;

    /// Get the root node ID (always exists)
    fn root(&self) -> NodeId;

    /// Get a node by its ID
    fn get(&self, id: NodeId) -> Option<&Node<Self::NodeData>>;

    /// Get the parent of a node (None for root)
    fn parent(&self, id: NodeId) -> Option<NodeId>;

    /// Iterate over children of a node (empty for files)
    fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId>;

    /// Check if a node is a leaf (file)
    fn is_leaf(&self, id: NodeId) -> bool {
        self.get(id).map(|n| n.kind == NodeKind::Leaf).unwrap_or(false)
    }

    /// Check if a node is a container (directory)
    fn is_directory(&self, id: NodeId) -> bool {
        self.get(id).map(|n| n.kind == NodeKind::Container).unwrap_or(false)
    }

    /// Get the name of a node
    fn name(&self, id: NodeId) -> Option<&str> {
        self.get(id).map(|n| n.name.as_str())
    }

    /// Get the full path from root to this node
    fn path(&self, id: NodeId) -> PathBuf {
        let mut components = Vec::new();
        let mut current = Some(id);
        
        while let Some(node_id) = current {
            if let Some(name) = self.name(node_id) {
                components.push(name.to_string());
            }
            current = self.parent(node_id);
        }
        
        components.reverse();
        components.into_iter().collect()
    }

    /// Get the depth of a node (root = 0)
    fn depth(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut current = self.parent(id);
        while let Some(parent_id) = current {
            depth += 1;
            current = self.parent(parent_id);
        }
        depth
    }

    /// Count total nodes in the tree
    fn node_count(&self) -> usize;

    /// Count children of a node
    fn child_count(&self, id: NodeId) -> usize {
        self.children(id).count()
    }
}
```

### Mutable Tree Extension

```rust
/// Extension trait for mutable tree operations
pub trait TreeMut: Tree {
    /// Get mutable reference to a node
    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<Self::NodeData>>;

    /// Add a child node to a directory, returns the new node's ID
    fn add_child(&mut self, parent: NodeId, node: Node<Self::NodeData>) -> Option<NodeId>;

    /// Remove a node and all its descendants
    fn remove(&mut self, id: NodeId) -> bool;

    /// Move a node to a new parent
    fn move_node(&mut self, id: NodeId, new_parent: NodeId) -> bool;
}
```

---

## Concrete Implementation: ArenaTree

An arena-based implementation for efficient storage and traversal:

```rust
/// Arena-based tree implementation
pub struct ArenaTree<D> {
    nodes: Vec<Node<D>>,
    parents: Vec<Option<NodeId>>,
    children: Vec<Vec<NodeId>>,
}

impl<D> ArenaTree<D> {
    /// Create a new tree with a root directory
    pub fn new(root_name: impl Into<String>, root_data: D) -> Self {
        Self {
            nodes: vec![Node {
                name: root_name.into(),
                kind: NodeKind::Container,
                data: root_data,
            }],
            parents: vec![None],
            children: vec![vec![]],
        }
    }

    /// Create from a filesystem path
    pub fn from_path(path: impl AsRef<std::path::Path>) -> std::io::Result<ArenaTree<FileData>>
    where
        D: From<FileData>,
    {
        // Implementation would walk the filesystem
        todo!()
    }
}

impl<D> Tree for ArenaTree<D> {
    type NodeData = D;

    fn root(&self) -> NodeId {
        NodeId::ROOT
    }

    fn get(&self, id: NodeId) -> Option<&Node<D>> {
        self.nodes.get(id.0)
    }

    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.parents.get(id.0).copied().flatten()
    }

    fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> {
        self.children
            .get(id.0)
            .map(|c| c.iter().copied())
            .into_iter()
            .flatten()
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
```

---

## Node Data Types

### Basic File Data

```rust
/// Basic metadata for filesystem nodes
#[derive(Debug, Clone, Default)]
pub struct FileData {
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// Last modified timestamp
    pub modified: Option<std::time::SystemTime>,
    /// File extension (if any)
    pub extension: Option<String>,
}
```

### Git-Aware File Data

```rust
/// File data with git status information
#[derive(Debug, Clone, Default)]
pub struct GitFileData {
    /// Basic file metadata
    pub file: FileData,
    /// Git status of this file
    pub git_status: GitStatus,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GitStatus {
    #[default]
    Unchanged,
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
}
```

---

## Traversal Utilities

```rust
/// Traversal order for walking the tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalOrder {
    /// Visit parent before children
    PreOrder,
    /// Visit children before parent
    PostOrder,
    /// Visit level by level
    BreadthFirst,
}

/// Iterator extension for tree traversal
pub trait TreeTraversal: Tree {
    /// Walk the tree in the specified order
    fn walk(&self, order: TraversalOrder) -> impl Iterator<Item = NodeId> {
        TreeWalker::new(self, self.root(), order)
    }

    /// Walk starting from a specific node
    fn walk_from(&self, start: NodeId, order: TraversalOrder) -> impl Iterator<Item = NodeId> {
        TreeWalker::new(self, start, order)
    }

    /// Get all leaf nodes (files)
    fn leaves(&self) -> impl Iterator<Item = NodeId> {
        self.walk(TraversalOrder::PreOrder)
            .filter(|&id| self.is_leaf(id))
    }

    /// Get all directory nodes
    fn directories(&self) -> impl Iterator<Item = NodeId> {
        self.walk(TraversalOrder::PreOrder)
            .filter(|&id| self.is_directory(id))
    }

    /// Find nodes matching a predicate
    fn find<F>(&self, predicate: F) -> impl Iterator<Item = NodeId>
    where
        F: Fn(&Node<Self::NodeData>) -> bool,
    {
        self.walk(TraversalOrder::PreOrder)
            .filter(move |&id| {
                self.get(id).map(&predicate).unwrap_or(false)
            })
    }

    /// Find a node by path
    fn find_by_path(&self, path: impl AsRef<std::path::Path>) -> Option<NodeId> {
        let mut current = self.root();
        
        for component in path.as_ref().components() {
            let name = component.as_os_str().to_str()?;
            current = self.children(current)
                .find(|&id| self.name(id) == Some(name))?;
        }
        
        Some(current)
    }
}

// Blanket implementation
impl<T: Tree> TreeTraversal for T {}
```

---

## Visualization Integration

### Mapping to 3D Scene

```rust
/// Information needed to render a node in 3D
pub struct NodeVisual {
    /// Position in 3D space
    pub position: [f32; 3],
    /// Size/scale of the visual representation
    pub scale: [f32; 3],
    /// Color (RGBA)
    pub color: [f32; 4],
    /// Whether this node is currently selected
    pub selected: bool,
    /// Whether this node is expanded (for directories)
    pub expanded: bool,
}

/// Trait for mapping tree nodes to visual representations
pub trait TreeLayout: Tree {
    /// Calculate the visual representation for all nodes
    fn layout(&self) -> Vec<(NodeId, NodeVisual)>;
    
    /// Update layout when a node is expanded/collapsed
    fn update_layout(&self, changed: NodeId) -> Vec<(NodeId, NodeVisual)>;
}
```

### Layout Algorithms

Different layout algorithms can implement `TreeLayout`:

1. **FlatGrid** - Simple grid of cubes
2. **ConicalTree** - 3D cone tree layout
3. **TreeMap** - Space-filling treemap
4. **RadialTree** - Nodes radiate from center

---

## Usage Examples

### Creating a Tree from Filesystem

```rust
// Load a directory into an ArenaTree
let tree = ArenaTree::<FileData>::from_path("./src")?;

println!("Total nodes: {}", tree.node_count());
println!("Files: {}", tree.leaves().count());
println!("Directories: {}", tree.directories().count());
```

### Walking the Tree

```rust
// Pre-order traversal
for id in tree.walk(TraversalOrder::PreOrder) {
    let node = tree.get(id).unwrap();
    let depth = tree.depth(id);
    let indent = "  ".repeat(depth);
    
    let icon = match node.kind {
        NodeKind::Container => "📁",
        NodeKind::Leaf => "📄",
    };
    
    println!("{}{} {}", indent, icon, node.name);
}
```

### Finding Files

```rust
// Find all Rust files
let rust_files: Vec<_> = tree
    .find(|node| {
        node.kind == NodeKind::Leaf && 
        node.name.ends_with(".rs")
    })
    .collect();

// Find by path
if let Some(id) = tree.find_by_path("src/main.rs") {
    println!("Found: {:?}", tree.path(id));
}
```

### Building a Tree Programmatically

```rust
let mut tree = ArenaTree::new("root", FileData::default());

let src = tree.add_child(NodeId::ROOT, Node {
    name: "src".into(),
    kind: NodeKind::Container,
    data: FileData::default(),
}).unwrap();

tree.add_child(src, Node {
    name: "main.rs".into(),
    kind: NodeKind::Leaf,
    data: FileData { size: 1024, ..default() },
});

tree.add_child(src, Node {
    name: "lib.rs".into(),
    kind: NodeKind::Leaf,
    data: FileData { size: 2048, ..default() },
});
```

---

## Implementation Plan

### Phase 1: Core Types
- [ ] `NodeId`, `NodeKind`, `Node<D>`
- [ ] `Tree` trait with basic methods
- [ ] `ArenaTree<D>` implementation

### Phase 2: Traversal
- [ ] `TreeTraversal` extension trait
- [ ] Pre-order, post-order, breadth-first iterators
- [ ] Path-based lookup

### Phase 3: Filesystem Integration
- [ ] `FileData` and `GitFileData` types
- [ ] `ArenaTree::from_path()` implementation
- [ ] Filtering (gitignore, hidden files)

### Phase 4: Visualization
- [ ] `NodeVisual` type
- [ ] `TreeLayout` trait
- [ ] Basic layout algorithms

### Phase 5: Bevy Integration
- [ ] Spawn entities from tree
- [ ] Update visuals on tree changes
- [ ] Selection and interaction

---

## Open Questions

1. **Lazy Loading**: Should we support lazy loading of children for very large trees?
2. **Caching**: Should we cache computed values like depth and path?
3. **Change Detection**: How do we efficiently detect and propagate changes?
4. **Thread Safety**: Do we need `Send + Sync` bounds?
5. **Custom NodeId**: Should users be able to provide their own ID type?

---

## References

- Bevy ECS patterns for hierarchical data
- indextree crate for arena-based trees
- walkdir crate for filesystem traversal
- ignore crate for gitignore handling