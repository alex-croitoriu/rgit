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
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(wincode::serialize(self)?)
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(wincode::deserialize(data)?)
    }

    fn hash(&self) -> Result<String> {
        use std::fmt::Write;

        let mut hasher = Sha1::new();
        hasher.update(self.serialize()?);
        let hash = hasher.finalize();
        let mut digest = String::new();
        for byte in hash {
            write!(digest, "{byte:02x}")?;
        }

        Ok(digest)
    }

    fn compress(data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;

        Ok(encoder.finish()?)
    }

    fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        Ok(decompressed)
    }

    pub fn load(root: &Path, hash: &str) -> Result<Self> {
        let Some((dir, file)) = hash.split_at_checked(2) else {
            return Err(anyhow!("Invalid hash: {hash}"));
        };

        let path = objects_dir_path(root).join(dir).join(file);

        let compressed = fs::read(path)?;

        Self::deserialize(&Self::decompress(&compressed)?)
    }

    pub fn store(&self, root: &Path) -> Result<String> {
        let hash = self.hash()?;
        let (dir_name, file_name) = hash.split_at(2);

        let dir_path = objects_dir_path(root).join(dir_name);
        let file_path = dir_path.join(file_name);

        if !file_path.exists() {
            fs::create_dir_all(dir_path)?;
            let compressed = Self::compress(&self.serialize()?)?;
            fs::write(&file_path, compressed)?;
        }

        Ok(hash)
    }

    pub fn blob_bytes(&self) -> Result<Vec<u8>> {
        if let Object::Blob(blob) = self {
            Ok(blob.bytes.clone())
        } else {
            Err(anyhow!("Object is not a blob"))
        }
    }

    pub fn blob_text(&self) -> Result<Option<String>> {
        if let Ok(text) = String::from_utf8(self.blob_bytes()?) {
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }
}
