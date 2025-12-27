use anyhow::Result;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn init() -> Result<PathBuf> {
    fs::create_dir_all(".rgit/objects")?;
    fs::create_dir_all(".rgit/refs/heads")?;

    File::create(".rgit/index")?;

    File::create(".rgit/HEAD")?.write_all(b"ref: refs/heads/master")?;

    Ok(env::current_dir()?)
}
