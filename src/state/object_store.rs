use std::io::prelude::*;

use anyhow::Result;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use wincode::{SchemaRead, SchemaWrite};

#[derive(SchemaRead, SchemaWrite, Serialize, Deserialize, Debug)]
pub struct Blob {
    pub bytes: Vec<u8>,
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
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(wincode::serialize(self)?)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(wincode::deserialize(data)?)
    }

    pub fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(self.serialize()?);
        let result = hasher.finalize();

        Ok(result.iter().map(|b| format!("{b:02x}")).collect())
    }

    pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;

        Ok(encoder.finish()?)
    }

    pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        Ok(decompressed)
    }
}
