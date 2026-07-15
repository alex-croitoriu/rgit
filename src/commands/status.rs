use anyhow::Result;

use crate::{
    commands::{self, FileDiff},
    state::Repository,
    utils::{get_unstaged_changes},
};

pub struct Command;

// TODO: refactor for detached head
pub struct Output {
     branch: String,
     staged: FileDiff,
     unstaged: FileDiff,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "On branch {}", self.branch)?;

        if !self.staged.is_empty() {
            write!(f, "\n\nStaged changes:")?;

            for change in &self.staged.added {
                if let Some(change) = change.to_str() {
                    write!(f, "\n{:<11}{change}", "Added:")?;
                }
            }
            for change in &self.staged.deleted {
                if let Some(change) = change.to_str() {
                    write!(f, "\n{:<11}{change}", "Deleted:")?;
                }
            }
            for change in &self.staged.modified {
                if let Some(change) = change.to_str() {
                    write!(f, "\n{:<11}{change}", "Modified:")?;
                }
            }
        }

        if !self.unstaged.is_empty() {
            write!(f, "\n\nUnstaged changes:")?;

            for change in &self.unstaged.added {
                if let Some(change) = change.to_str() {
                    write!(f, "\n{:<11}{change}", "Added:")?;
                }
            }
            for change in &self.unstaged.deleted {
                if let Some(change) = change.to_str() {
                    write!(f, "\n{:<11}{change}", "Deleted:")?;
                }
            }
            for change in &self.unstaged.modified {
                if let Some(change) = change.to_str() {
                    write!(f, "\n{:<11}{change}", "Modified:")?;
                }
            }
        }
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = ();
    type Output = Output;

    fn execute(repository: &Repository, _: Self::Args) -> Result<Self::Output> {
        let head = repository.current_branch_name()?;
        let staged = repository.staged_changes()?;
        let unstaged = get_unstaged_changes(repository)?;

        Ok(Output {
            branch: head,
            staged,
            unstaged,
        })
    }
}
