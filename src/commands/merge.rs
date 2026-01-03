use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{Result, anyhow};

use crate::utils::{create_tree_from_index, update_working_directory};
use crate::{
    index::Index,
    object_store::{Commit, Object},
    utils::{
        get_blob_content, get_branch_path, get_current_branch_path, get_repository_root,
        get_staged_changes, get_unstaged_changes,
    },
};

pub fn merge(target: &str) -> Result<String> {
    let staged_changes = get_staged_changes()?;
    let unstaged_changes = get_unstaged_changes()?;

    if !(staged_changes.0.is_empty()
        && staged_changes.1.is_empty()
        && staged_changes.2.is_empty()
        && unstaged_changes.0.is_empty()
        && unstaged_changes.1.is_empty()
        && unstaged_changes.2.is_empty())
    {
        return Err(anyhow!("Unable to merge: uncommited changes"));
    }

    let current_branch_path = get_current_branch_path()?;
    let current_hash = fs::read_to_string(&current_branch_path)?;
    let target_path = get_branch_path(target)?;

    if !target_path.exists() {
        return Err(anyhow!("Branch '{}' does not exist", target));
    }
    let target_hash = fs::read_to_string(&target_path)?;

    if is_ancestor(&current_hash, &target_hash)? {
        println!("Fast-forward merge");
        fs::write(&current_branch_path, &target_hash)?;

        let target_index = Index::restore_from_commit(&target_hash)?;
        let current_index = Index::load()?;
        update_working_directory(&mut target_index.clone(), &current_index)?;
        target_index.store()?;

        return Ok(target_hash);
    }

    let base_hash = find_common_ancestor(&current_hash, &target_hash)?;

    let base_index = Index::restore_from_commit(&base_hash)?;
    let head_index = Index::restore_from_commit(&current_hash)?;
    let target_index = Index::restore_from_commit(&target_hash)?;

    let (mut merged_index, conflicts) = three_way_merge(&base_index, &head_index, &target_index)?;

    if !conflicts.is_empty() {
        write_conflict_markers(&conflicts, &head_index, &target_index)?;
        return Err(anyhow!("Merge conflicts in: {:?}", conflicts));
    }

    let tree_hash = create_tree_from_index(&merged_index)?;
    let parent_hashes = vec![current_hash.clone(), target_hash.clone()];

    let commit = Commit {
        tree_hash,
        message: format!("Merge branch '{}'", target),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
        parent_hashes,
    };

    let commit_hash = Object::Commit(commit).store()?;
    fs::write(current_branch_path, &commit_hash)?;

    update_working_directory(&mut merged_index, &head_index)?;
    merged_index.store()?;

    Ok(commit_hash)
}

fn is_ancestor(ancestor: &str, descendant: &str) -> Result<bool> {
    let mut queue = VecDeque::from([descendant.to_string()]);
    let mut visited = HashSet::new();

    while let Some(hash) = queue.pop_front() {
        if !visited.insert(hash.clone()) {
            continue;
        }
        if hash == ancestor {
            return Ok(true);
        }
        if let Object::Commit(commit) = Object::load(&hash)? {
            for parent in commit.parent_hashes {
                queue.push_back(parent);
            }
        }
    }

    Ok(false)
}

fn find_common_ancestor(hash1: &str, hash2: &str) -> Result<String> {
    let mut ancestors1 = HashSet::new();
    let mut queue = VecDeque::from([hash1.to_string()]);

    while let Some(hash) = queue.pop_front() {
        if !ancestors1.insert(hash.clone()) {
            continue;
        }
        if let Object::Commit(commit) = Object::load(&hash)? {
            for parent in commit.parent_hashes {
                queue.push_back(parent);
            }
        }
    }

    let mut queue = VecDeque::from([hash2.to_string()]);
    let mut visited = HashSet::new();

    while let Some(hash) = queue.pop_front() {
        if !visited.insert(hash.clone()) {
            continue;
        }
        if ancestors1.contains(&hash) {
            return Ok(hash);
        }
        if let Object::Commit(commit) = Object::load(&hash)? {
            for parent in commit.parent_hashes {
                queue.push_back(parent);
            }
        }
    }

    Err(anyhow!("No common ancestor found"))
}

fn three_way_merge(base: &Index, head: &Index, target: &Index) -> Result<(Index, Vec<PathBuf>)> {
    let mut merged = Index::new();
    let mut conflicts = Vec::new();

    let mut all_paths = HashSet::new();
    for path in base.entries.keys() {
        all_paths.insert(path.clone());
    }
    for path in head.entries.keys() {
        all_paths.insert(path.clone());
    }
    for path in target.entries.keys() {
        all_paths.insert(path.clone());
    }

    for path in all_paths {
        let base_entry = base.entries.get(&path);
        let head_entry = head.entries.get(&path);
        let target_entry = target.entries.get(&path);

        match (base_entry, head_entry, target_entry) {
            (Some(b), Some(h), Some(t)) => {
                if h.hash == t.hash {
                    merged.entries.insert(path.clone(), h.clone());
                } else if h.hash == b.hash {
                    merged.entries.insert(path.clone(), t.clone());
                } else if t.hash == b.hash {
                    merged.entries.insert(path.clone(), h.clone());
                } else {
                    conflicts.push(path.clone());
                }
            }
            (Some(b), Some(h), None) => {
                if h.hash != b.hash {
                    conflicts.push(path.clone());
                }
            }
            (Some(b), None, Some(t)) => {
                if t.hash != b.hash {
                    conflicts.push(path.clone());
                }
            }
            (None, Some(h), Some(t)) => {
                if h.hash == t.hash {
                    merged.entries.insert(path.clone(), h.clone());
                } else {
                    conflicts.push(path.clone());
                }
            }
            (None, Some(h), None) => {
                merged.entries.insert(path.clone(), h.clone());
            }
            (None, None, Some(t)) => {
                merged.entries.insert(path.clone(), t.clone());
            }
            _ => {}
        }
    }

    Ok((merged, conflicts))
}

fn write_conflict_markers(conflicts: &[PathBuf], ours: &Index, theirs: &Index) -> Result<()> {
    let root = get_repository_root()?;
    for path in conflicts {
        let ours_content = if let Some(entry) = ours.entries.get(path) {
            get_blob_content(&entry.hash)?
        } else {
            String::new()
        };

        let theirs_content = if let Some(entry) = theirs.entries.get(path) {
            get_blob_content(&entry.hash)?
        } else {
            String::new()
        };

        let content = format!(
            "<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>>\n",
            ours_content, theirs_content
        );

        let full_path = root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full_path, content)?;
    }
    Ok(())
}

