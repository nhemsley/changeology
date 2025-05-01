// Git integration for Changeology
// This crate provides access to git repository operations and status information

mod repository;
mod status;

pub use repository::Repository;
pub use status::{FileStatus, StatusEntry, StatusKind, StatusList};