use anyhow::Result;

use crate::utils::{get_current_branch_name, get_staged_changes, get_unstaged_changes};

pub fn status() -> Result<String> {
    let mut status = String::new();
    let staged = get_staged_changes()?;
    let unstaged = get_unstaged_changes()?;

    let current_branch = get_current_branch_name()?;
    status.push_str(format!("On branch {current_branch}\n\n").as_str());

    if !(staged.0.is_empty() && staged.1.is_empty() && staged.2.is_empty()) {
        status.push_str("Staged changes:\n");
    }
    for change in staged.0 {
        if let Some(change) = change.to_str() {
            status.push_str(format!("{:<11}{change}\n", "Added:").as_str());
        }
    }
    for change in staged.1 {
        if let Some(change) = change.to_str() {
            status.push_str(format!("{:<11}{change}\n", "Deleted:").as_str());
        }
    }
    for change in staged.2 {
        if let Some(change) = change.to_str() {
            status.push_str(format!("{:<11}{change}\n", "Modified:").as_str());
        }
    }

    if !(unstaged.0.is_empty() && unstaged.1.is_empty() && unstaged.2.is_empty()) {
        status.push_str("\nUnstaged changes:\n");
    }
    for change in unstaged.0 {
        if let Some(change) = change.to_str() {
            status.push_str(format!("{:<11}{change}\n", "Added:").as_str());
        }
    }
    for change in unstaged.1 {
        if let Some(change) = change.to_str() {
            status.push_str(format!("{:<11}{change}\n", "Deleted:").as_str());
        }
    }
    for change in unstaged.2 {
        if let Some(change) = change.to_str() {
            status.push_str(format!("{:<11}{change}\n", "Modified:").as_str());
        }
    }

    Ok(status)
}
