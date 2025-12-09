//! # Virtual file paths and loading
//!
//! Analysis file IDs and file roots are owned by `salsa`. This crate keeps the
//! path types, file-root partitioning helpers, and loader abstraction used by
//! rust-analyzer's outer IO shell.
//!
//! [`PathClassifier`] classifies loaded paths while [`loader::Entry`] controls
//! file watching. In particular, a
//! single file root may correspond to several [`loader::Entry`] values. For
//! example, a crate from crates.io which uses code generation would have entries
//! for sources in `~/.cargo` and for generated code in `./target/debug/build`,
//! while those files are classified into one file root.
//!
//! [`PathClassifier`]: path_classifier::PathClassifier
//! [`Handle`]: loader::Handle
//! [`Entries`]: loader::Entry

mod anchored_path;
pub mod loader;
pub mod path_classifier;
mod vfs_path;

pub use crate::{
    anchored_path::{AnchoredPath, AnchoredPathBuf},
    vfs_path::VfsPath,
};
pub use paths::{AbsPath, AbsPathBuf};

/// Kind of file change received from the loader or pending in the server shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    /// The file was (re-)created
    Create,
    /// The file was modified
    Modify,
    /// The file was deleted
    Delete,
}
