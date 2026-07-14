use std::fs;

use anyhow::{Result, anyhow};

use crate::{
    commands::{BranchCreateCommand, Command, CommandOutput},
    state::Repository,
    utils::{get_staged_changes, get_unstaged_changes, update_working_tree},
};

pub struct CheckoutCommand {
    pub target: String,
}

impl Command for CheckoutCommand {
    // TODO: refactor this garbage
    // TODO: add checkout on commits
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let staged_changes = get_staged_changes(repository)?;
        let unstaged_changes = get_unstaged_changes(repository)?;

        if !(staged_changes.0.is_empty()
            && staged_changes.1.is_empty()
            && staged_changes.2.is_empty()
            && unstaged_changes.0.is_empty()
            && unstaged_changes.1.is_empty()
            && unstaged_changes.2.is_empty())
        {
            return Err(anyhow!("Unable to checkout: uncommited changes"));
        }

        let branch_path = repository.branch_path(&self.target);
        if branch_path == repository.current_branch_path()? {
            return Err(anyhow!("Unable to checkout: already on that branch"));
        }

        if !branch_path.exists() {
            BranchCreateCommand {
                name: self.target.clone(),
            }
            .execute(repository)?;
        }

        let target_hash = fs::read_to_string(branch_path)?;
        let current_index = repository.load_index()?;
        let mut target_index = repository.load_index_from_commit(&target_hash)?;

        update_working_tree(repository, &mut target_index, &current_index)?;
        repository.store_index(&target_index)?;

        fs::write(
            repository.head_path(),
            format!("ref: refs/heads/{}", self.target),
        )?;

        Ok(CommandOutput::Empty)
    }
}
