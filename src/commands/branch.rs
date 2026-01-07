use std::fs;

use anyhow::{Result, anyhow};

use crate::utils::{get_current_branch_name, get_repository_root};

pub fn list() -> Result<Vec<String>> {
    let root = get_repository_root()?;
    let mut branches = Vec::new();
    let current_branch = get_current_branch_name()?;
    branches.push(format!("{current_branch} -> HEAD"));

    for entry in root.join(".rgit/refs/heads").read_dir()?.flatten() {
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow!("Invalid UTF-8"))?;
        if file_name != current_branch {
            branches.push(file_name);
        }
    }
    Ok(branches)
}

pub fn create(name: &str) -> Result<()> {
    let root = get_repository_root()?;

    let branch_path = root.join(".rgit/refs/heads").join(name);

    if branch_path.exists() {
        return Err(anyhow!("Branch already exists : '{name}'"));
    }

    if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
        let path = get_repository_root()?.join(".rgit").join(head);
        let commit_hash =
            fs::read_to_string(path).map_err(|_| anyhow!("Branch not created: no commits yet"))?;
        fs::write(branch_path, commit_hash)?;
    }

    Ok(())
}

pub fn delete(name: &str) -> Result<()> {
    let root = get_repository_root()?;

    let branch_path = root.join(".rgit/refs/heads").join(name);
    if !branch_path.exists() {
        return Err(anyhow!("Branch doest not exist: '{name}'"));
    }

    if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
        let path = get_repository_root()?.join(".rgit").join(head);
        if path == branch_path {
            return Err(anyhow!("Unable to delete the current branch"));
        } else {
            fs::remove_file(branch_path)?;
        }
    }

    Ok(())
}
