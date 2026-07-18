use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{Result, anyhow};

use crate::{commands, state::Repository};

pub struct Command;

pub struct Output {
    path: PathBuf,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Repository initialized: '{}'", self.path.display())
    }
}

impl commands::StatelessCommand for Command {
    type Args = ();
    type Output = Output;

    fn execute((): ()) -> Result<Self::Output> {
        let current_dir = env::current_dir()?;
        if Repository::is_valid_root(&current_dir) {
            return Err(anyhow!(
                "Repository not initialized: already exists in '{}'",
                current_dir.display()
            ));
        }

        fs::create_dir_all(current_dir.join(".rgit/objects"))?;
        fs::create_dir_all(current_dir.join(".rgit/refs/heads"))?;
        File::create_new(current_dir.join(".rgit/index"))?;
        File::create_new(current_dir.join(".rgit/HEAD"))?.write_all(b"ref: refs/heads/master")?;

        Ok(Output { path: current_dir })
    }
}
