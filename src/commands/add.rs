use std::env;
use std::fs;

use anyhow::{Result, anyhow};

use crate::index::*;
use crate::object_store::*;
use crate::utils::get_mtime;
use crate::utils::{get_repository_root, normalize_path};

pub fn add(path: &str) -> Result<()> {
    let root = get_repository_root()?;
    let absolute_path = normalize_path(&env::current_dir()?.join(path));

    if !absolute_path.exists() {
        return Err(anyhow!("Path '{}' does not exist", absolute_path.display()));
    }

    if absolute_path.is_file() {
        let blob = Object::Blob(Blob {
            content: fs::read_to_string(&absolute_path)?,
        });

        let hash = blob.store()?;

        Index::add(
            absolute_path
                .strip_prefix(&root)?
                .to_str()
                .ok_or(anyhow!("Invalid path"))?
                .to_string(),
            IndexEntry {
                hash,
                size: absolute_path.metadata()?.len(),
                mtime: get_mtime(&absolute_path)?,
            },
        )?;
    } else if absolute_path.is_dir() {
        for entry in absolute_path.read_dir()?.flatten() {
            if entry.path().ends_with(".rgit") {
                continue;
            }

            add(entry.path().to_str().ok_or(anyhow!("Invalid path"))?)?;
        }
    }
    Ok(())
}
