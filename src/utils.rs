use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::SystemTime,
};

use anyhow::Result;
use chrono::{DateTime, Local};

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

pub fn file_mtime(path: &Path) -> Result<u64> {
    Ok(path
        .metadata()?
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub fn file_size(path: &Path) -> Result<u64> {
    Ok(path.metadata()?.len())
}

pub fn trimmed_file_content(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?.trim().to_string())
}

pub fn date_from_timestamp(timestamp: u64) -> String {
    if let Some(timestamp) = DateTime::from_timestamp_secs(timestamp as i64) {
        timestamp.with_timezone(&Local).to_string()
    } else {
        String::from("Invalid timestamp")
    }
}

pub fn objects_dir_path(root: &Path) -> PathBuf {
    root.join(".rgit/objects")
}

pub fn heads_dir_path(root: &Path) -> PathBuf {
    root.join(".rgit/refs/heads")
}

pub fn index_file_path(root: &Path) -> PathBuf {
    root.join(".rgit/index")
}

pub fn head_file_path(root: &Path) -> PathBuf {
    root.join(".rgit/HEAD")
}

pub fn merge_head_file_path(root: &Path) -> PathBuf {
    root.join(".rgit/MERGE_HEAD")
}
