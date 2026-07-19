use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    state::{Head, Index, ignored_paths},
    utils::{file_mtime, file_size},
};

#[derive(Default)]
pub struct Changes {
    pub added: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.deleted.is_empty() && self.modified.is_empty()
    }
}

impl std::fmt::Display for Changes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for change in &self.added {
            if let Some(change) = change.to_str() {
                write!(f, "\n{:<11}{change}", "Added:")?;
            }
        }
        for change in &self.deleted {
            if let Some(change) = change.to_str() {
                write!(f, "\n{:<11}{change}", "Deleted:")?;
            }
        }
        for change in &self.modified {
            if let Some(change) = change.to_str() {
                write!(f, "\n{:<11}{change}", "Modified:")?;
            }
        }

        Ok(())
    }
}

pub fn staged_changes(root: &Path) -> Result<Changes> {
    let mut changes = Changes::default();

    let current_index = Index::load(root)?;
    let head_index = if let Some(head_hash) = Head::load(root)?.hash() {
        Index::load_from_commit(root, &head_hash)?
    } else {
        Index::default()
    };

    for (name, index_entry) in &current_index.entries {
        if let Some(head_entry) = head_index.entries.get(name) {
            if index_entry.hash != head_entry.hash {
                changes.modified.push(name.clone());
            }
        } else {
            changes.added.push(name.clone());
        }
    }

    for (name, _) in head_index.entries {
        if !current_index.entries.contains_key(&name) {
            changes.deleted.push(name);
        }
    }

    Ok(changes)
}

pub fn unstaged_changes(root: &Path) -> Result<Changes> {
    let mut changes = Changes::default();

    let index = Index::load(root)?;
    let mut stack = vec![root.to_path_buf()];
    let ignored_paths = ignored_paths(root);

    while let Some(path) = stack.pop() {
        if path.is_file() {
            let relative_path = path.strip_prefix(root)?;
            if let Some(entry) = index.entries.get(relative_path) {
                if entry.size != file_size(&path)? || entry.mtime != file_mtime(&path)? {
                    changes.modified.push(relative_path.to_path_buf());
                }
            } else {
                changes.added.push(relative_path.to_path_buf());
            }
        } else if path.is_dir() {
            for entry in path.read_dir()?.flatten() {
                let entry_path = entry.path();
                let relative_path = entry_path.strip_prefix(root)?;

                if relative_path == ".rgit" {
                    continue;
                }
                if ignored_paths.iter().any(|p| relative_path.starts_with(p)) {
                    continue;
                }

                if entry.file_type()?.is_file() {
                    stack.push(entry_path);
                } else if entry.file_type()?.is_dir() {
                    if index
                        .entries
                        .iter()
                        .any(|(name, _)| name.starts_with(relative_path))
                    {
                        stack.push(entry_path);
                    } else if entry_path.read_dir()?.next().is_some() {
                        changes.added.push(relative_path.to_path_buf());
                    }
                }
            }
        }
    }

    for (name, _) in index.entries {
        if !root.join(&name).exists() {
            changes.deleted.push(name);
        }
    }

    Ok(changes)
}
