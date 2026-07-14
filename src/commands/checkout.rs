use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands::create,
    state::Index,
    utils::{
        get_branch_path, get_current_branch_path, get_repository_root, get_staged_changes,
        get_unstaged_changes, update_working_tree,
    },
};

// TODO: refactor this garbage
// TODO: add checkout on commits
pub fn checkout(target: &str) -> Result<()> {
    let staged_changes = get_staged_changes()?;
    let unstaged_changes = get_unstaged_changes()?;

    if !(staged_changes.0.is_empty()
        && staged_changes.1.is_empty()
        && staged_changes.2.is_empty()
        && unstaged_changes.0.is_empty()
        && unstaged_changes.1.is_empty()
        && unstaged_changes.2.is_empty())
    {
        return Err(anyhow!("Unable to checkout: uncommited changes"));
    }

    if let Ok(branch_path) = get_branch_path(target) {
        if branch_path == get_current_branch_path()? {
            return Err(anyhow!("Unable to checkout: already on that branch"));
        }

        if !branch_path.exists() {
            create(target)?;
        }

        let target_hash = fs::read_to_string(branch_path)?;
        let current_index = Index::load()?;
        let mut target_index = Index::restore_from_commit(&target_hash)?;

        update_working_tree(&mut target_index, &current_index)?;
        target_index.store()?;

        let root = get_repository_root()?;
        fs::write(root.join(".rgit/HEAD"), format!("ref: refs/heads/{target}"))?;
    }

    Ok(())
}
