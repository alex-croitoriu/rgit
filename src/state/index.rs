use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

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

    pub fn add(&mut self, name: &Path, entry: IndexEntry) -> Option<IndexEntry> {
        self.entries.insert(name.to_path_buf(), entry)
    }

    pub fn remove(&mut self, path: &Path) -> Result<()> {
        if self.entries.remove(path).is_some() {
            Ok(())
        } else {
            Err(anyhow!("File not in index: '{}'", path.display()))
        }
    }
}
