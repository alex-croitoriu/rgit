use std::{fs, path::Path};

use anyhow::{Result, anyhow};

use crate::{
    state::Object,
    utils::{branch_path, head_file_path, trimmed_file_content},
};

pub enum Head {
    Branch { name: String, hash: Option<String> },
    Commit { hash: String },
}

pub enum Target {
    Branch { name: String, hash: String },
    Commit { hash: String },
}

impl Head {
    pub fn hash(&self) -> Option<String> {
        match self {
            Self::Branch { hash, .. } => hash.clone(),
            Self::Commit { hash } => Some(hash.clone()),
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
            Ok(Self::Commit { hash: content })
        } else {
            Err(anyhow!("Unable to resolve HEAD"))
        }
    }

    pub fn update(root: &Path, target: &Target) -> Result<()> {
        match target {
            Target::Branch { name, .. } => {
                fs::write(head_file_path(root), format!("ref: refs/heads/{name}"))?;
            }
            Target::Commit { hash } => {
                fs::write(head_file_path(root), hash)?;
            }
        }

        Ok(())
    }

    pub fn advance(&mut self, root: &Path, commit_hash: &str) -> Result<()> {
        match self {
            Self::Branch { name, hash } => {
                let branch_path = branch_path(root, name);
                fs::write(branch_path, commit_hash)?;
                *hash = Some(commit_hash.to_string());
            }
            Self::Commit { hash } => {
                fs::write(head_file_path(root), commit_hash)?;
                *hash = commit_hash.to_string();
            }
        }

        Ok(())
    }
}

impl Target {
    pub fn hash(&self) -> String {
        match self {
            Self::Branch { hash, .. } | Self::Commit { hash } => hash.clone(),
        }
    }

    pub fn resolve(root: &Path, target: &str) -> Result<Self> {
        let branch_path = branch_path(root, target);

        if branch_path.exists() {
            let hash = trimmed_file_content(&branch_path)?;
            Ok(Self::Branch {
                name: target.to_string(),
                hash,
            })
            // TODO: make this lazy
        } else if let Ok(Object::Commit(_)) = Object::load(root, target) {
            Ok(Self::Commit {
                hash: target.to_string(),
            })
        } else {
            Err(anyhow!("target '{target}' is invalid"))
        }
    }
}
