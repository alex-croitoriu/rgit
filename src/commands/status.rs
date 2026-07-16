use anyhow::Result;

use crate::{
    commands::{self, FileDiff},
    state::{Head, Repository},
    utils::unstaged_changes,
};

pub struct Command;

pub struct Output {
    head: Head,
    staged: FileDiff,
    unstaged: FileDiff,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Head::Branch { name } => write!(f, "On branch: {name}")?,
            Head::Commit { hash } => write!(f, "Detached HEAD: {hash}")?,
        }

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

    fn execute(repository: &Repository, (): ()) -> Result<Self::Output> {
        let head = repository.head()?;
        let staged = repository.staged_changes()?;
        let unstaged = unstaged_changes(repository)?;

        Ok(Output {
            head,
            staged,
            unstaged,
        })
    }
}
