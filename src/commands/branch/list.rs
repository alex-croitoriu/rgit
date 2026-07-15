use anyhow::{Result, anyhow};

use crate::{commands, state::Repository};

pub struct Command;

pub struct Output {
     branches: Vec<String>,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(head) = self.branches.first() {
            write!(f, "{head} -> HEAD")?;
            for branch in self.branches.iter().skip(1) {
                writeln!(f)?;
                write!(f, "{branch}")?;
            }
        } else {
            write!(f, "No branches found")?;
        }

        Ok(())
    }
}

impl commands::Command for Command {
    type Args = ();
    type Output = Output;

    fn execute(repository: &Repository, _: Self::Args) -> Result<Self::Output> {
        let mut branches = Vec::new();
        let head = repository.current_branch_name()?;
        branches.push(head.clone());

        for entry in repository.heads_path().read_dir()?.flatten() {
            let branch = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid UTF-8"))?;
            if branch != head {
                branches.push(branch);
            }
        }
        Ok(Output { branches })
    }
}
