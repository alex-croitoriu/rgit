use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::index::Index;
use crate::utils::{get_mtime, get_repository_root};

pub fn status() -> Result<String> {
    let index = Index::read()?;
    let root = get_repository_root()?;
    let mut stack = vec![root.clone()];

    let mut status = String::new();

    while !stack.is_empty() {
        if let Some(path) = stack.pop() {
            if path.is_file() {
                let relative_path = path.strip_prefix(&root)?;
                if let Some(entry) = index
                    .entries
                    .get(relative_path.to_str().ok_or(anyhow!("Invalid path"))?)
                {
                    if entry.size != path.metadata()?.len() || entry.mtime != get_mtime(&path)? {
                        status.push_str(
                            format!("{}: staged but modified\n", relative_path.display()).as_str(),
                        );
                    } else {
                        status.push_str(format!("{}: staged\n", relative_path.display()).as_str());
                    }
                } else {
                    status.push_str(format!("{}: untracked\n", relative_path.display()).as_str());
                }
            } else if path.is_dir() {
                for entry in path.read_dir()?.flatten() {
                    let entry_path = entry.path();
                    let relative_path = entry_path.strip_prefix(&root)?;

                    if relative_path == ".rgit" {
                        continue;
                    }

                    if entry.file_type()?.is_file() {
                        stack.push(entry.path());
                    } else if entry.file_type()?.is_dir() {
                        if index
                            .entries
                            .iter()
                            .any(|(name, _)| PathBuf::from(&name).starts_with(relative_path))
                        {
                            stack.push(entry.path());
                        } else {
                            status.push_str(
                                format!("{}: untracked\n", relative_path.display()).as_str(),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(status)
}
