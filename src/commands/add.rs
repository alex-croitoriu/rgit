use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::index::{Index, IndexEntry};
use crate::object_store::{Blob, Object};
use crate::utils::get_ignored;
use crate::utils::get_mtime;
use crate::utils::{get_repository_root, normalize_path};
use base64::{Engine as _, engine::general_purpose};

pub fn add(path: &str) -> Result<()> {
    let root = get_repository_root()?;
    let absolute_path = normalize_path(&env::current_dir()?.join(path));
    let relative_path = absolute_path.strip_prefix(&root)?;
    let mut index = Index::load()?;
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
        } else {
            for path in to_remove {
                index.remove(&path)?;
            }
            index.store()?;
            return Ok(());
        }
    }

    if absolute_path.is_file() {
        if let Some(ignored) = ignored
            && ignored.iter().any(|p| relative_path.starts_with(p))
        {
            return Err(anyhow!("Ignored path: '{}'", relative_path.display()));
        }

        let blob = Object::Blob(Blob {
            content: general_purpose::STANDARD.encode(fs::read(&absolute_path)?),
        });

        let hash = blob.store()?;
        index.add(
            relative_path,
            IndexEntry {
                hash,
                size: absolute_path.metadata()?.len(),
                mtime: get_mtime(&absolute_path)?,
            },
        )?;
        index.store()?;
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
        index.store()?;

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
            add(entry.path().to_str().ok_or(anyhow!("Invalid path"))?)?;
        }
    } else {
        return Err(anyhow!("Invalid path: '{}'", relative_path.display()));
    }
    Ok(())
}
