use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

pub struct Repository {
    pub root: PathBuf,
}

impl Repository {
    pub fn is_valid_root(path: &Path) -> bool {
        path.join(".rgit/objects").is_dir()
            && path.join(".rgit/refs/heads").is_dir()
            && path.join(".rgit/index").is_file()
            && path.join(".rgit/HEAD").is_file()
    }

    pub fn load() -> Result<Self> {
        let mut root = env::current_dir()?;
        while !Self::is_valid_root(&root) {
            if !root.pop() {
                return Err(anyhow!("Repository not found"));
            }
        }

        Ok(Self { root })
    }
}
