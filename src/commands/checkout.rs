use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Head, Object, Repository},
    utils::{unstaged_changes, update_working_tree},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    target: String,
}

pub struct Output {
    head: Head,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Head::Branch { name } => write!(f, "Switched to branch: {name}")?,
            Head::Commit { hash } => write!(f, "Switched to commit: {hash}")?,
        }

        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = repository.staged_changes()?;
        let unstaged_changes = unstaged_changes(repository)?;

        if !staged_changes.is_empty() || !unstaged_changes.is_empty() {
            return Err(anyhow!("Unable to checkout: uncommited changes"));
        }

        if let Head::Branch { name } = repository.head()?
            && name == args.target
        {
            return Err(anyhow!("Unable to checkout: already on '{}'", args.target));
        }

        let target_branch_path = repository.branch_path(&args.target);
        let target_hash;
        let target_head;

        if target_branch_path.exists() {
            target_hash = fs::read_to_string(target_branch_path)?;
            target_head = Head::Branch {
                name: args.target.clone(),
            };
        } else if let Ok(Object::Commit(_)) = repository.load_object(&args.target) {
            target_hash = args.target.clone();
            target_head = Head::Commit {
                hash: args.target.clone(),
            };
        } else {
            return Err(anyhow!(
                "Unable to checkout: target '{}' is invalid",
                args.target
            ));
        }

        let current_index = repository.load_index()?;
        let mut target_index = repository.load_index_from_commit(&target_hash)?;

        update_working_tree(repository, &mut target_index, &current_index)?;
        repository.store_index(&target_index)?;

        repository.update_head(&target_head)?;

        Ok(Output { head: target_head })
    }
}
