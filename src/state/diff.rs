use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use similar::ChangeTag;

use crate::{
    state::{Index, read_blob_text},
    utils::objects_dir_path,
};

pub struct DiffEntry {
    pub path: PathBuf,
    pub change: String,
}

#[derive(Default)]
pub struct Diff {
    pub added: Vec<DiffEntry>,
    pub deleted: Vec<DiffEntry>,
    pub modified: Vec<DiffEntry>,
}

impl std::fmt::Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.added {
            writeln!(f, "{:<11}{}", "Added:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }
        for entry in &self.deleted {
            writeln!(f, "{:<11}{}", "Deleted:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }
        for entry in &self.modified {
            writeln!(f, "{:<11}{}", "Modified:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }

        Ok(())
    }
}

pub fn diff_indexes(root: &Path, from: &Index, to: &Index) -> Result<Diff> {
    let mut diff = Diff::default();

    for (name, from_entry) in &from.entries {
        if let Some(to_entry) = to.entries.get(name) {
            if from_entry.hash != to_entry.hash {
                let from_content = read_blob_text(&objects_dir_path(&root), &from_entry.hash)?;
                let to_content = read_blob_text(&objects_dir_path(&root), &to_entry.hash)?;
                diff.modified.push(DiffEntry {
                    path: name.clone(),
                    change: diff_text(&from_content, &to_content)?,
                });
            }
        } else if let Ok(from_content) = read_blob_text(&objects_dir_path(&root), &from_entry.hash)
        {
            diff.deleted.push(DiffEntry {
                path: name.clone(),
                change: diff_text(&from_content, "")?,
            });
        } else {
            diff.deleted.push(DiffEntry {
                path: name.clone(),
                change: String::from("Binary file"),
            });
        }
    }

    for (name, to_entry) in &to.entries {
        if !from.entries.contains_key(name) {
            if let Ok(to_content) = read_blob_text(&objects_dir_path(&root), &to_entry.hash) {
                diff.added.push(DiffEntry {
                    path: name.clone(),
                    change: diff_text("", &to_content)?,
                });
            } else {
                diff.added.push(DiffEntry {
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
