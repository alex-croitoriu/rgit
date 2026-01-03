use std::fs;

use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose};

use crate::{
    index::Index,
    object_store::Object,
    utils::{
        get_branch_path, get_current_branch_path, get_mtime, get_repository_root,
        get_staged_changes, get_unstaged_changes, normalize_path,
    },
};

pub fn checkout(target: &str) -> Result<()> {
    let staged_changes = get_staged_changes()?;
    let unstaged_changes = get_unstaged_changes()?;

    if !(staged_changes.0.is_empty()
        && staged_changes.1.is_empty()
        && staged_changes.2.is_empty()
        && unstaged_changes.0.is_empty()
        && unstaged_changes.1.is_empty()
        && unstaged_changes.2.is_empty())
    {
        return Err(anyhow!("Unable to checkout: uncommited changes"));
    }

    if let Ok(branch_path) = get_branch_path(target) {
        if branch_path == get_current_branch_path()? {
            return Err(anyhow!("Unable to checkout: already on that branch"));
        }
        if !branch_path.exists() {
            return Err(anyhow!("Unable to checkout: branch does not exist"));
        }

        let hash = fs::read_to_string(branch_path)?;

        let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

        let current_index = Index::load()?;
        let mut target_index = Index::restore_from_commit(&hash)?;

        for (name, index_entry) in &target_index.entries {
            if let Some(head_entry) = current_index.entries.get(name) {
                if index_entry.hash != head_entry.hash {
                    modified.push((name.clone(), index_entry.hash.clone()));
                }
            } else {
                added.push((name.clone(), index_entry.hash.clone()));
            }
        }

        for (name, entry) in &current_index.entries {
            if !target_index.entries.contains_key(name) {
                deleted.push((name.clone(), entry.hash.clone()));
            }
        }

        println!("{added:?}\n{deleted:?}\n{modified:?}\n");

        let root = get_repository_root()?;

        for (path, hash) in added {
            let absolute_path = normalize_path(&root.join(path));
            if let Object::Blob(blob) = Object::load(&hash)? {
                let content = general_purpose::STANDARD.decode(blob.content)?;
                if let Some(parent) = absolute_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(absolute_path, content)?;
            }
        }

        for (path, hash) in modified {
            if let Object::Blob(blob) = Object::load(&hash)? {
                let content = general_purpose::STANDARD.decode(blob.content)?;
                fs::write(root.join(&path), content.as_slice())?;
            }
        }

        for (path, _) in deleted {
            fs::remove_file(root.join(&path))?;
        }

        for (path, entry) in &mut target_index.entries {
            let full_path = root.join(path);
            if full_path.exists() {
                entry.mtime = get_mtime(&full_path)?;
                entry.size = fs::metadata(&full_path)?.len();
            }
        }

        target_index.store()?;

        fs::write(root.join(".rgit/HEAD"), format!("ref: refs/heads/{target}"))?;
    }

    Ok(())
}
