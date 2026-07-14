use std::fs;
use std::time::SystemTime;

use anyhow::Result;

use crate::{
    state::{Commit, Index, Object},
    utils::{get_current_branch_path, get_repository_root},
};

pub fn commit(message: &str) -> Result<String> {
    let index = Index::load()?;
    let root = get_repository_root()?;

    let tree_hash = index.store_tree()?;

    let merge_head_path = root.join(".rgit/MERGE_HEAD");
    let mut parent_hashes = if let Ok(path) = get_current_branch_path()
        && path.exists()
    {
        vec![fs::read_to_string(path)?]
    } else {
        Vec::new()
    };

    if merge_head_path.exists() {
        parent_hashes.push(fs::read_to_string(&merge_head_path)?);
        fs::remove_file(merge_head_path)?;
    }

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
