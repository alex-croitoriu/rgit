use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use similar::ChangeTag;

use crate::state::{Index, Object};

struct DiffEntry {
    path: PathBuf,
    change: String,
}

#[derive(Default)]
pub struct Diff {
    added: Vec<DiffEntry>,
    deleted: Vec<DiffEntry>,
    modified: Vec<DiffEntry>,
}

impl std::fmt::Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: fix newlines
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
                let change = match (
                    Object::load(root, &from_entry.hash)?.blob_text()?,
                    Object::load(root, &to_entry.hash)?.blob_text()?,
                ) {
                    (Some(from_content), Some(to_content)) => {
                        diff_text(&from_content, &to_content)?
                    }
                    _ => String::from("Binary file"),
                };

                diff.modified.push(DiffEntry {
                    path: name.clone(),
                    change,
                });
            }
        } else {
            let change =
                if let Some(from_content) = Object::load(root, &from_entry.hash)?.blob_text()? {
                    let diff_text = diff_text(&from_content, "")?;
                    if diff_text.is_empty() {
                        String::from("Empty file")
                    } else {
                        diff_text
                    }
                } else {
                    String::from("Binary file")
                };

            diff.deleted.push(DiffEntry {
                path: name.clone(),
                change,
            });
        }
    }

    for (name, to_entry) in &to.entries {
        if !from.entries.contains_key(name) {
            let change =
                if let Some(to_content) = Object::load(root, &to_entry.hash)?.blob_text()? {
                    let diff_text = diff_text("", &to_content)?;
                    if diff_text.is_empty() {
                        String::from("Empty file")
                    } else {
                        diff_text
                    }
                } else {
                    String::from("Binary file")
                };

            diff.added.push(DiffEntry {
                path: name.clone(),
                change,
            });
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
