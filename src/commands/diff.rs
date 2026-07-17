use std::{fmt::Write, fs};

use anyhow::{Result, anyhow};
use similar::ChangeTag;

use crate::{
    commands,
    state::{Index, Repository, TextDiff, TextDiffEntry, branch_path, head_hash, read_blob_text},
    utils::objects_dir_path,
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    target: Option<String>,
}

pub struct Output {
    diff: TextDiff,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diff)
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let head_index = if let Some(hash) = head_hash(&repository.root)? {
            Index::load_from_commit(&repository.root, &hash)?
        } else {
            Index::new()
        };

        if let Some(target) = &args.target {
            let branch_path = branch_path(&repository.root, target);
            if !branch_path.exists() {
                return Err(anyhow!("Branch does not exist: '{target}'"));
            }

            let branch_hash = fs::read_to_string(branch_path)?;
            let branch_index = Index::load_from_commit(&repository.root, &branch_hash)?;

            let diff = diff_indexes(repository, &head_index, &branch_index)?;

            Ok(Output { diff })
        } else {
            let current_index = Index::load(&repository.root)?;
            let diff = diff_indexes(repository, &head_index, &current_index)?;

            Ok(Output { diff })
        }
    }
}

fn diff_indexes(repository: &Repository, from: &Index, to: &Index) -> Result<TextDiff> {
    let mut diff = TextDiff::default();

    for (name, from_entry) in &from.entries {
        if let Some(to_entry) = to.entries.get(name) {
            if from_entry.hash != to_entry.hash {
                let from_content =
                    read_blob_text(&objects_dir_path(&repository.root), &from_entry.hash)?;
                let to_content =
                    read_blob_text(&objects_dir_path(&repository.root), &to_entry.hash)?;
                diff.modified.push(TextDiffEntry {
                    path: name.clone(),
                    change: diff_text(&from_content, &to_content)?,
                });
            }
        } else if let Ok(from_content) =
            read_blob_text(&objects_dir_path(&repository.root), &from_entry.hash)
        {
            diff.deleted.push(TextDiffEntry {
                path: name.clone(),
                change: diff_text(&from_content, "")?,
            });
        } else {
            diff.deleted.push(TextDiffEntry {
                path: name.clone(),
                change: String::from("Binary file"),
            });
        }
    }

    for (name, to_entry) in &to.entries {
        if !from.entries.contains_key(name) {
            if let Ok(to_content) =
                read_blob_text(&objects_dir_path(&repository.root), &to_entry.hash)
            {
                diff.added.push(TextDiffEntry {
                    path: name.clone(),
                    change: diff_text("", &to_content)?,
                });
            } else {
                diff.added.push(TextDiffEntry {
                    path: name.clone(),
                    change: String::from("Binary file"),
                });
            }
        }
    }

    Ok(diff)
}

fn diff_text(from: &str, to: &str) -> Result<String> {
    let diff = similar::TextDiff::from_lines(from, to);
    let mut output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };
        write!(output, "{sign}{change}")?;
    }
    Ok(output)
}
