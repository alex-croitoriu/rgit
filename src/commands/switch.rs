use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{
        Head, Index, Object, Repository, branch_path, resolve_head, staged_changes, unstaged_changes,
        update_head, update_working_tree,
    },
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
            Head::Branch { name, .. } => write!(f, "Switched to branch: {name}"),
            Head::Detached { hash } => write!(f, "Switched to commit: {hash}"),
        }
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = staged_changes(&repository.root)?;
        let unstaged_changes = unstaged_changes(&repository.root)?;

        if !staged_changes.is_empty() || !unstaged_changes.is_empty() {
            return Err(anyhow!("Unable to switch: uncommited changes"));
        }

        if let Head::Branch { name, .. } = resolve_head(&repository.root)?
            && name == args.target
        {
            return Err(anyhow!("Unable to switch: already on '{}'", args.target));
        }

        let target_branch_path = branch_path(&repository.root, &args.target);
        let target_hash;
        let target_head;

        if target_branch_path.exists() {
            target_hash = fs::read_to_string(target_branch_path)?;
            target_head = Head::Branch {
                name: args.target.clone(),
                hash: Some(target_hash.clone())
            };
        } else if let Ok(Object::Commit(_)) = Object::load(&repository.root, &args.target) {
            target_hash = args.target.clone();
            target_head = Head::Detached {
                hash: args.target.clone(),
            };
        } else {
            return Err(anyhow!(
                "Unable to switch: target '{}' is invalid",
                args.target
            ));
        }

        let current_index = Index::load(&repository.root)?;
        let mut target_index = Index::load_from_commit(&repository.root, &target_hash)?;

        update_working_tree(&repository.root, &mut target_index, &current_index)?;
        target_index.store(&repository.root)?;

        update_head(&repository.root, &target_head)?;

        Ok(Output { head: target_head })
    }
}
