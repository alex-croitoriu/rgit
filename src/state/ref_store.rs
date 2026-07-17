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
    Branch { name: String },
    Commit { hash: String },
}

pub fn head(root: &Path) -> Result<Head> {
    let content = fs::read_to_string(head_file_path(root))?;

    if let Some(head) = content.strip_prefix("ref: ") {
        let branch = PathBuf::from(head)
            .file_name()
            .ok_or(anyhow!("Current branch not found"))?
            .to_string_lossy()
            .to_string();
        Ok(Head::Branch { name: branch })
    } else if let Ok(Object::Commit(_)) = Object::load(root, &content) {
        Ok(Head::Commit { hash: content })
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}

pub fn head_hash(root: &Path) -> Result<Option<String>> {
    let content = fs::read_to_string(head_file_path(root))?;

    if let Some(head) = content.strip_prefix("ref: ") {
        let path = root.join(".rgit").join(head);
        if path.exists() {
            let hash = fs::read_to_string(path)?;
            Ok(Some(hash))
        } else if let Some(file) = path.file_name()
            && file.to_string_lossy() == "master"
        {
            Ok(None)
        } else {
            Err(anyhow!("Corrupt HEAD file"))
        }
    } else if let Ok(Object::Commit(_)) = Object::load(root, &content) {
        Ok(Some(content))
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}

pub fn update_head(root: &Path, target: &Head) -> Result<()> {
    match target {
        Head::Branch { name } => {
            fs::write(head_file_path(root), format!("ref: refs/heads/{name}"))?;
        }
        Head::Commit { hash } => {
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
