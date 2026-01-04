//! Core tree traits for hierarchical data structures

use crate::tree::{Node, NodeId, NodeKind};
use std::path::PathBuf;

/// A hierarchical tree structure that maps to filesystem-like concepts
///
/// This trait provides the core abstraction for working with tree data structures.
/// Implementations must provide basic operations for navigating the tree, while
/// derived methods provide convenient higher-level operations.
///
/// # Type Parameters
///
/// * `NodeData` - User-defined data stored at each node (e.g., file size, git status)
///
/// # Example
///
/// ```ignore
/// fn print_tree<T: Tree>(tree: &T) {
///     for id in tree.walk(TraversalOrder::PreOrder) {
///         let node = tree.get(id).unwrap();
///         let depth = tree.depth(id);
///         println!("{:indent$}{}", "", node.name, indent = depth * 2);
///     }
/// }
/// ```
pub trait Tree {
    /// User-defined data stored at each node
    type NodeData;

    /// Get the root node ID (always exists)
    ///
    /// The root is guaranteed to exist and typically has ID 0.
    fn root(&self) -> NodeId;

    /// Get a node by its ID
    ///
    /// Returns `None` if the ID is invalid.
    fn get(&self, id: NodeId) -> Option<&Node<Self::NodeData>>;

    /// Get the parent of a node
    ///
    /// Returns `None` for the root node.
    fn parent(&self, id: NodeId) -> Option<NodeId>;

    /// Iterate over children of a node
    ///
    /// Returns an empty iterator for leaf nodes or invalid IDs.
    fn children(&self, id: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;

    /// Check if a node is a leaf (cannot have children)
    ///
    /// Returns false for invalid IDs.
    fn is_leaf(&self, id: NodeId) -> bool {
        self.get(id)
            .map(|n| n.kind == NodeKind::Leaf)
            .unwrap_or(false)
    }

    /// Check if a node is a container (can have children)
    ///
    /// Returns false for invalid IDs.
    fn is_container(&self, id: NodeId) -> bool {
        self.get(id)
            .map(|n| n.kind == NodeKind::Container)
            .unwrap_or(false)
    }

    /// Get the name of a node
    ///
    /// Returns `None` if the ID is invalid.
    fn name(&self, id: NodeId) -> Option<&str> {
        self.get(id).map(|n| n.name.as_str())
    }

    /// Get the full path from root to this node
    ///
    /// Returns an empty path if the ID is invalid.
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
    ///
    /// Returns 0 for invalid IDs.
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
    ///
    /// Returns 0 for leaf nodes or invalid IDs.
    fn child_count(&self, id: NodeId) -> usize {
        self.children(id).count()
    }

    /// Get all ancestors of a node, from parent to root
    ///
    /// Returns an empty vector for the root or invalid IDs.
    fn ancestors(&self, id: NodeId) -> Vec<NodeId> {
        let mut ancestors = Vec::new();
        let mut current = self.parent(id);
        while let Some(parent_id) = current {
            ancestors.push(parent_id);
            current = self.parent(parent_id);
        }
        ancestors
    }

    /// Check if a node is an ancestor of another
    fn is_ancestor_of(&self, ancestor: NodeId, descendant: NodeId) -> bool {
        let mut current = self.parent(descendant);
        while let Some(parent_id) = current {
            if parent_id == ancestor {
                return true;
            }
            current = self.parent(parent_id);
        }
        false
    }
}

/// Traversal order for walking the tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraversalOrder {
    /// Visit parent before children (top-down)
    PreOrder,
    /// Visit children before parent (bottom-up)
    PostOrder,
    /// Visit level by level (breadth-first)
    BreadthFirst,
}

/// Extension trait providing tree traversal and search utilities
///
/// This trait is automatically implemented for all types that implement `Tree`.
pub trait TreeTraversal: Tree {
    /// Walk the tree from the root in the specified order
    fn walk(&self, order: TraversalOrder) -> TreeWalker<'_, Self>
    where
        Self: Sized,
    {
        TreeWalker::new(self, self.root(), order)
    }

    /// Walk the tree starting from a specific node
    fn walk_from(&self, start: NodeId, order: TraversalOrder) -> TreeWalker<'_, Self>
    where
        Self: Sized,
    {
        TreeWalker::new(self, start, order)
    }

    /// Get all leaf nodes (files)
    fn leaves(&self) -> Vec<NodeId>
    where
        Self: Sized,
    {
        self.walk(TraversalOrder::PreOrder)
            .filter(|&id| self.is_leaf(id))
            .collect()
    }

    /// Get all container nodes (directories)
    fn containers(&self) -> Vec<NodeId>
    where
        Self: Sized,
    {
        self.walk(TraversalOrder::PreOrder)
            .filter(|&id| self.is_container(id))
            .collect()
    }

    /// Find nodes matching a predicate
    fn find<F>(&self, predicate: F) -> Vec<NodeId>
    where
        F: Fn(&Node<Self::NodeData>) -> bool,
        Self: Sized,
    {
        self.walk(TraversalOrder::PreOrder)
            .filter(|&id| self.get(id).map(&predicate).unwrap_or(false))
            .collect()
    }

    /// Find a node by path
    ///
    /// Returns `None` if the path doesn't exist.
    fn find_by_path(&self, path: impl AsRef<std::path::Path>) -> Option<NodeId> {
        let mut current = self.root();

        for component in path.as_ref().components() {
            let component_str = component.as_os_str().to_str()?;

            // Skip the root component if it matches the root name
            if current == self.root() && self.name(current) == Some(component_str) {
                continue;
            }

            current = self
                .children(current)
                .find(|&id| self.name(id) == Some(component_str))?;
        }

        Some(current)
    }

    /// Find a node by name (first match only)
    fn find_by_name(&self, name: &str) -> Option<NodeId>
    where
        Self: Sized,
    {
        self.walk(TraversalOrder::PreOrder)
            .find(|&id| self.name(id) == Some(name))
    }

    /// Find all nodes with a given name
    fn find_all_by_name(&self, name: &str) -> Vec<NodeId>
    where
        Self: Sized,
    {
        self.walk(TraversalOrder::PreOrder)
            .filter(|&id| self.name(id) == Some(name))
            .collect()
    }
}

// Blanket implementation for all Tree types
impl<T: Tree> TreeTraversal for T {}

/// Iterator for traversing a tree in different orders
pub struct TreeWalker<'a, T: Tree + ?Sized> {
    tree: &'a T,
    order: TraversalOrder,
    stack: Vec<NodeId>,
    visited: std::collections::HashSet<NodeId>,
}

impl<'a, T: Tree + ?Sized> TreeWalker<'a, T> {
    /// Create a new tree walker starting from the given node
    pub fn new(tree: &'a T, start: NodeId, order: TraversalOrder) -> Self {
        let mut stack = vec![start];
        let visited = std::collections::HashSet::new();

        // For breadth-first, we'll use the stack as a queue
        if matches!(order, TraversalOrder::BreadthFirst) {
            stack.reverse();
        }

        Self {
            tree,
            order,
            stack,
            visited,
        }
    }
}

impl<'a, T: Tree + ?Sized> Iterator for TreeWalker<'a, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        match self.order {
            TraversalOrder::PreOrder => self.next_preorder(),
            TraversalOrder::PostOrder => self.next_postorder(),
            TraversalOrder::BreadthFirst => self.next_breadthfirst(),
        }
    }
}

impl<'a, T: Tree + ?Sized> TreeWalker<'a, T> {
    fn next_preorder(&mut self) -> Option<NodeId> {
        let current = self.stack.pop()?;

        // Add children in reverse order so they're popped in correct order
        let children: Vec<_> = self.tree.children(current).collect();
        for child in children.into_iter().rev() {
            self.stack.push(child);
        }

        Some(current)
    }

    fn next_postorder(&mut self) -> Option<NodeId> {
        while let Some(&current) = self.stack.last() {
            if self.visited.contains(&current) {
                self.stack.pop();
                return Some(current);
            }

            self.visited.insert(current);

            // Add children in reverse order
            let children: Vec<_> = self.tree.children(current).collect();
            for child in children.into_iter().rev() {
                self.stack.push(child);
            }
        }
        None
    }

    fn next_breadthfirst(&mut self) -> Option<NodeId> {
        if self.stack.is_empty() {
            return None;
        }

        // Pop from front (treating stack as queue)
        let current = self.stack.remove(0);

        // Add children at the end
        for child in self.tree.children(current) {
            self.stack.push(child);
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test tree implementation
    struct TestTree {
        nodes: Vec<Node<i32>>,
        parents: Vec<Option<NodeId>>,
        children: Vec<Vec<NodeId>>,
    }

    impl TestTree {
        fn new() -> Self {
            Self {
                nodes: vec![Node::container("root", 0)],
                parents: vec![None],
                children: vec![vec![]],
            }
        }

        fn add_child(&mut self, parent: NodeId, node: Node<i32>) -> NodeId {
            let id = NodeId::new(self.nodes.len());
            self.nodes.push(node);
            self.parents.push(Some(parent));
            self.children.push(vec![]);
            self.children[parent.get()].push(id);
            id
        }
    }

    impl Tree for TestTree {
        type NodeData = i32;

        fn root(&self) -> NodeId {
            NodeId::ROOT
        }

        fn get(&self, id: NodeId) -> Option<&Node<i32>> {
            self.nodes.get(id.get())
        }

        fn parent(&self, id: NodeId) -> Option<NodeId> {
            self.parents.get(id.get()).copied().flatten()
        }

        fn children(&self, id: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
            Box::new(
                self.children
                    .get(id.get())
                    .map(|c| c.iter().copied())
                    .into_iter()
                    .flatten(),
            )
        }

        fn node_count(&self) -> usize {
            self.nodes.len()
        }
    }

    #[test]
    fn test_basic_tree_operations() {
        let mut tree = TestTree::new();
        let child1 = tree.add_child(NodeId::ROOT, Node::leaf("file1.txt", 1));
        let child2 = tree.add_child(NodeId::ROOT, Node::container("dir1", 2));
        let _child3 = tree.add_child(child2, Node::leaf("file2.txt", 3));

        assert_eq!(tree.node_count(), 4);
        assert_eq!(tree.child_count(NodeId::ROOT), 2);
        assert_eq!(tree.child_count(child2), 1);
        assert!(tree.is_leaf(child1));
        assert!(tree.is_container(child2));
    }

    #[test]
    fn test_tree_path() {
        let mut tree = TestTree::new();
        let dir1 = tree.add_child(NodeId::ROOT, Node::container("dir1", 0));
        let file1 = tree.add_child(dir1, Node::leaf("file.txt", 0));

        let path = tree.path(file1);
        assert_eq!(path.to_str().unwrap(), "root/dir1/file.txt");
    }

    #[test]
    fn test_tree_depth() {
        let mut tree = TestTree::new();
        let dir1 = tree.add_child(NodeId::ROOT, Node::container("dir1", 0));
        let dir2 = tree.add_child(dir1, Node::container("dir2", 0));
        let file1 = tree.add_child(dir2, Node::leaf("file.txt", 0));

        assert_eq!(tree.depth(NodeId::ROOT), 0);
        assert_eq!(tree.depth(dir1), 1);
        assert_eq!(tree.depth(dir2), 2);
        assert_eq!(tree.depth(file1), 3);
    }

    #[test]
    fn test_tree_traversal_preorder() {
        let mut tree = TestTree::new();
        let dir1 = tree.add_child(NodeId::ROOT, Node::container("dir1", 0));
        let file1 = tree.add_child(NodeId::ROOT, Node::leaf("file1.txt", 0));
        let file2 = tree.add_child(dir1, Node::leaf("file2.txt", 0));

        let nodes: Vec<_> = tree.walk(TraversalOrder::PreOrder).collect();
        assert_eq!(nodes, vec![NodeId::ROOT, dir1, file2, file1]);
    }
}
