use std::env;
use std::fs::{self, File};
use std::io::Write;

use anyhow::{Result, anyhow};

use crate::commands::CommandOutput;
use crate::{commands::StatelessCommand, state::Repository};

pub struct InitCommand {}

impl StatelessCommand for InitCommand {
    fn execute(&mut self) -> Result<CommandOutput> {
        let current_dir = env::current_dir()?;
        if Repository::is_valid_root(&current_dir) {
            return Err(anyhow!(
                "Repository already exists in '{}'",
                current_dir.display()
            ));
        }

        fs::create_dir_all(".rgit/objects")?;
        fs::create_dir_all(".rgit/refs/heads")?;
        File::create_new(".rgit/index")?;
        File::create_new(".rgit/HEAD")?.write_all(b"ref: refs/heads/master")?;

        Ok(CommandOutput::Path(current_dir))
    }
}
