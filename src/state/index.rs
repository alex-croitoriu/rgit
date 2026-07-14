use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::state::{Tree, TreeEntry};
use crate::{state::Object, utils::get_repository_root};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexEntry {
    pub hash: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Index {
    pub entries: BTreeMap<PathBuf, IndexEntry>,
}

// TODO: replace json serialization with a better alternative (maybe wincode)
impl Index {
    pub fn new() -> Self {
        Index {
            entries: BTreeMap::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let root = get_repository_root()?;
        let file = OpenOptions::new()
            .read(true)
            .open(root.join(".rgit/index"))?;
        let reader = BufReader::new(file);
        let index = serde_json::from_reader(reader).unwrap_or(Index::new());

        Ok(index)
    }

    pub fn store(&self) -> Result<()> {
        let root = get_repository_root()?;
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(root.join(".rgit/index"))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self)?;

        Ok(())
    }

    pub fn add(&mut self, name: &Path, entry: IndexEntry) -> Result<()> {
        self.entries.insert(name.to_path_buf(), entry);

        Ok(())
    }

    pub fn remove(&mut self, path: &Path) -> Result<()> {
        if self.entries.remove(path).is_some() {
            Ok(())
        } else {
            Err(anyhow!("File not in index: '{}'", path.display()))
        }
    }

    pub fn restore_from_commit(hash: &str) -> Result<Self> {
        let mut index = Index::new();
        let mut stack = Vec::new();

        if let Object::Commit(commit) = Object::load(hash)? {
            if let Object::Tree(tree) = Object::load(&commit.tree_hash)? {
                stack.push((tree, PathBuf::new()));
            }

            while let Some((tree, path)) = stack.pop() {
                for entry in tree.entries {
                    if entry.object_type == "Blob" {
                        index.entries.insert(
                            path.join(entry.name),
                            IndexEntry {
                                hash: entry.object_hash,
                                size: 0,
                                mtime: 0,
                            },
                        );
                    } else if entry.object_type == "Tree"
                        && let Object::Tree(subtree) = Object::load(&entry.object_hash)?
                    {
                        stack.push((subtree, path.join(entry.name)));
                    }
                }
            }
        }

        Ok(index)
    }

    pub fn store_tree(&self) -> Result<String> {
        let mut stack = Vec::<(String, Tree)>::new();
        stack.push((
            String::from("root"),
            Tree {
                entries: Vec::new(),
            },
        ));

        for (name, entry) in &self.entries {
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
}
