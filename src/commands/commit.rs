use std::fs;
use std::time::SystemTime;

use anyhow::Result;

use crate::{
    index::Index,
    object_store::{Commit, Object},
    utils::{create_tree_from_index, get_current_branch_path},
};

pub fn commit(message: &str) -> Result<String> {
    let index = Index::load()?;

    let tree_hash = create_tree_from_index(&index)?;
    let parent_hashes = if let Ok(path) = get_current_branch_path()
        && path.exists()
    {
        vec![fs::read_to_string(path)?]
    } else {
        Vec::new()
    };

    let commit_object = Object::Commit(Commit {
        message: message.to_string(),
        parent_hashes,
        tree_hash,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
    });

    let commit_hash = commit_object.store()?;

    fs::write(get_current_branch_path()?, &commit_hash)?;

    Ok(commit_hash)
}
