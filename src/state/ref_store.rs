use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::{
    state::Object,
    utils::{head_file_path, heads_dir_path, trimmed_file_content},
};

pub enum Head {
    Branch { name: String, hash: Option<String> },
    Detached { hash: String },
}

impl Head {
    pub fn hash(&self) -> Option<String> {
        match self {
            Self::Branch { hash, .. } => hash.clone(),
            Self::Detached { hash } => Some(hash.clone()),
        }
    }

    pub fn load(root: &Path) -> Result<Self> {
        let content = trimmed_file_content(&head_file_path(root))?;

        if let Some(head) = content.strip_prefix("ref: ") {
            let path = root.join(".rgit").join(head);
            let name = path
                .file_name()
                .ok_or(anyhow!("Unable to resolve HEAD"))?
                .to_str()
                .ok_or(anyhow!("Unable to resolve HEAD"))?
                .to_string();
            if path.exists() {
                let hash = trimmed_file_content(&path)?;
                Ok(Self::Branch {
                    name,
                    hash: Some(hash),
                })
            } else {
                Ok(Self::Branch { name, hash: None })
            }
        } else if let Ok(Object::Commit(_)) = Object::load(root, &content) {
            Ok(Self::Detached { hash: content })
        } else {
            Err(anyhow!("Unable to resolve HEAD"))
        }
    }

    pub fn store(&self, root: &Path) -> Result<()> {
        match self {
            Self::Branch { name, .. } => {
                fs::write(head_file_path(root), format!("ref: refs/heads/{name}"))?;
            }
            Self::Detached { hash } => {
                fs::write(head_file_path(root), hash)?;
            }
        }

        Ok(())
    }
}

pub fn branch_path(root: &Path, name: &str) -> PathBuf {
    heads_dir_path(root).join(name)
}
