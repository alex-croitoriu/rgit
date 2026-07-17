use std::{fs, io::prelude::*, path::Path};

use anyhow::{Result, anyhow};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use wincode::{SchemaRead, SchemaWrite};

use crate::utils::objects_dir_path;

#[derive(SchemaRead, SchemaWrite, Debug)]
pub struct Blob {
    pub bytes: Vec<u8>,
}

#[derive(SchemaRead, SchemaWrite, Debug)]
pub enum TreeEntryType {
    Blob,
    Tree,
}

#[derive(SchemaRead, SchemaWrite, Debug)]
pub struct TreeEntry {
    pub object_type: TreeEntryType,
    pub object_hash: String,
    pub name: String,
}

#[derive(SchemaRead, SchemaWrite, Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

#[derive(SchemaRead, SchemaWrite, Debug)]
pub struct Commit {
    pub tree_hash: String,
    pub message: String,
    pub timestamp: u64,
    pub parent_hashes: Vec<String>,
}

#[derive(SchemaRead, SchemaWrite, Debug)]
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

        Ok(result
            .iter()
            .fold(String::new(), |acc, byte| format!("{acc}{byte:02x}")))
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

    pub fn read(objects_dir: &Path, hash: &str) -> Result<Self> {
        let (dir, file) = hash.split_at(2);
        let path = objects_dir.join(dir).join(file);

        let compressed = fs::read(path)?;

        Self::deserialize(&Self::decompress(&compressed)?)
    }

    pub fn write(&self, objects_dir: &Path) -> Result<String> {
        let hash = self.hash()?;
        let (dir_name, file_name) = hash.split_at(2);

        let dir_path = objects_dir.join(dir_name);
        let file_path = dir_path.join(file_name);

        if !file_path.exists() {
            fs::create_dir_all(dir_path)?;
            let compressed = Self::compress(&self.serialize()?)?;
            fs::write(&file_path, compressed)?;
        }

        Ok(hash)
    }

    pub fn load(root: &Path, hash: &str) -> Result<Self> {
        Self::read(&objects_dir_path(root), hash)
    }

    pub fn store(&self, root: &Path) -> Result<String> {
        self.write(&objects_dir_path(root))
    }
}

pub fn read_blob_bytes(objects_dir: &Path, hash: &str) -> Result<Vec<u8>> {
    if let Object::Blob(blob) = Object::read(objects_dir, hash)? {
        Ok(blob.bytes)
    } else {
        Err(anyhow!("Object is not a blob: {hash}"))
    }
}

pub fn read_blob_text(objects_dir: &Path, hash: &str) -> Result<String> {
    if let Ok(text) = String::from_utf8(read_blob_bytes(objects_dir, hash)?) {
        Ok(text)
    } else {
        Ok(String::from("Binary file"))
    }
}
