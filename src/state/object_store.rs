use std::fs;
use std::io::prelude::*;

use anyhow::Result;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use wincode::{SchemaRead, SchemaWrite};

use crate::utils::get_repository_root;

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug)]
pub struct Blob {
    pub content: Vec<u8>,
}

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug)]

pub struct TreeEntry {
    pub object_type: String,
    pub object_hash: String,
    pub name: String,
}

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug)]

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug)]

pub struct Commit {
    pub tree_hash: String,
    pub message: String,
    pub timestamp: u64,
    pub parent_hashes: Vec<String>,
}

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug)]

pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl Object {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(wincode::serialize(self)?)
    }

    fn deserialize(data: Vec<u8>) -> Result<Self> {
        Ok(wincode::deserialize(data.as_slice())?)
    }

    fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(self.serialize()?);
        let result = hasher.finalize();

        Ok(result.iter().map(|b| format!("{b:02x}")).collect())
    }

    fn compress(data: Vec<u8>) -> Result<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data.as_slice())?;

        Ok(encoder.finish()?)
    }

    fn decompress(data: Vec<u8>) -> Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        Ok(decompressed)
    }

    pub fn store(&self) -> Result<String> {
        let hash = self.hash()?;
        let root = get_repository_root()?;
        let (dir_name, file_name) = hash.split_at(2);

        let compressed = Self::compress(self.serialize()?)?;

        let dir_path = root.join(".rgit/objects").join(dir_name);
        let file_path = dir_path.join(file_name);
        fs::create_dir_all(dir_path)?;

        if !file_path.exists() {
            fs::write(&file_path, compressed)?;
        }

        Ok(hash)
    }

    pub fn load(hash: &str) -> Result<Self> {
        let root = get_repository_root()?;
        let (dir, file) = hash.split_at(2);
        let path = root.join(".rgit/objects").join(dir).join(file);

        let compressed = fs::read(path)?;

        Ok(Self::deserialize(Self::decompress(compressed)?)?)
    }
}
