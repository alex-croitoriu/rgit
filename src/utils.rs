use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use anyhow::Result;

use crate::state::{FileDiff, Index, Repository};

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component),
        }
    }
    result
}

pub fn modification_time(path: &Path) -> Result<u64> {
    Ok(path
        .metadata()?
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub fn unstaged_changes(repository: &Repository) -> Result<FileDiff> {
    let mut diff = FileDiff {
        added: Vec::new(),
        deleted: Vec::new(),
        modified: Vec::new(),
    };

    let index = repository.load_index()?;
    let mut stack = vec![repository.root.clone()];

    while !stack.is_empty() {
        if let Some(path) = stack.pop() {
            if path.is_file() {
                let relative_path = path.strip_prefix(&repository.root)?;
                if let Some(entry) = index.entries.get(relative_path) {
                    if entry.size != path.metadata()?.len()
                        || entry.mtime != modification_time(&path)?
                    {
                        diff.modified.push(relative_path.to_path_buf());
                    }
                } else {
                    diff.added.push(relative_path.to_path_buf());
                }
            } else if path.is_dir() {
                for entry in path.read_dir()?.flatten() {
                    let entry_path = entry.path();
                    let relative_path = entry_path.strip_prefix(&repository.root)?;

                    if relative_path == ".rgit" {
                        continue;
                    }
                    if repository
                        .ignored()
                        .iter()
                        .any(|p| relative_path.starts_with(p))
                    {
                        continue;
                    }

                    if entry.file_type()?.is_file() {
                        stack.push(entry_path);
                    } else if entry.file_type()?.is_dir() {
                        if index
                            .entries
                            .iter()
                            .any(|(name, _)| PathBuf::from(&name).starts_with(relative_path))
                        {
                            stack.push(entry_path);
                        } else if entry_path.read_dir()?.count() > 0 {
                            diff.added.push(relative_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    for (name, _) in index.entries {
        if !repository.root.join(&name).exists() {
            diff.deleted.push(name);
        }
    }

    Ok(diff)
}

pub fn update_working_tree(
    repository: &Repository,
    index: &mut Index,
    old_index: &Index,
) -> Result<()> {
    for path in old_index.entries.keys() {
        if !index.entries.contains_key(path) {
            let absolute_path = repository.root.join(path);
            if absolute_path.exists() {
                fs::remove_file(absolute_path)?;
            }
        }
    }

    for (path, entry) in &mut index.entries {
        let bytes = repository.load_blob_bytes(&entry.hash)?;
        let absolute_path = repository.root.join(path);
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&absolute_path, bytes)?;

        entry.mtime = modification_time(&absolute_path)?;
        entry.size = fs::metadata(&absolute_path)?.len();
    }

    Ok(())
}
