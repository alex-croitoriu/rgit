use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{Result, anyhow};

use crate::{
    index::Index,
    object_store::{Commit, Object, Tree, TreeEntry},
    utils::get_current_branch_path,
};

pub fn commit(message: &str) -> Result<String> {
    let index = Index::load()?;
    let mut stack = Vec::<(String, Tree)>::new();

    stack.push((
        String::from("root"),
        Tree {
            entries: Vec::new(),
        },
    ));

    for (name, entry) in index.entries {
        let path = PathBuf::from(&name);
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
                object_hash: entry.hash,
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

    let tree_hash = Object::Tree(stack.pop().ok_or(anyhow!("Error at pop stack"))?.1).store()?;
    let parent_hashes = if let Ok(path) = get_current_branch_path()
        && path.exists()
    {
        vec![fs::read_to_string(path)?]
    } else {
        Vec::new()
    };

    let commit_object = Object::Commit(Commit {
        message: message.to_string(),
        parent_hashes,
        tree_hash,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs(),
    });

    let commit_hash = commit_object.store()?;
    fs::write(get_current_branch_path()?, &commit_hash)?;

    Ok(commit_hash)
}
