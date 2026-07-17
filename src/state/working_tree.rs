use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    state::{FileDiff, Index, read_blob_bytes},
    utils::{file_mtime, file_size, objects_dir_path},
};

pub fn ignored_paths(root: &Path) -> Vec<PathBuf> {
    let mut ignored = Vec::new();
    if let Ok(file) = File::open(root.join(".rgitignore")) {
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            ignored.push(PathBuf::from(line));
        }
    }

    ignored
}

pub fn unstaged_changes(root: &Path) -> Result<FileDiff> {
    let mut diff = FileDiff::default();

    let index = Index::load(root)?;
    let mut stack = vec![root.to_path_buf()];
    let ignored = ignored_paths(root);

    while !stack.is_empty() {
        if let Some(path) = stack.pop() {
            if path.is_file() {
                let relative_path = path.strip_prefix(root)?;
                if let Some(entry) = index.entries.get(relative_path) {
                    if entry.size != path.metadata()?.len() || entry.mtime != file_mtime(&path)? {
                        diff.modified.push(relative_path.to_path_buf());
                    }
                } else {
                    diff.added.push(relative_path.to_path_buf());
                }
            } else if path.is_dir() {
                for entry in path.read_dir()?.flatten() {
                    let entry_path = entry.path();
                    let relative_path = entry_path.strip_prefix(root)?;

                    if relative_path == ".rgit" {
                        continue;
                    }
                    if ignored.iter().any(|p| relative_path.starts_with(p)) {
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
        if !root.join(&name).exists() {
            diff.deleted.push(name);
        }
    }

    Ok(diff)
}

pub fn update_working_tree(root: &Path, index: &mut Index, old_index: &Index) -> Result<()> {
    for path in old_index.entries.keys() {
        if !index.entries.contains_key(path) {
            let absolute_path = root.join(path);
            if absolute_path.exists() {
                fs::remove_file(absolute_path)?;
            }
        }
    }

    for (path, entry) in &mut index.entries {
        let bytes = read_blob_bytes(&objects_dir_path(root), &entry.hash)?;
        let absolute_path = root.join(path);
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&absolute_path, bytes)?;

        entry.mtime = file_mtime(&absolute_path)?;
        entry.size = file_size(&absolute_path)?;
    }

    Ok(())
}
