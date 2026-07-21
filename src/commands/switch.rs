use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{
        Head, Index, Repository, Target, staged_changes, unstaged_changes, update_working_tree,
    },
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
    target: Target,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.target {
            Target::Branch { name, .. } => write!(f, "Switched to branch: {name}"),
            Target::Commit { hash } => write!(f, "Switched to commit: {hash}"),
        }
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repo: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = staged_changes(&repo.root)?;
        let unstaged_changes = unstaged_changes(&repo.root)?;

        if (staged_changes.is_empty() || unstaged_changes.is_empty()) && !args.force {
            return Err(anyhow!("Unable to switch: uncommited changes"));
        }

        match Head::load(&repo.root)? {
            Head::Branch { name, .. } => {
                if name == args.target {
                    return Err(anyhow!(
                        "Unable to switch: already on branch '{}'",
                        args.target
                    ));
                }
            }
            Head::Commit { hash } => {
                if hash == args.target {
                    return Err(anyhow!(
                        "Unable to switch: already on commit {}",
                        args.target
                    ));
                }
                // TODO: handle this (or not)
                // if !args.force {
                //     return Err(anyhow!(
                //         "Unable to switch: commit {hash} will remain unreachable\nYou can create a new branch or use the --force option"
                //     ));
                // }
            }
        }

        let target = Target::resolve(&repo.root, &args.target)
            .map_err(|e| anyhow!("Unable to switch: {e}"))?;

        let current_index = Index::load(&repo.root)?;
        let mut target_index = Index::load_from_commit(&repo.root, &target.hash())?;

        update_working_tree(&repo.root, &mut target_index, &current_index)?;
        target_index.store(&repo.root)?;

        Head::update(&repo.root, &target)?;

        Ok(Output { target })
    }
}
