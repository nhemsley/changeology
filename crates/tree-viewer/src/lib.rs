//! Tree Viewer Library
//!
//! A library for representing and visualizing hierarchical tree structures,
//! with a focus on filesystem-like hierarchies and 3D visualization using Bevy.
//!
//! # Core Concepts
//!
//! - **Tree**: Generic trait for hierarchical data structures
//! - **Node**: Individual elements in the tree (containers or leaves)
//! - **FilesystemTree**: Lazy-loading filesystem implementation
//!
//! # Example
//!
//! ```no_run
//! use tree_viewer::tree::prelude::*;
//!
//! // Create a filesystem tree (loads lazily)
//! let mut tree = FilesystemTree::new("./src").expect("Failed to load directory");
//!
//! // Load the root's children
//! tree.ensure_loaded(tree.root()).expect("Failed to load children");
//!
//! // Walk the tree
//! for id in tree.walk(TraversalOrder::PreOrder) {
//!     let node = tree.get(id).unwrap();
//!     let depth = tree.depth(id);
//!     println!("{:indent$}{}", "", node.name, indent = depth * 2);
//! }
//! ```

pub mod tree;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::tree::prelude::*;
}
