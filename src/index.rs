use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::object_store::Object;
use crate::utils::get_repository_root;

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
        let index: Index = serde_json::from_reader(reader).unwrap_or(Index::new());

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
}
