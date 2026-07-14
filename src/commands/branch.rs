use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands::{Command, CommandOutput},
    state::Repository,
};

pub struct BranchListCommand {}

pub struct BranchCreateCommand {
    pub name: String,
}

pub struct BranchDeleteCommand {
    pub name: String,
}

impl Command for BranchListCommand {
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let mut branches = Vec::new();
        let current_branch_name = repository.current_branch_name()?;
        branches.push(format!("{current_branch_name} -> HEAD"));

        for entry in repository.heads_path().read_dir()?.flatten() {
            let file_name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid UTF-8"))?;
            if file_name != current_branch_name {
                branches.push(file_name);
            }
        }
        Ok(CommandOutput::List(branches))
    }
}

impl Command for BranchCreateCommand {
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let branch_path = repository.branch_path(&self.name);
        if branch_path.exists() {
            return Err(anyhow!("Branch already exists: '{}'", self.name));
        }

        let current_branch_path = repository.current_branch_path()?;
        let commit_hash = fs::read_to_string(current_branch_path)
            .map_err(|_| anyhow!("Branch not created: no commits yet"))?;
        fs::write(branch_path, commit_hash)?;

        Ok(CommandOutput::Empty)
    }
}

impl Command for BranchDeleteCommand {
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let branch_path = repository.branch_path(&self.name);
        if !branch_path.exists() {
            return Err(anyhow!("Branch doesn't not exist: '{}'", self.name));
        }

        let current_branch_path = repository.current_branch_path()?;
        if current_branch_path == branch_path {
            return Err(anyhow!("Unable to delete the current branch"));
        }

        fs::remove_file(branch_path)?;

        Ok(CommandOutput::Empty)
    }
}
