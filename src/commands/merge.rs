use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::PathBuf,
    time::SystemTime,
};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Commit, Index, Object, Repository},
    utils::{unstaged_changes, update_working_tree},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    target: String,
}

pub struct Output {
    hash: String,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Commit: {}", self.hash)?;
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = repository.staged_changes()?;
        let unstaged_changes = unstaged_changes(repository)?;

        if !staged_changes.is_empty() || !unstaged_changes.is_empty() {
            return Err(anyhow!("Unable to merge: uncommited changes"));
        }

        let current_branch_path = repository.current_branch_path()?;
        let current_hash = fs::read_to_string(&current_branch_path)?;
        let target_path = repository.branch_path(&args.target);

        if !target_path.exists() {
            return Err(anyhow!("Branch does not exist: '{}'", args.target));
        }
        let target_hash = fs::read_to_string(&target_path)?;

        if is_ancestor(repository, &current_hash, &target_hash)? {
            fs::write(&current_branch_path, &target_hash)?;

            let mut target_index = repository.load_index_from_commit(&target_hash)?;
            let current_index = repository.load_index()?;
            update_working_tree(repository, &mut target_index, &current_index)?;
            repository.store_index(&target_index)?;

            return Ok(Output { hash: target_hash });
        }

        let base_hash = find_common_ancestor(repository, &current_hash, &target_hash)?;

        let base_index = repository.load_index_from_commit(&base_hash)?;
        let head_index = repository.load_index_from_commit(&current_hash)?;
        let target_index = repository.load_index_from_commit(&target_hash)?;

        let (mut merged_index, conflicts) =
            three_way_merge(&base_index, &head_index, &target_index);

        if !conflicts.is_empty() {
            fs::write(repository.merge_head_file_path(), &target_hash)?;

            update_working_tree(repository, &mut merged_index, &head_index)?;
            repository.store_index(&merged_index)?;

            write_conflict_markers(repository, conflicts.clone(), &head_index, &target_index)?;

            return Err(anyhow!(
                "Merge conflicts in: {}",
                conflicts
                    .iter()
                    .fold(String::new(), |acc, path| if acc.is_empty() {
                        path.display().to_string()
                    } else {
                        format!("{}, {}", acc, path.display())
                    })
            ));
        }

        let tree_hash = repository.store_index_tree(&merged_index)?;
        let parent_hashes = vec![current_hash.clone(), target_hash.clone()];

        let commit = Commit {
            tree_hash,
            message: format!("Merge branch '{}'", args.target),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
            parent_hashes,
        };

        let commit_hash = repository.store_object(&Object::Commit(commit))?;
        fs::write(current_branch_path, &commit_hash)?;

        update_working_tree(repository, &mut merged_index, &head_index)?;
        repository.store_index(&merged_index)?;

        Ok(Output { hash: commit_hash })
    }
}

fn is_ancestor(repository: &Repository, ancestor: &str, descendant: &str) -> Result<bool> {
    let mut queue = VecDeque::from([descendant.to_string()]);
    let mut visited = HashSet::new();

    while let Some(hash) = queue.pop_front() {
        if !visited.insert(hash.clone()) {
            continue;
        }
        if hash == ancestor {
            return Ok(true);
        }
        if let Object::Commit(commit) = repository.load_object(&hash)? {
            for parent in commit.parent_hashes {
                queue.push_back(parent);
            }
        }
    }

    Ok(false)
}

fn find_common_ancestor(repository: &Repository, hash1: &str, hash2: &str) -> Result<String> {
    let mut ancestors1 = HashSet::new();
    let mut queue = VecDeque::from([hash1.to_string()]);

    while let Some(hash) = queue.pop_front() {
        if !ancestors1.insert(hash.clone()) {
            continue;
        }
        if let Object::Commit(commit) = repository.load_object(&hash)? {
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
        if let Object::Commit(commit) = repository.load_object(&hash)? {
            for parent in commit.parent_hashes {
                queue.push_back(parent);
            }
        }
    }

    Err(anyhow!("No common ancestor found"))
}

fn three_way_merge(base: &Index, head: &Index, target: &Index) -> (Index, Vec<PathBuf>) {
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

    (merged, conflicts)
}

fn write_conflict_markers(
    repository: &Repository,
    conflicts: Vec<PathBuf>,
    head: &Index,
    target: &Index,
) -> Result<()> {
    for path in conflicts {
        let head_content = if let Some(entry) = head.entries.get(&path) {
            repository.load_blob_text(&entry.hash)?
        } else {
            String::new()
        };

        let target_content = if let Some(entry) = target.entries.get(&path) {
            repository.load_blob_text(&entry.hash)?
        } else {
            String::new()
        };

        let content = format!("<<<<<<< HEAD\n{head_content}\n=======\n{target_content}\n>>>>>>>\n");

        let absolute_path = repository.root.join(path);
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(absolute_path, content)?;
    }
    Ok(())
}
