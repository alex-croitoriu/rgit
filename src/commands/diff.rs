use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Diff, Head, Index, Repository, branch_path, diff_indexes},
    utils::trimmed_file_content,
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    target: Option<String>,
}

pub struct Output {
    diff: Diff,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diff)
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repo: &Repository, args: Self::Args) -> Result<Self::Output> {
        let head_index = if let Some(hash) = Head::load(&repo.root)?.hash() {
            Index::load_from_commit(&repo.root, &hash)?
        } else {
            Index::new()
        };

        if let Some(target) = &args.target {
            let branch_path = branch_path(&repo.root, target);
            if !branch_path.exists() {
                return Err(anyhow!("Branch does not exist: '{target}'"));
            }

            let branch_hash = trimmed_file_content(&branch_path)?;
            let branch_index = Index::load_from_commit(&repo.root, &branch_hash)?;

            let diff = diff_indexes(&repo.root, &head_index, &branch_index)?;

            Ok(Output { diff })
        } else {
            let current_index = Index::load(&repo.root)?;
            let diff = diff_indexes(&repo.root, &head_index, &current_index)?;

            Ok(Output { diff })
        }
    }
}
