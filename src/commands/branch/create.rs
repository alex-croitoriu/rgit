use std::fs;

use anyhow::{Result, anyhow};

use crate::{commands, state::Repository};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    pub name: String,
}

pub struct Output {
    pub name: String,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Branch created: '{}'", self.name)?;
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let branch_path = repository.branch_path(&args.name);
        if branch_path.exists() {
            return Err(anyhow!(
                "Branch not created: '{}' already exists",
                args.name
            ));
        }

        let head = repository.current_branch_path()?;
        let commit_hash = fs::read_to_string(head)
            .map_err(|_| anyhow!("Branch not created: no commits on HEAD yet"))?;
        fs::write(branch_path, commit_hash)?;

        Ok(Output { name: args.name })
    }
}
