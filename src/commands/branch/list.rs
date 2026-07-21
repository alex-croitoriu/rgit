use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Head, Repository},
    utils::heads_dir_path,
};

pub struct Command;

pub struct Output {
    head: Head,
    branches: Vec<String>,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Head::Branch { name, .. } => {
                write!(f, "HEAD -> {name}")?;
                for branch in self.branches.iter().filter(|b| b != &name) {
                    write!(f, "\n{branch}")?;
                }
            }
            Head::Commit { hash } => {
                write!(f, "Detached HEAD -> {hash}")?;
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

    fn execute(repo: &Repository, (): ()) -> Result<Self::Output> {
        let mut branches = Vec::new();
        let head = Head::load(&repo.root)?;

        for entry in heads_dir_path(&repo.root).read_dir()?.flatten() {
            let branch = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid UTF-8"))?;
            branches.push(branch);
        }

        Ok(Output { head, branches })
    }
}
