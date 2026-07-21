use anyhow::Result;

use crate::{
    commands,
    state::{Changes, Head, Repository, staged_changes, unstaged_changes},
};

pub struct Command;

pub struct Output {
    head: Head,
    staged: Changes,
    unstaged: Changes,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Head::Branch { name, .. } => write!(f, "On branch: {name}")?,
            Head::Commit { hash } => write!(f, "Detached HEAD: {hash}")?,
        }
        if !self.staged.is_empty() {
            write!(f, "\n\nStaged changes:")?;
            write!(f, "{}", self.staged)?;
        }
        if !self.unstaged.is_empty() {
            write!(f, "\n\nUnstaged changes:")?;
            write!(f, "{}", self.unstaged)?;
        }

        Ok(())
    }
}

impl commands::Command for Command {
    type Args = ();
    type Output = Output;

    fn execute(repo: &Repository, (): ()) -> Result<Self::Output> {
        let head = Head::load(&repo.root)?;
        let staged = staged_changes(&repo.root)?;
        let unstaged = unstaged_changes(&repo.root)?;

        Ok(Output {
            head,
            staged,
            unstaged,
        })
    }
}
