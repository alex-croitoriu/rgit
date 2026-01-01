use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

use crate::utils::get_repository_root;

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexEntry {
    pub hash: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Index {
    pub entries: BTreeMap<String, IndexEntry>,
}

impl Index {
    pub fn read() -> Result<Self> {
        let root = get_repository_root()?;
        let file = OpenOptions::new()
            .read(true)
            .open(root.join(".rgit/index"))?;
        let reader = BufReader::new(file);
        let index: Index = serde_json::from_reader(reader).unwrap_or(Index {
            entries: BTreeMap::new(),
        });

        Ok(index)
    }

    pub fn write(index: Self) -> Result<()> {
        let root = get_repository_root()?;
        let file = OpenOptions::new()
            .write(true)
            .open(root.join(".rgit/index"))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &index)?;

        Ok(())
    }

    pub fn add(name: String, entry: IndexEntry) -> Result<()> {
        let mut index = Self::read()?;
        index.entries.insert(name, entry);
        Self::write(index)?;

        Ok(())
    }

    // pub fn remove() -> Result<()> {
    //     Ok(())
    // }
}
