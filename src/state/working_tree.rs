use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    state::{Index, Object},
    utils::{file_mtime, file_size, normalize_path},
};

pub fn ignored_paths(root: &Path) -> Vec<PathBuf> {
    let mut ignored = Vec::new();
    if let Ok(file) = File::open(root.join(".rgitignore")) {
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            ignored.push(normalize_path(&PathBuf::from(line)));
        }
    }

    ignored
}

pub fn update_working_tree(root: &Path, new_index: &mut Index, old_index: &Index) -> Result<()> {
    for path in old_index.entries.keys() {
        if !new_index.entries.contains_key(path) {
            let absolute_path = root.join(path);
            if absolute_path.exists() {
                fs::remove_file(&absolute_path)?;
                if let Some(parent) = absolute_path.parent() {
                    for ancestor in parent.ancestors() {
                        if ancestor == root || !ancestor.starts_with(root) {
                            break;
                        }
                        if ancestor.read_dir()?.next().is_none() {
                            fs::remove_dir(ancestor)?;
                        }
                        else {
                            break;
                        }
                    }
                }
            }
        }
    }

    for (path, new_entry) in &mut new_index.entries {
        if let Some(old_entry) = old_index.entries.get(path)
            && new_entry.hash == old_entry.hash
        {
            new_entry.mtime = old_entry.mtime;
            new_entry.size = old_entry.size;
            continue;
        }

        let bytes = Object::load(root, &new_entry.hash)?.blob_bytes()?;
        let absolute_path = root.join(path);
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&absolute_path, bytes)?;

        new_entry.mtime = file_mtime(&absolute_path)?;
        new_entry.size = file_size(&absolute_path)?;
    }

    Ok(())
}
