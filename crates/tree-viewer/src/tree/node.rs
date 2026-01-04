//! Core node types for the tree abstraction

use std::fmt;

/// Unique identifier for a node within a tree
///
/// Internally represented as an index into an arena-based storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    /// The root node always has ID 0
    pub const ROOT: NodeId = NodeId(0);

    /// Create a new NodeId from a usize
    pub const fn new(id: usize) -> Self {
        NodeId(id)
    }

    /// Get the inner usize value
    pub const fn get(self) -> usize {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        NodeId(id)
    }
}

impl From<NodeId> for usize {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

/// The type/kind of a node in the tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// A container node - can have children (e.g., directory)
    Container,
    /// A leaf node - cannot have children (e.g., file)
    Leaf,
}

impl NodeKind {
    /// Returns true if this is a container node
    pub const fn is_container(self) -> bool {
        matches!(self, NodeKind::Container)
    }

    /// Returns true if this is a leaf node
    pub const fn is_leaf(self) -> bool {
        matches!(self, NodeKind::Leaf)
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Container => write!(f, "Container"),
            NodeKind::Leaf => write!(f, "Leaf"),
        }
    }
}

/// A single node in the tree
///
/// Generic over the data type `D` which can store arbitrary user-defined
/// metadata (file size, git status, etc.)
#[derive(Debug, Clone)]
pub struct Node<D> {
    /// The node's name (not full path)
    pub name: String,
    /// Whether this is a container or leaf node
    pub kind: NodeKind,
    /// User-defined data associated with this node
    pub data: D,
}

impl<D> Node<D> {
    /// Create a new node
    pub fn new(name: impl Into<String>, kind: NodeKind, data: D) -> Self {
        Self {
            name: name.into(),
            kind,
            data,
        }
    }

    /// Create a new container node
    pub fn container(name: impl Into<String>, data: D) -> Self {
        Self::new(name, NodeKind::Container, data)
    }

    /// Create a new leaf node
    pub fn leaf(name: impl Into<String>, data: D) -> Self {
        Self::new(name, NodeKind::Leaf, data)
    }

    /// Returns true if this is a container node
    pub fn is_container(&self) -> bool {
        self.kind.is_container()
    }

    /// Returns true if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.kind.is_leaf()
    }
}

impl<D: Default> Node<D> {
    /// Create a new container node with default data
    pub fn container_default(name: impl Into<String>) -> Self {
        Self::container(name, D::default())
    }

    /// Create a new leaf node with default data
    pub fn leaf_default(name: impl Into<String>) -> Self {
        Self::leaf(name, D::default())
    }
}

impl<D: fmt::Display> fmt::Display for Node<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) - {}", self.name, self.kind, self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        assert_eq!(NodeId::ROOT, NodeId(0));
        assert_eq!(NodeId::new(5).get(), 5);
        assert_eq!(NodeId::from(10), NodeId(10));
        assert_eq!(usize::from(NodeId(7)), 7);
    }

    #[test]
    fn test_node_kind() {
        assert!(NodeKind::Container.is_container());
        assert!(!NodeKind::Container.is_leaf());
        assert!(NodeKind::Leaf.is_leaf());
        assert!(!NodeKind::Leaf.is_container());
    }

    #[test]
    fn test_node() {
        let node = Node::container("dir", 42);
        assert_eq!(node.name, "dir");
        assert!(node.is_container());
        assert!(!node.is_leaf());
        assert_eq!(node.data, 42);

        let leaf = Node::leaf("file.txt", "hello");
        assert_eq!(leaf.name, "file.txt");
        assert!(leaf.is_leaf());
        assert!(!leaf.is_container());
        assert_eq!(leaf.data, "hello");
    }

    #[test]
    fn test_node_default() {
        let node: Node<i32> = Node::container_default("test");
        assert_eq!(node.data, 0);

        let leaf: Node<String> = Node::leaf_default("file");
        assert_eq!(leaf.data, "");
    }
}
