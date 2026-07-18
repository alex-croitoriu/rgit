use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Index, Repository},
    utils::normalize_path,
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true)]
    paths: Vec<String>,
}

pub struct Output;

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index updated")
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repo: &Repository, args: Self::Args) -> Result<Self::Output> {
        let mut index = Index::load(&repo.root)?;
        let current_dir = &env::current_dir()?;

        let mut paths = args
            .paths
            .iter()
            .map(|p| normalize_path(&current_dir.join(p)))
            .collect::<Vec<PathBuf>>();

        paths.sort_unstable();
        paths.dedup();

        for path in &paths {
            validate_path(&repo.root, path, &index)
                .map_err(|e| anyhow!("Index not updated: {e}"))?;
        }

        for path in &paths {
            let relative_path = path.strip_prefix(&repo.root)?;

            let to_remove = index
                .entries
                .keys()
                .filter(|p| p.starts_with(relative_path))
                .cloned()
                .collect::<Vec<PathBuf>>();

            for path in to_remove {
                index.remove(&path)?;
            }
        }

        index.store(&repo.root)?;

        Ok(Output)
    }
}

fn validate_path(root: &Path, path: &Path, index: &Index) -> Result<()> {
    let relative_path = path
        .strip_prefix(root)
        .map_err(|_| anyhow!("path '{}' is outside repository", path.display()))?;

    let indexed = index.entries.keys().any(|p| p.starts_with(relative_path));
    if !indexed {
        return Err(anyhow!("path '{}' is not indexed", path.display()));
    }

    Ok(())
}
