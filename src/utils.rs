use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs};

use anyhow::{Result, anyhow};

use crate::index::Index;

pub fn get_repository_root() -> Result<PathBuf> {
    let mut path = env::current_dir()?;
    loop {
        if is_repository_root(&path) {
            return Ok(path);
        }
        if !path.pop() {
            return Err(anyhow!("Repository not found"));
        }
    }
}

pub fn is_repository_root(path: &Path) -> bool {
    path.join(".rgit/objects").is_dir()
        && path.join(".rgit/refs/heads").is_dir()
        && path.join(".rgit/index").is_file()
        && path.join(".rgit/HEAD").is_file()
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component.as_os_str()),
        }
    }
    result
}

pub fn get_mtime(path: &Path) -> Result<u64> {
    Ok(path
        .metadata()?
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub fn get_current_branch_name() -> Result<String> {
    let root = get_repository_root()?;
    if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
        let current_branch = PathBuf::from(head)
            .file_name()
            .ok_or(anyhow!("Current branch not found"))?
            .to_string_lossy()
            .to_string();
        Ok(current_branch)
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}

pub fn get_current_branch_path() -> Result<PathBuf> {
    let root = get_repository_root()?;
    if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
        let path = normalize_path(&root.join(".rgit").join(head));
        Ok(path)
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}

pub fn get_branch_path(name: &str) -> Result<PathBuf> {
    let root = get_repository_root()?;
    Ok(normalize_path(&root.join(".rgit/refs/heads").join(name)))
}

pub fn get_staged_changes() -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    let index = Index::load()?;
    let head = if let Ok(path) = get_current_branch_path()
        && let Ok(hash) = fs::read_to_string(path)
    {
        Index::restore_from_commit(&hash)?
    } else {
        Index::new()
    };

    for (name, index_entry) in &index.entries {
        if let Some(head_entry) = head.entries.get(name) {
            if index_entry.hash != head_entry.hash {
                modified.push(name.clone());
            }
        } else {
            added.push(name.clone());
        }
    }

    for (name, _) in head.entries {
        if !index.entries.contains_key(&name) {
            deleted.push(name);
        }
    }

    Ok((added, deleted, modified))
}

pub fn get_unstaged_changes() -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    let ignored = get_ignored();
    let index = Index::load()?;
    let root = get_repository_root()?;
    let mut stack = vec![root.clone()];

    while !stack.is_empty() {
        if let Some(path) = stack.pop() {
            if path.is_file() {
                let relative_path = path.strip_prefix(&root)?;
                if let Some(entry) = index.entries.get(relative_path) {
                    if entry.size != path.metadata()?.len() || entry.mtime != get_mtime(&path)? {
                        modified.push(relative_path.to_path_buf());
                    }
                } else {
                    added.push(relative_path.to_path_buf());
                }
            } else if path.is_dir() {
                for entry in path.read_dir()?.flatten() {
                    let entry_path = entry.path();
                    let relative_path = entry_path.strip_prefix(&root)?;

                    if relative_path == ".rgit" {
                        continue;
                    }
                    if let Some(ignored) = &ignored
                        && ignored.iter().any(|p| relative_path.starts_with(p))
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
                            added.push(relative_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    for (name, _) in index.entries {
        if !root.join(&name).exists() {
            deleted.push(name);
        }
    }

    Ok((added, deleted, modified))
}

pub fn get_commit_index_diff(hash: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    let current_index = Index::load()?;
    let commit_index = Index::restore_from_commit(hash)?;

    for (name, index_entry) in &commit_index.entries {
        if let Some(head_entry) = current_index.entries.get(name) {
            if index_entry.hash != head_entry.hash {
                modified.push(name.clone());
            }
        } else {
            added.push(name.clone());
        }
    }

    for (name, _) in current_index.entries {
        if !commit_index.entries.contains_key(&name) {
            deleted.push(name);
        }
    }

    Ok((added, deleted, modified))
}

pub fn get_ignored() -> Option<Vec<PathBuf>> {
    let mut ignored = Vec::new();
    let file = File::open(get_repository_root().ok()?.join(".rgitignore")).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        ignored.push(PathBuf::from(line));
    }
    Some(ignored)
}
