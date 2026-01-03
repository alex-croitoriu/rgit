use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

use crate::utils::is_repository_root;

pub fn init() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    if is_repository_root(&current_dir) {
        return Err(anyhow!(
            "Repository already exists in '{}'",
            current_dir.display()
        ));
    }

    fs::create_dir_all(".rgit/objects")?;
    fs::create_dir_all(".rgit/refs/heads")?;
    File::create_new(".rgit/index")?;
    File::create_new(".rgit/HEAD")?.write_all(b"ref: refs/heads/master")?;

    Ok(current_dir)
}
