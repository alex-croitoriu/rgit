use std::{fmt::Write, fs};

use anyhow::{Result, anyhow};
use similar::ChangeTag;

use crate::{
    commands::{self, TextDiff, TextDiffEntry},
    state::{Index, Repository},
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
        for entry in &self.diff.added {
            writeln!(f, "{:<11}{}", "Added:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }
        for entry in &self.diff.deleted {
            writeln!(f, "{:<11}{}", "Deleted:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }
        for entry in &self.diff.modified {
            writeln!(f, "{:<11}{}", "Modified:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }

        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        if let Some(target) = &args.target {
            let branch_path = repository.branch_path(target);
            if !branch_path.exists() {
                return Err(anyhow!("Branch does not exist: '{target}'"));
            }

            let branch_hash = fs::read_to_string(branch_path)?;
            let branch_index = repository.load_index_from_commit(&branch_hash)?;

            let head_path = repository.current_branch_path()?;
            let head_hash = fs::read_to_string(head_path)?;
            let head_index = repository.load_index_from_commit(&head_hash)?;

            let diff = diff_indexes(repository, &head_index, &branch_index)?;

            Ok(Output { diff })
        } else {
            let head_path = repository.current_branch_path()?;
            let head_index = if let Ok(head_hash) = fs::read_to_string(head_path) {
                repository.load_index_from_commit(&head_hash)?
            } else {
                Index::new()
            };

            let current_index = repository.load_index()?;

            let diff = diff_indexes(repository, &head_index, &current_index)?;

            Ok(Output { diff })
        }
    }
}

fn diff_indexes(repository: &Repository, from: &Index, to: &Index) -> Result<TextDiff> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    for (name, from_entry) in &from.entries {
        if let Some(to_entry) = to.entries.get(name) {
            if from_entry.hash != to_entry.hash {
                let from_content = repository.load_blob_text(&from_entry.hash)?;
                let to_content = repository.load_blob_text(&to_entry.hash)?;
                modified.push(TextDiffEntry {
                    path: name.clone(),
                    change: diff_text(&from_content, &to_content)?,
                });
            }
        } else if let Ok(from_content) = repository.load_blob_text(&from_entry.hash) {
            deleted.push(TextDiffEntry {
                path: name.clone(),
                change: diff_text(&from_content, "")?,
            });
        } else {
            deleted.push(TextDiffEntry {
                path: name.clone(),
                change: String::from("Binary file"),
            });
        }
    }

    for (name, to_entry) in &to.entries {
        if !from.entries.contains_key(name) {
            if let Ok(to_content) = repository.load_blob_text(&to_entry.hash) {
                added.push(TextDiffEntry {
                    path: name.clone(),
                    change: diff_text("", &to_content)?,
                });
            } else {
                added.push(TextDiffEntry {
                    path: name.clone(),
                    change: String::from("Binary file"),
                });
            }
        }
    }

    Ok(TextDiff {
        added,
        deleted,
        modified,
    })
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
