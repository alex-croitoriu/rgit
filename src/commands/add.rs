use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::commands::CommandOutput;
use crate::{
    commands::Command,
    state::{Blob, Index, IndexEntry, Object, Repository},
    utils::{modification_time, normalize_path},
};
pub struct AddCommand {
    pub paths: Vec<String>,
}

impl Command for AddCommand {
    // TODO: add detailed feedback on individual added files
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let mut index = repository.load_index()?;
        self.paths.sort();
        self.paths.dedup();

        for path in &self.paths {
            add_recursive(repository, &mut index, path)?;
        }

        repository.store_index(&index)?;
        Ok(CommandOutput::Empty)
    }
}

pub fn add_recursive(repository: &Repository, index: &mut Index, path: &str) -> Result<()> {
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
            .ignored
            .iter()
            .any(|p| relative_path.starts_with(p))
        {
            return Err(anyhow!("Ignored path: '{}'", relative_path.display()));
        }

        let blob = Object::Blob(Blob {
            bytes: fs::read(&absolute_path)?,
        });

        let hash = repository.store_object(&blob)?;

        // TODO: add only if needed
        index.add(
            relative_path,
            IndexEntry {
                hash,
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
                .ignored
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
