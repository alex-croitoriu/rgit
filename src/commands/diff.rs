use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Diff, Head, Index, Repository, Target, diff_indexes},
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
            Index::default()
        };

        if let Some(target) = &args.target {
            let target = Target::resolve(&repo.root, target).map_err(|e| anyhow!("Failed: {e}"))?;
            let target_index = Index::load_from_commit(&repo.root, &target.hash())?;
            let diff = diff_indexes(&repo.root, &head_index, &target_index)?;

            Ok(Output { diff })
        } else {
            let current_index = Index::load(&repo.root)?;
            let diff = diff_indexes(&repo.root, &head_index, &current_index)?;

            Ok(Output { diff })
        }
    }
}
