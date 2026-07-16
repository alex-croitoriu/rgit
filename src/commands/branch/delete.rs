use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Head, Repository},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    name: String,
}

pub struct Output {
    name: String,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Branch deleted: '{}'", self.name)?;
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let branch_path = repository.branch_path(&args.name);
        if !branch_path.exists() {
            return Err(anyhow!(
                "Branch not deleted: '{}' does not exist",
                args.name
            ));
        }

        if let Head::Branch { name } = repository.head()?
            && name == args.name
        {
            return Err(anyhow!(
                "Branch not deleted: '{}' is the current branch",
                args.name
            ));
        }

        fs::remove_file(branch_path)?;

        Ok(Output { name: args.name })
    }
}
