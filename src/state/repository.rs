use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::state::{FileDiff, Index, head_hash};

pub struct Repository {
    pub root: PathBuf,
}

impl Repository {
    pub fn is_valid_root(path: &Path) -> bool {
        path.join(".rgit/objects").is_dir()
            && path.join(".rgit/refs/heads").is_dir()
            && path.join(".rgit/index").is_file()
            && path.join(".rgit/HEAD").is_file()
    }

    pub fn load() -> Result<Self> {
        let mut root = env::current_dir()?;
        while !Self::is_valid_root(&root) {
            if !root.pop() {
                return Err(anyhow!("Repository not found"));
            }
        }

        Ok(Self { root })
    }

    pub fn staged_changes(&self) -> Result<FileDiff> {
        let mut diff = FileDiff::default();

        let current_index = Index::load(&self.root)?;
        let head_index = if let Some(head_hash) = head_hash(&self.root)? {
            Index::load_from_commit(&self.root, &head_hash)?
        } else {
            Index::new()
        };

        for (name, index_entry) in &current_index.entries {
            if let Some(head_entry) = head_index.entries.get(name) {
                if index_entry.hash != head_entry.hash {
                    diff.modified.push(name.clone());
                }
            } else {
                diff.added.push(name.clone());
            }
        }

        for (name, _) in head_index.entries {
            if !current_index.entries.contains_key(&name) {
                diff.deleted.push(name);
            }
        }

        Ok(diff)
    }
}
