use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    state::{Index, read_blob_bytes},
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
