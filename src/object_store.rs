use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs;

use crate::utils::get_repository_root;

#[derive(Serialize, Deserialize)]
pub struct Blob {
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct TreeEntry {
    pub object_type: String,
    pub object_hash: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub tree_hash: String,
    pub message: String,
    pub timestamp: u64,
    pub parent_hashes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl Object {
    pub fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        let json = serde_json::to_string(self)?;
        hasher.update(json);
        let result = hasher.finalize();

        Ok(result.iter().map(|b| format!("{b:02x}")).collect())
    }

    pub fn store(&self) -> Result<String> {
        let root = get_repository_root()?;
        let hash = self.hash()?;
        let (dir_name, file_name) = hash.split_at(2);
        let json = serde_json::to_string(self)?;

        let dir_path = root.join(".rgit/objects").join(dir_name);
        let file_path = dir_path.join(file_name);
        fs::create_dir_all(dir_path)?;

        if !file_path.exists() {
            fs::write(&file_path, json)?;
        }

        Ok(hash)
    }
}
