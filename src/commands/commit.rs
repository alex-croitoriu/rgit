use std::{fs, time::SystemTime};

use anyhow::Result;

use crate::{
    commands,
    state::{Commit, Object, Repository},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    message: String,
}

pub struct Output {
    hash: String,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Commit: {}", self.hash)?;
        Ok(())
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;
    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let index = repository.load_index()?;

        let tree_hash = repository.store_index_tree(&index)?;

        let merge_head_path = repository.merge_head_file_path();
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
            message: args.message,
            parent_hashes,
            tree_hash,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        });

        let commit_hash = repository.store_object(&commit)?;

        fs::write(repository.current_branch_path()?, &commit_hash)?;

        Ok(Output { hash: commit_hash })
    }
}
