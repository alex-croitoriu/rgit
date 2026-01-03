use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs;

use crate::utils::get_repository_root;

#[derive(Serialize, Deserialize, Debug)]
pub struct Blob {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TreeEntry {
    pub object_type: String,
    pub object_hash: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
    pub tree_hash: String,
    pub message: String,
    pub timestamp: u64,
    pub parent_hashes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl Object {
    fn hash(&self) -> Result<String> {
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

    pub fn load(hash: &str) -> Result<Object> {
        let root = get_repository_root()?;
        let (dir, file) = hash.split_at(2);
        let path = root.join(".rgit/objects").join(dir).join(file);

        let content = fs::read_to_string(path)?;
        let obj: Object = serde_json::from_str(&content)?;
        Ok(obj)
    }
}
