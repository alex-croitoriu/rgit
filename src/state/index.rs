use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use wincode::{SchemaRead, SchemaWrite};

use crate::{
    state::{Object, Tree, TreeEntry, TreeEntryType},
    utils::{file_size, index_file_path},
};

#[derive(SchemaRead, SchemaWrite, Debug, Clone)]
pub struct IndexEntry {
    pub hash: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Default, Debug, Clone)]
pub struct Index {
    pub entries: BTreeMap<PathBuf, IndexEntry>,
}

#[derive(SchemaRead, SchemaWrite, Debug, Clone)]
pub struct SerializableIndex {
    entries: BTreeMap<String, IndexEntry>,
}

impl From<Index> for SerializableIndex {
    fn from(value: Index) -> Self {
        SerializableIndex {
            entries: value
                .entries
                .into_iter()
                // TODO: handle string conversion errors
                .map(|(p, e)| (p.to_string_lossy().to_string(), e))
                .collect(),
        }
    }
}

impl From<SerializableIndex> for Index {
    fn from(value: SerializableIndex) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|(p, e)| (PathBuf::from(p), e))
                .collect(),
        }
    }
}

impl Index {
    pub fn add(&mut self, name: &Path, entry: IndexEntry) {
        self.entries.insert(name.to_path_buf(), entry);
    }

    pub fn remove(&mut self, path: &Path) -> Result<()> {
        if self.entries.remove(path).is_some() {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to remove: file '{}' not in index",
                path.display()
            ))
        }
    }

    pub fn load(root: &Path) -> Result<Self> {
        let index_file_path = index_file_path(root);

        if file_size(&index_file_path)? == 0 {
            Ok(Self::default())
        } else {
            Ok(Self::from(
                wincode::deserialize::<SerializableIndex>(&fs::read(index_file_path)?)
                    .map_err(|_| anyhow!("Corrupt index file"))?,
            ))
        }
    }

    pub fn store(&self, root: &Path) -> Result<()> {
        Ok(fs::write(
            index_file_path(root),
            wincode::serialize(&SerializableIndex::from(self.clone()))?,
        )?)
    }

    pub fn load_from_commit(root: &Path, hash: &str) -> Result<Self> {
        let mut index = Self::default();
        let mut stack = Vec::new();

        if let Object::Commit(commit) = Object::load(root, hash)? {
            if let Object::Tree(tree) = Object::load(root, &commit.tree_hash)? {
                stack.push((tree, PathBuf::new()));
            }

            while let Some((tree, path)) = stack.pop() {
                for entry in tree.entries {
                    match entry.object_type {
                        TreeEntryType::Blob => {
                            index.entries.insert(
                                path.join(entry.name),
                                IndexEntry {
                                    hash: entry.object_hash,
                                    size: 0,
                                    mtime: 0,
                                },
                            );
                        }
                        TreeEntryType::Tree => {
                            if let Object::Tree(subtree) = Object::load(root, &entry.object_hash)? {
                                stack.push((subtree, path.join(entry.name)));
                            }
                        }
                    }
                }
            }

            Ok(index)
        } else {
            Err(anyhow!("Object is not a commit: {hash}"))
        }
    }

    pub fn store_tree(&self, root: &Path) -> Result<String> {
        let mut stack = Vec::<(String, Tree)>::new();
        stack.push((
            String::from("root"),
            Tree {
                entries: Vec::new(),
            },
        ));

        for (path, entry) in &self.entries {
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
                        object_type: TreeEntryType::Tree,
                        object_hash: Object::Tree(last.1).store(root)?,
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
                    object_type: TreeEntryType::Blob,
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
                    object_type: TreeEntryType::Tree,
                    object_hash: Object::Tree(last.1).store(root)?,
                    name: last.0,
                });
            }
        }

        if let Some(last) = stack.pop() {
            Ok(Object::Tree(last.1).store(root)?)
        } else {
            Err(anyhow!("Error at stack"))
        }
    }
}
