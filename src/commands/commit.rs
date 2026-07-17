use std::{fs, time::SystemTime};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Commit, Head, Index, Object, Repository, current_branch_path, head, head_hash},
    utils::{merge_head_file_path, objects_dir_path},
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
        write!(f, "Commit: {}", self.hash)
    }
}

impl commands::Command for Command {
    type Args = Args;
    type Output = Output;
    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = repository.staged_changes()?;

        if staged_changes.is_empty() {
            return Err(anyhow!("Commit not created: no changes"));
        }
        if let Head::Commit { .. } = head(&repository.root)? {
            return Err(anyhow!("Commit not created: Detached HEAD"));
        }

        let index = Index::load(&repository.root)?;
        let tree_hash = index.write_tree(&objects_dir_path(&repository.root))?;
        let merge_head_path = merge_head_file_path(&repository.root);

        let mut parent_hashes = if let Some(head_hash) = head_hash(&repository.root)? {
            vec![head_hash]
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

        let commit_hash = commit.store(&repository.root)?;

        fs::write(current_branch_path(&repository.root)?, &commit_hash)?;

        Ok(Output { hash: commit_hash })
    }
}
