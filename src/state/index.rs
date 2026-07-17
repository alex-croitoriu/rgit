use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

use crate::{
    state::{Object, Tree, TreeEntry, TreeEntryType},
    utils::{index_file_path, objects_dir_path},
};

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug, Clone)]
pub struct IndexEntry {
    pub hash: String,
    pub size: u64,
    pub mtime: u64,
}

// #[derive(SchemaRead, SchemaWrite, Deserialize, Debug, Clone)]
// pub struct SerializableIndex {
//     pub entries: BTreeMap<String, IndexEntry>
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Index {
    pub entries: BTreeMap<PathBuf, IndexEntry>,
}

// TODO: replace json serialization with a better alternative (maybe wincode)
impl Index {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, name: &Path, entry: IndexEntry) {
        self.entries.insert(name.to_path_buf(), entry);
    }

    pub fn remove(&mut self, path: &Path) -> Result<()> {
        if self.entries.remove(path).is_some() {
            Ok(())
        } else {
            Err(anyhow!("File not in index: '{}'", path.display()))
        }
    }

    pub fn read_from_file(path: &Path) -> Result<Self> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let index = serde_json::from_reader(reader).unwrap_or(Self::new());

        Ok(index)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let file = OpenOptions::new().write(true).truncate(true).open(path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self)?;

        Ok(())
    }

    pub fn load(root: &Path) -> Result<Self> {
        Self::read_from_file(&index_file_path(root))
    }

    pub fn load_from_commit(root: &Path, hash: &str) -> Result<Self> {
        Self::read_from_commit(&objects_dir_path(root), hash)
    }

    pub fn store(&self, root: &Path) -> Result<()> {
        self.write_to_file(&index_file_path(root))
    }

    pub fn read_from_commit(objects_dir: &Path, hash: &str) -> Result<Self> {
        let mut index = Self::new();
        let mut stack = Vec::new();

        if let Object::Commit(commit) = Object::read(objects_dir, hash)? {
            if let Object::Tree(tree) = Object::read(objects_dir, &commit.tree_hash)? {
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
                            if let Object::Tree(subtree) =
                                Object::read(objects_dir, &entry.object_hash)?
                            {
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

    pub fn write_tree(&self, objects_dir: &Path) -> Result<String> {
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
                        object_type: TreeEntryType::Tree,
                        object_hash: Object::Tree(last.1).write(objects_dir)?,
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
                    object_hash: Object::Tree(last.1).write(objects_dir)?,
                    name: last.0,
                });
            }
        }

        if let Some(last) = stack.pop() {
            Ok(Object::Tree(last.1).write(objects_dir)?)
        } else {
            Err(anyhow!("Error at stack"))
        }
    }
}
