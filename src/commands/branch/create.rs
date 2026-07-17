use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Repository, branch_path, resolve_head_hash},
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
        write!(f, "Branch created: '{}'", self.name)
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let branch_path = branch_path(&repository.root, &args.name);
        if branch_path.exists() {
            return Err(anyhow!(
                "Branch not created: '{}' already exists",
                args.name
            ));
        }

        if let Some(hash) = resolve_head_hash(&repository.root)? {
            fs::write(branch_path, hash)?;
        } else {
            return Err(anyhow!("Branch not created: no commits on HEAD yet"));
        }

        Ok(Output { name: args.name })
    }
}
