use anyhow::Result;

use crate::{
    commands::{Command, CommandOutput},
    state::Repository,
    utils::{get_staged_changes, get_unstaged_changes},
};

pub struct StatusCommand {}

impl Command for StatusCommand {
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let mut status = String::new();
        let staged = get_staged_changes(repository)?;
        let unstaged = get_unstaged_changes(repository)?;

        let current_branch_name = repository.current_branch_name()?;
        status.push_str(format!("On branch {current_branch_name}").as_str());

        if !(staged.0.is_empty() && staged.1.is_empty() && staged.2.is_empty()) {
            status.push_str("\n\nStaged changes:");
        }
        for change in staged.0 {
            if let Some(change) = change.to_str() {
                status.push_str(format!("\n{:<11}{change}", "Added:").as_str());
            }
        }
        for change in staged.1 {
            if let Some(change) = change.to_str() {
                status.push_str(format!("\n{:<11}{change}", "Deleted:").as_str());
            }
        }
        for change in staged.2 {
            if let Some(change) = change.to_str() {
                status.push_str(format!("\n{:<11}{change}", "Modified:").as_str());
            }
        }

        if !(unstaged.0.is_empty() && unstaged.1.is_empty() && unstaged.2.is_empty()) {
            status.push_str("\n\nUnstaged changes:");
        }
        for change in unstaged.0 {
            if let Some(change) = change.to_str() {
                status.push_str(format!("\n{:<11}{change}", "Added:").as_str());
            }
        }
        for change in unstaged.1 {
            if let Some(change) = change.to_str() {
                status.push_str(format!("\n{:<11}{change}", "Deleted:").as_str());
            }
        }
        for change in unstaged.2 {
            if let Some(change) = change.to_str() {
                status.push_str(format!("\n{:<11}{change}", "Modified:").as_str());
            }
        }

        Ok(CommandOutput::FileDiff(status))
    }
}
