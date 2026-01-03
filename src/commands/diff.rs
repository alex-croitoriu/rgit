use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    object_store::Object,
    utils::{get_branch_path, get_current_branch_path, get_commit_index_diff},
};

pub fn diff(target: Option<String>) -> Result<()> {
    if let Some(target) = target {
        let branch_path = get_branch_path(&target)?;
        if !branch_path.exists() {
            return Err(anyhow!("Branch does not exist"));
        }

        let hash = fs::read_to_string(branch_path)?;
        let diff = get_commit_index_diff(&hash)?;
        println!("{diff:?}");
    } else {
        let commit_hash = fs::read_to_string(get_current_branch_path()?)?;
        let commit_object = Object::load(&commit_hash)?;

        if let Object::Commit(commit) = commit_object {
            if commit.parent_hashes.is_empty() {
                return Err(anyhow!("Commit has no parents"));
            }

            for parent_hash in commit.parent_hashes {
                if let Ok(diff) = get_commit_index_diff(&parent_hash) {
                    println!("{diff:?}");
                }
            }
        }
    }

    Ok(())
}
