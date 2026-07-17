use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::{
    state::Object,
    utils::{head_file_path, heads_dir_path, normalize_path},
};

pub enum Head {
    Branch { name: String, hash: Option<String> },
    Detached { hash: String },
}

pub fn resolve_head(root: &Path) -> Result<Head> {
    let content = fs::read_to_string(head_file_path(root))?;

    if let Some(head) = content.strip_prefix("ref: ") {
        let path = root.join(".rgit").join(head);
        let name = path
            .file_name()
            .ok_or(anyhow!("Unable to resolve HEAD"))?
            .to_str()
            .ok_or(anyhow!("Unable to resolve HEAD"))?
            .to_owned();
        if path.exists() {
            let hash = fs::read_to_string(path)?;
            Ok(Head::Branch {
                name,
                hash: Some(hash),
            })
        } else {
            Ok(Head::Branch { name, hash: None })
        }
    } else if let Ok(Object::Commit(_)) = Object::load(root, &content) {
        Ok(Head::Detached { hash: content })
    } else {
        Err(anyhow!("Unable to resolve HEAD"))
    }
}

pub fn resolve_head_hash(root: &Path) -> Result<Option<String>> {
    match resolve_head(root)? {
        Head::Branch { hash, .. } => Ok(hash),
        Head::Detached { hash } => Ok(Some(hash)),
    }
}

pub fn update_head(root: &Path, target: &Head) -> Result<()> {
    match target {
        Head::Branch { name, .. } => {
            fs::write(head_file_path(root), format!("ref: refs/heads/{name}"))?;
        }
        Head::Detached { hash } => {
            fs::write(head_file_path(root), hash)?;
        }
    }

    Ok(())
}

pub fn branch_path(root: &Path, name: &str) -> PathBuf {
    heads_dir_path(root).join(name)
}

pub fn current_branch_path(root: &Path) -> Result<PathBuf> {
    let content = fs::read_to_string(head_file_path(root))?;

    if let Some(head) = content.strip_prefix("ref: ") {
        let path = normalize_path(&root.join(".rgit").join(head));
        Ok(path)
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}
