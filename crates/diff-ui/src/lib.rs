//! Diff UI - GPUI components for displaying text diffs
//!
//! This crate provides reusable components for rendering text diffs
//! with colored backgrounds indicating added, deleted, and unchanged lines.

mod diff_text_view;
mod theme;

pub use diff_text_view::{DiffDisplayLine, DiffLineStyle, DiffTextView, RenderMode};
pub use theme::{DiffIndicatorColors, DiffTheme};
