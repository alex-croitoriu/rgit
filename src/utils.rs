use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs};

use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose;
use similar::{ChangeTag, TextDiff};

use crate::index::Index;
use crate::object_store::{Object, Tree, TreeEntry};

pub fn get_repository_root() -> Result<PathBuf> {
    let mut path = env::current_dir()?;
    loop {
        if is_repository_root(&path) {
            return Ok(path);
        }
        if !path.pop() {
            return Err(anyhow!("Repository not found"));
        }
    }
}

pub fn is_repository_root(path: &Path) -> bool {
    path.join(".rgit/objects").is_dir()
        && path.join(".rgit/refs/heads").is_dir()
        && path.join(".rgit/index").is_file()
        && path.join(".rgit/HEAD").is_file()
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component.as_os_str()),
        }
    }
    result
}

pub fn get_mtime(path: &Path) -> Result<u64> {
    Ok(path
        .metadata()?
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub fn get_current_branch_name() -> Result<String> {
    let root = get_repository_root()?;
    if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
        let current_branch = PathBuf::from(head)
            .file_name()
            .ok_or(anyhow!("Current branch not found"))?
            .to_string_lossy()
            .to_string();
        Ok(current_branch)
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}

pub fn get_current_branch_path() -> Result<PathBuf> {
    let root = get_repository_root()?;
    if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
        let path = normalize_path(&root.join(".rgit").join(head));
        Ok(path)
    } else {
        Err(anyhow!("Corrupt HEAD file"))
    }
}

pub fn get_branch_path(name: &str) -> Result<PathBuf> {
    let root = get_repository_root()?;
    Ok(normalize_path(&root.join(".rgit/refs/heads").join(name)))
}

pub fn get_staged_changes() -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    let current_index = Index::load()?;
    let head_path = get_current_branch_path()?;
    let head_index = if let Ok(head_hash) = fs::read_to_string(head_path) {
        Index::restore_from_commit(&head_hash)?
    } else {
        Index::new()
    };

    for (name, index_entry) in &current_index.entries {
        if let Some(head_entry) = head_index.entries.get(name) {
            if index_entry.hash != head_entry.hash {
                modified.push(name.clone());
            }
        } else {
            added.push(name.clone());
        }
    }

    for (name, _) in head_index.entries {
        if !current_index.entries.contains_key(&name) {
            deleted.push(name);
        }
    }

    Ok((added, deleted, modified))
}

pub fn get_unstaged_changes() -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    let ignored = get_ignored();
    let index = Index::load()?;
    let root = get_repository_root()?;
    let mut stack = vec![root.clone()];

    while !stack.is_empty() {
        if let Some(path) = stack.pop() {
            if path.is_file() {
                let relative_path = path.strip_prefix(&root)?;
                if let Some(entry) = index.entries.get(relative_path) {
                    if entry.size != path.metadata()?.len() || entry.mtime != get_mtime(&path)? {
                        modified.push(relative_path.to_path_buf());
                    }
                } else {
                    added.push(relative_path.to_path_buf());
                }
            } else if path.is_dir() {
                for entry in path.read_dir()?.flatten() {
                    let entry_path = entry.path();
                    let relative_path = entry_path.strip_prefix(&root)?;

                    if relative_path == ".rgit" {
                        continue;
                    }
                    if let Some(ignored) = &ignored
                        && ignored.iter().any(|p| relative_path.starts_with(p))
                    {
                        continue;
                    }

                    if entry.file_type()?.is_file() {
                        stack.push(entry_path);
                    } else if entry.file_type()?.is_dir() {
                        if index
                            .entries
                            .iter()
                            .any(|(name, _)| PathBuf::from(&name).starts_with(relative_path))
                        {
                            stack.push(entry_path);
                        } else if entry_path.read_dir()?.count() > 0 {
                            added.push(relative_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    for (name, _) in index.entries {
        if !root.join(&name).exists() {
            deleted.push(name);
        }
    }

    Ok((added, deleted, modified))
}

pub fn diff_indices(
    from: &Index,
    to: &Index,
) -> Result<(
    Vec<(PathBuf, String)>,
    Vec<(PathBuf, String)>,
    Vec<(PathBuf, String)>,
)> {
    let (mut added, mut deleted, mut modified) = (Vec::new(), Vec::new(), Vec::new());

    for (name, from_entry) in &from.entries {
        if let Some(to_entry) = to.entries.get(name) {
            if from_entry.hash != to_entry.hash {
                modified.push((name.clone(), diff(&from_entry.hash, &to_entry.hash)?));
            }
        } else if let Ok(from_content) = get_blob_content(&from_entry.hash) {
            deleted.push((name.clone(), generate_diff(&from_content, "")));
        } else {
            deleted.push((name.clone(), String::from("Binary file")));
        }
    }

    for (name, to_entry) in &to.entries {
        if !from.entries.contains_key(name) {
            if let Ok(to_content) = get_blob_content(&to_entry.hash) {
                added.push((name.clone(), generate_diff("", &to_content)));
            } else {
                added.push((name.clone(), String::from("Binary file")));
            }
        }
    }

    Ok((added, deleted, modified))
}

pub fn diff(from: &str, to: &str) -> Result<String> {
    let from_content = get_blob_content(from)?;
    let to_content = get_blob_content(to)?;
    Ok(generate_diff(&from_content, &to_content))
}

pub fn get_blob_content(hash: &str) -> Result<String> {
    let bytes = get_blob_bytes(hash)?;
    if let Ok(content) = String::from_utf8(bytes) {
        Ok(content)
    } else {
        Ok(String::from("Binary file"))
    }
}

pub fn get_blob_bytes(hash: &str) -> Result<Vec<u8>> {
    if let Object::Blob(blob) = Object::load(hash)? {
        let bytes = general_purpose::STANDARD.decode(blob.content)?;
        Ok(bytes)
    } else {
        Err(anyhow!("Object {} is not a blob", hash))
    }
}

fn generate_diff(from: &str, to: &str) -> String {
    let diff = TextDiff::from_lines(from, to);
    let mut output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }
    output
}

pub fn get_ignored() -> Option<Vec<PathBuf>> {
    let mut ignored = Vec::new();
    let file = File::open(get_repository_root().ok()?.join(".rgitignore")).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        ignored.push(PathBuf::from(line));
    }
    Some(ignored)
}

pub fn update_working_directory(index: &mut Index, old_index: &Index) -> Result<()> {
    let root = get_repository_root()?;

    for path in old_index.entries.keys() {
        if !index.entries.contains_key(path) {
            let absolute_path = root.join(path);
            if absolute_path.exists() {
                fs::remove_file(absolute_path)?;
            }
        }
    }

    for (path, entry) in &mut index.entries {
        let content = get_blob_bytes(&entry.hash)?;
        let absolute_path = root.join(path);
        if let Some(parent) = absolute_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&absolute_path, content)?;

        entry.mtime = get_mtime(&absolute_path)?;
        entry.size = fs::metadata(&absolute_path)?.len();
    }

    Ok(())
}

pub fn create_tree_from_index(index: &Index) -> Result<String> {
    let mut stack = Vec::<(String, Tree)>::new();
    stack.push((
        String::from("root"),
        Tree {
            entries: Vec::new(),
        },
    ));

    for (name, entry) in &index.entries {
        let path = PathBuf::from(name);
        let components = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect::<Vec<_>>();

        let file = components
            .last()
            .ok_or(anyhow!("Last element not found"))?
            .to_string();
        let mut i = 0;

        while i + 1 < stack.len() && i < components.len() && stack[i + 1].0 == components[i] {
            i += 1;
        }

        while i + 1 < stack.len() {
            if let Some(last) = stack.pop()
                && let Some(second_to_last) = stack.last_mut()
            {
                second_to_last.1.entries.push(TreeEntry {
                    object_type: String::from("Tree"),
                    object_hash: Object::Tree(last.1).store()?,
                    name: last.0,
                });
            }
        }

        stack.extend(components[i..components.len() - 1].iter().map(|c| {
            (
                c.to_string(),
                Tree {
                    entries: Vec::new(),
                },
            )
        }));

        if let Some(last) = stack.last_mut() {
            last.1.entries.push(TreeEntry {
                object_type: String::from("Blob"),
                object_hash: entry.hash.clone(),
                name: file,
            });
        }
    }

    while 1 < stack.len() {
        if let Some(last) = stack.pop()
            && let Some(second_to_last) = stack.last_mut()
        {
            second_to_last.1.entries.push(TreeEntry {
                object_type: String::from("Tree"),
                object_hash: Object::Tree(last.1).store()?,
                name: last.0,
            });
        }
    }

    Object::Tree(stack.pop().ok_or(anyhow!("Error at pop stack"))?.1).store()
}
