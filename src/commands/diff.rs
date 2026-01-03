use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    index::Index,
    utils::{diff_indices, get_branch_path, get_current_branch_path},
};

pub fn diff(target: Option<String>) -> Result<String> {
    let mut output = String::new();
    if let Some(target) = target {
        let branch_path = get_branch_path(&target)?;
        if !branch_path.exists() {
            return Err(anyhow!("Branch does not exist"));
        }

        let branch_hash = fs::read_to_string(branch_path)?;
        let branch_index = Index::restore_from_commit(&branch_hash)?;

        let head_path = get_current_branch_path()?;
        let head_hash = fs::read_to_string(head_path)?;
        let head_index = Index::restore_from_commit(&head_hash)?;

        let diff = diff_indices(&head_index, &branch_index)?;
        for (path, change) in diff.0 {
            output.push_str(&format!("{:<11}{}\n", "Added:", path.display()));
            output.push_str(&change);
        }
        for (path, change) in diff.1 {
            output.push_str(&format!("{:<11}{}\n", "Deleted:", path.display()));
            output.push_str(&change);
        }
        for (path, change) in diff.2 {
            output.push_str(&format!("{:<11}{}\n", "Modified:", path.display()));
            output.push_str(&change);
        }

    } else {
        let head_path = get_current_branch_path()?;

        let head_index = if let Ok(head_hash) = fs::read_to_string(head_path) {
            Index::restore_from_commit(&head_hash)?
        } else {
            Index::new()
        };

        let current_index = Index::load()?;

        let diff = diff_indices(&head_index, &current_index)?;
        for (path, change) in diff.0 {
            output.push_str(&format!("{:<11}{}\n", "Added:", path.display()));
            output.push_str(&change);
        }
        for (path, change) in diff.1 {
            output.push_str(&format!("{:<11}{}\n", "Deleted:", path.display()));
            output.push_str(&change);
        }
        for (path, change) in diff.2 {
            output.push_str(&format!("{:<11}{}\n", "Modified:", path.display()));
            output.push_str(&change);
        }
    }

    Ok(output)
}
