use std::{
    env,
    fs::File,
    io::BufRead,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

pub struct Repository {
    root: PathBuf,
    ignored: Vec<PathBuf>,
}

impl Repository {
    fn is_valid_root(path: &Path) -> bool {
        path.join(".rgit/objects").is_dir()
            && path.join(".rgit/refs/heads").is_dir()
            && path.join(".rgit/index").is_file()
            && path.join(".rgit/HEAD").is_file()
    }

    pub fn load() -> Result<Self> {
        let mut path = env::current_dir()?;
        while !Self::is_valid_root(&path) {
            if !path.pop() {
                return Err(anyhow!("Repository not found"));
            }
        }

        let mut ignored = Vec::new();
        let file = File::open(path.join(".rgitignore"))?;
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            ignored.push(PathBuf::from(line));
        }
        Ok(Repository {
            root: path,
            ignored: ignored,
        })
    }
}
