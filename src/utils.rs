use std::env;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Result, anyhow};

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
