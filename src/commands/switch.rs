use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{
        Head, Index, Object, Repository, staged_changes, unstaged_changes, update_working_tree,
    },
    utils::{branch_path, trimmed_file_content},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    target: String,
    /// Discard all changes and switch
    #[arg(short, long)]
    force: bool,
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

    fn execute(repo: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = staged_changes(&repo.root)?;
        let unstaged_changes = unstaged_changes(&repo.root)?;

        if !(staged_changes.is_empty() && unstaged_changes.is_empty()) && !args.force {
            return Err(anyhow!("Unable to switch: uncommited changes"));
        }

        if let Head::Branch { name, .. } = Head::load(&repo.root)?
            && name == args.target
        {
            return Err(anyhow!("Unable to switch: already on '{}'", args.target));
        }

        let target_branch_path = branch_path(&repo.root, &args.target);
        let target_hash;
        let target_head;

        if target_branch_path.exists() {
            target_hash = trimmed_file_content(&target_branch_path)?;
            target_head = Head::Branch {
                name: args.target.clone(),
                hash: Some(target_hash.clone()),
            };
            // TODO: make this lazy
        } else if let Ok(Object::Commit(_)) = Object::load(&repo.root, &args.target) {
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

        let current_index = Index::load(&repo.root)?;
        let mut target_index = Index::load_from_commit(&repo.root, &target_hash)?;

        update_working_tree(&repo.root, &mut target_index, &current_index)?;
        target_index.store(&repo.root)?;

        target_head.store(&repo.root)?;

        Ok(Output { head: target_head })
    }
}
