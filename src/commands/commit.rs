use std::fs;
use std::time::SystemTime;

use anyhow::Result;

use crate::{
    commands::{Command, CommandOutput},
    state::{Commit, Object, Repository},
};

pub struct CommitCommand {
    pub message: String,
}

impl Command for CommitCommand {
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput> {
        let index = repository.load_index()?;

        let tree_hash = repository.store_index_tree(&index)?;

        let merge_head_path = repository.merge_head_path();
        let mut parent_hashes = if let Ok(path) = repository.current_branch_path()
            && path.exists()
        {
            vec![fs::read_to_string(path)?]
        } else {
            Vec::new()
        };

        if merge_head_path.exists() {
            parent_hashes.push(fs::read_to_string(&merge_head_path)?);
            fs::remove_file(merge_head_path)?;
        }

        let commit = Object::Commit(Commit {
            message: self.message.clone(),
            parent_hashes,
            tree_hash,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        });

        let commit_hash = repository.store_object(&commit)?;

        fs::write(repository.current_branch_path()?, &commit_hash)?;

        Ok(CommandOutput::Hash(commit_hash))
    }
}
