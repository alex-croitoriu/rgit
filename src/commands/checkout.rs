use std::fs;

use anyhow::{Result, anyhow};

use crate::utils::get_repository_root;

pub fn checkout(branch: &str) -> Result<()> {
    // let root = get_repository_root()?;
    // if let Some(head) = fs::read_to_string(root.join(".rgit/HEAD"))?.strip_prefix("ref: ") {
    //     fs::write(root.join(".rgit").join(head), hash.as_bytes())?;
    //     Ok(())
    // } else {
    //     Err(anyhow!("Corrupt HEAD file"))
    // }
    Ok(())
}
