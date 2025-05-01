// Core diff library for Changeology
// This crate provides diff calculation and representation

mod buffer_diff;
mod diff_hunk;
mod text_diff;

pub use buffer_diff::{BufferDiff, BufferDiffSnapshot};
pub use diff_hunk::{DiffHunk, DiffHunkStatus, DiffHunkRange, DiffLineType, DiffHunkSecondaryStatus};
pub use text_diff::TextDiff;