use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    args::{Commands, BranchSubcommands},
    commands,
    dispatch::dispatch,
    state::Repository,
    utils::{get_unstaged_changes, update_working_tree},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    target: String,
}

pub struct Output {
    message: String,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    // TODO: add checkout on commits
    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = repository.staged_changes()?;
        let unstaged_changes = get_unstaged_changes(repository)?;

        if !staged_changes.is_empty() || !unstaged_changes.is_empty() {
            return Err(anyhow!("Unable to checkout: uncommited changes"));
        }

        let branch_path = repository.branch_path(&args.target);
        if branch_path == repository.current_branch_path()? {
            return Err(anyhow!("Unable to checkout: already on '{}'", args.target));
        }

        if !branch_path.exists() {
            dispatch(Commands::Branch(BranchSubcommands::Create(
                commands::branch::create::Args {
                    name: args.target.clone(),
                },
            )))?;
        }

        let target_hash = fs::read_to_string(branch_path)?;
        let current_index = repository.load_index()?;
        let mut target_index = repository.load_index_from_commit(&target_hash)?;

        update_working_tree(repository, &mut target_index, &current_index)?;
        repository.store_index(&target_index)?;

        fs::write(
            repository.head_path(),
            format!("ref: refs/heads/{}", args.target),
        )?;

        Ok(Output {
            message: format!("Switched to: '{}'", args.target),
        })
    }
}
