use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Blob, Index, IndexEntry, Object, Repository},
    utils::{modification_time, normalize_path},
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
        write!(f, "Added path(s) to the index")?;
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let mut index = repository.load_index()?;
        let mut paths = args.paths.clone();
        paths.sort_unstable();
        paths.dedup();

        for path in &paths {
            add_recursive(repository, &mut index, path)?;
        }

        repository.store_index(&index)?;
        Ok(Output)
    }
}

fn add_recursive(repository: &Repository, index: &mut Index, path: &str) -> Result<()> {
    let absolute_path = normalize_path(&env::current_dir()?.join(path));
    let relative_path = absolute_path.strip_prefix(&repository.root)?;

    if !absolute_path.exists() {
        let to_remove = index
            .entries
            .keys()
            .filter(|p| p.starts_with(relative_path))
            .cloned()
            .collect::<Vec<PathBuf>>();

        if to_remove.is_empty() {
            return Err(anyhow!("Invalid path: '{}'", relative_path.display()));
        }
        for path in to_remove {
            index.remove(&path)?;
        }
        return Ok(());
    }

    if absolute_path.is_file() {
        if repository
            .ignored()
            .iter()
            .any(|p| relative_path.starts_with(p))
        {
            return Ok(());
        }

        let blob = Object::Blob(Blob {
            bytes: fs::read(&absolute_path)?,
        });

        let hash = repository.store_object(&blob)?;

        index.add(
            relative_path,
            IndexEntry {
                hash: hash.clone(),
                size: absolute_path.metadata()?.len(),
                mtime: modification_time(&absolute_path)?,
            },
        );
    } else if absolute_path.is_dir() {
        let to_remove = index
            .entries
            .keys()
            .filter(|p| p.starts_with(relative_path) && !&repository.root.join(p).exists())
            .cloned()
            .collect::<Vec<PathBuf>>();
        for path in to_remove {
            index.remove(&path)?;
        }

        for entry in absolute_path.read_dir()?.flatten() {
            let entry_path = entry.path();
            let relative_entry_path = entry_path.strip_prefix(&repository.root)?;
            if relative_entry_path == ".rgit" {
                continue;
            }
            if repository
                .ignored()
                .iter()
                .any(|p| relative_entry_path.starts_with(p))
            {
                continue;
            }
            add_recursive(
                repository,
                index,
                entry.path().to_str().ok_or(anyhow!("Invalid path"))?,
            )?;
        }
    } else {
        return Err(anyhow!("Invalid path: '{}'", relative_path.display()));
    }
    Ok(())
}
