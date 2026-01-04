//! Core tree abstraction for hierarchical data structures
//!
//! This module provides a generic tree trait that can represent any hierarchical
//! data structure, with specific focus on filesystem-like trees.

pub mod filesystem;
mod node;
mod traits;

pub use filesystem::{FileData, FilesystemTree};
pub use node::{Node, NodeId, NodeKind};
pub use traits::{TraversalOrder, Tree, TreeTraversal};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        FileData, FilesystemTree, Node, NodeId, NodeKind, TraversalOrder, Tree, TreeTraversal,
    };
}
