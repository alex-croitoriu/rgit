mod diff;
mod index;
mod object_store;
mod ref_store;
mod repository;
mod working_tree;

pub use diff::{FileDiff, TextDiff, TextDiffEntry};
pub use index::{Index, IndexEntry};
pub use object_store::{
    Blob, Commit, Object, Tree, TreeEntry, TreeEntryType, read_blob_bytes, read_blob_text,
};
pub use ref_store::{Head, branch_path, current_branch_path, head, head_hash, update_head};
pub use repository::Repository;
pub use working_tree::{ignored_paths, unstaged_changes, update_working_tree};
