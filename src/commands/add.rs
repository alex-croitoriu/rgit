use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::{
    state::{Blob, Index, IndexEntry, Object},
    utils::{get_ignored, get_mtime, get_repository_root, normalize_path},
};

// TODO: add detailed feedback on individual added files
pub fn add(paths: Vec<String>) -> Result<()> {
    let mut index = Index::load()?;
    let mut paths = paths;

    paths.sort();
    paths.dedup();

    for path in &paths {
        add_recursive(&mut index, path)?;
    }

    index.store()?;

    Ok(())
}

pub fn add_recursive(index: &mut Index, path: &str) -> Result<()> {
    let root = get_repository_root()?;
    let absolute_path = normalize_path(&env::current_dir()?.join(path));
    let relative_path = absolute_path.strip_prefix(&root)?;

    // TODO: cache within a repository global state
    let ignored = get_ignored();

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
        if let Some(ignored) = ignored
            && ignored.iter().any(|p| relative_path.starts_with(p))
        {
            return Err(anyhow!("Ignored path: '{}'", relative_path.display()));
        }

        let blob = Object::Blob(Blob {
            content: fs::read(&absolute_path)?,
        });

        let hash = blob.store()?;

        // TODO: add only if needed
        index.add(
            relative_path,
            IndexEntry {
                hash,
                size: absolute_path.metadata()?.len(),
                mtime: get_mtime(&absolute_path)?,
            },
        )?;
    } else if absolute_path.is_dir() {
        let to_remove = index
            .entries
            .keys()
            .filter(|p| p.starts_with(relative_path) && !root.join(p).exists())
            .cloned()
            .collect::<Vec<PathBuf>>();
        for path in to_remove {
            index.remove(&path)?;
        }

        for entry in absolute_path.read_dir()?.flatten() {
            let entry_path = entry.path();
            let relative_entry_path = entry_path.strip_prefix(&root)?;
            if relative_entry_path == ".rgit" {
                continue;
            }
            if let Some(ignored) = &ignored
                && ignored.iter().any(|p| relative_entry_path.starts_with(p))
            {
                continue;
            }
            add_recursive(index, entry.path().to_str().ok_or(anyhow!("Invalid path"))?)?;
        }
    } else {
        return Err(anyhow!("Invalid path: '{}'", relative_path.display()));
    }
    Ok(())
}
