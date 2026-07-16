use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Head, Repository},
};

pub struct Command;

pub struct Output {
    head: Head,
    branches: Vec<String>,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Head::Branch { name } => {
                write!(f, "{name} -> HEAD")?;
                for branch in self.branches.iter().filter(|branch| branch != &name) {
                    write!(f, "\n{branch}")?;
                }
            }
            Head::Commit { hash } => {
                write!(f, "{hash} -> Detached HEAD")?;
                for branch in &self.branches {
                    write!(f, "\n{branch}")?;
                }
            }
        }

        Ok(())
    }
}

impl commands::Command for Command {
    type Args = ();
    type Output = Output;

    fn execute(repository: &Repository, (): ()) -> Result<Self::Output> {
        let mut branches = Vec::new();
        let head = repository.head()?;

        for entry in repository.heads_dir_path().read_dir()?.flatten() {
            let branch = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid UTF-8"))?;
            branches.push(branch);
        }

        Ok(Output { head, branches })
    }
}
