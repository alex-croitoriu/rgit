use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use similar::{ChangeTag, TextDiff};

use crate::{
    state::Index,
    utils::{get_blob_content, get_branch_path, get_current_branch_path},
};

struct DiffEntry {
    path: PathBuf,
    change: String,
}

struct Diff {
    added: Vec<DiffEntry>,
    deleted: Vec<DiffEntry>,
    modified: Vec<DiffEntry>,
}

pub fn diff(target: Option<String>) -> Result<String> {
    let mut output = String::new();
    if let Some(target) = target {
        let branch_path = get_branch_path(&target)?;
        if !branch_path.exists() {
            return Err(anyhow!("Branch does not exist: '{target}'"));
        }

        let branch_hash = fs::read_to_string(branch_path)?;
        let branch_index = Index::restore_from_commit(&branch_hash)?;

        let head_path = get_current_branch_path()?;
        let head_hash = fs::read_to_string(head_path)?;
        let head_index = Index::restore_from_commit(&head_hash)?;

        let diff = diff_indices(&head_index, &branch_index)?;
        for entry in diff.added {
            output.push_str(&format!("{:<11}{}\n", "Added:", entry.path.display()));
            output.push_str(&entry.change);
        }
        for entry in diff.deleted {
            output.push_str(&format!("{:<11}{}\n", "Deleted:", entry.path.display()));
            output.push_str(&entry.change);
        }
        for entry in diff.modified {
            output.push_str(&format!("{:<11}{}\n", "Modified:", entry.path.display()));
            output.push_str(&entry.change);
        }
    } else {
        let head_path = get_current_branch_path()?;

        let head_index = if let Ok(head_hash) = fs::read_to_string(head_path) {
            Index::restore_from_commit(&head_hash)?
        } else {
            Index::new()
        };

        let current_index = Index::load()?;

        let diff = diff_indices(&head_index, &current_index)?;
        for entry in diff.added {
            output.push_str(&format!("{:<11}{}\n", "Added:", entry.path.display()));
            output.push_str(&entry.change);
        }
        for entry in diff.deleted {
            output.push_str(&format!("{:<11}{}\n", "Deleted:", entry.path.display()));
            output.push_str(&entry.change);
        }
        for entry in diff.modified {
            output.push_str(&format!("{:<11}{}\n", "Modified:", entry.path.display()));
            output.push_str(&entry.change);
        }
    }

    Ok(output)
}

fn diff_indices(from: &Index, to: &Index) -> Result<Diff> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    for (name, from_entry) in &from.entries {
        if let Some(to_entry) = to.entries.get(name) {
            if from_entry.hash != to_entry.hash {
                modified.push(DiffEntry {
                    path: name.clone(),
                    change: diff_files(&from_entry.hash, &to_entry.hash)?,
                });
            }
        } else if let Ok(from_content) = get_blob_content(&from_entry.hash) {
            deleted.push(DiffEntry {
                path: name.clone(),
                change: diff_text(&from_content, ""),
            });
        } else {
            deleted.push(DiffEntry {
                path: name.clone(),
                change: String::from("Binary file"),
            });
        }
    }

    for (name, to_entry) in &to.entries {
        if !from.entries.contains_key(name) {
            if let Ok(to_content) = get_blob_content(&to_entry.hash) {
                added.push(DiffEntry {
                    path: name.clone(),
                    change: diff_text("", &to_content),
                });
            } else {
                added.push(DiffEntry {
                    path: name.clone(),
                    change: String::from("Binary file"),
                });
            }
        }
    }

    Ok(Diff {
        added,
        deleted,
        modified,
    })
}

fn diff_files(from: &str, to: &str) -> Result<String> {
    let from_content = get_blob_content(from)?;
    let to_content = get_blob_content(to)?;
    Ok(diff_text(&from_content, &to_content))
}

fn diff_text(from: &str, to: &str) -> String {
    let diff = TextDiff::from_lines(from, to);
    let mut output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };
        output.push_str(&format!("{sign}{change}"));
    }
    output
}
