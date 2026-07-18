use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Blob, Index, IndexEntry, Object, Repository, ignored_paths},
    utils::{file_mtime, file_size, normalize_path},
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
        let ignored_paths = ignored_paths(&repo.root);
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
            add_recursive(&repo.root, path, &ignored_paths, &mut index)?;
        }

        index.store(&repo.root)?;

        Ok(Output)
    }
}

fn validate_path(root: &Path, path: &Path, index: &Index) -> Result<()> {
    let relative_path = path
        .strip_prefix(root)
        .map_err(|_| anyhow!("path '{}' is outside repository", path.display()))?;

    if !path.exists() {
        let indexed = index.entries.keys().any(|p| p.starts_with(relative_path));
        if !indexed {
            return Err(anyhow!("path '{}' is invalid", path.display()));
        }
        return Ok(());
    }

    if path.is_file() || path.is_dir() {
        Ok(())
    } else {
        Err(anyhow!("path '{}' is invalid", path.display()))
    }
}

fn add_recursive(
    root: &Path,
    path: &Path,
    ignored_paths: &[PathBuf],
    index: &mut Index,
) -> Result<()> {
    let relative_path = path.strip_prefix(root)?;

    if !path.exists() {
        let to_remove = index
            .entries
            .keys()
            .filter(|p| p.starts_with(relative_path))
            .cloned()
            .collect::<Vec<PathBuf>>();

        for path in to_remove {
            index.remove(&path)?;
        }

        return Ok(());
    }

    if path.is_file() {
        if ignored_paths.iter().any(|p| relative_path.starts_with(p)) {
            return Ok(());
        }

        let blob = Object::Blob(Blob {
            bytes: fs::read(path)?,
        });

        let hash = blob.store(root)?;

        index.add(
            relative_path,
            IndexEntry {
                hash,
                size: file_size(path)?,
                mtime: file_mtime(path)?,
            },
        );
    } else if path.is_dir() {
        let to_remove = index
            .entries
            .keys()
            .filter(|p| p.starts_with(relative_path) && !root.join(p).exists())
            .cloned()
            .collect::<Vec<PathBuf>>();
        for path in to_remove {
            index.remove(&path)?;
        }

        for entry in path.read_dir()?.flatten() {
            let entry_path = entry.path();
            let relative_entry_path = entry_path.strip_prefix(root)?;
            if relative_entry_path == ".rgit" {
                continue;
            }
            if ignored_paths
                .iter()
                .any(|p| relative_entry_path.starts_with(p))
            {
                continue;
            }
            add_recursive(root, &entry_path, ignored_paths, index)?;
        }
    }

    Ok(())
}
