use std::{fs, time::SystemTime};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Commit, Head, Index, Object, Repository, staged_changes},
    utils::{merge_head_file_path, trimmed_file_content},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    message: String,
    /// Allow commit with no changes
    #[arg(short, long)]
    allow_empty: bool,
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
    fn execute(repo: &Repository, args: Self::Args) -> Result<Self::Output> {
        let staged_changes = staged_changes(&repo.root)?;

        if staged_changes.is_empty() && !args.allow_empty {
            return Err(anyhow!("Commit not created: no changes"));
        }

        let mut head = Head::load(&repo.root)?;
        let head_hash = head.hash();

        let index = Index::load(&repo.root)?;
        let tree_hash = index.store_tree(&repo.root)?;
        let merge_head_path = merge_head_file_path(&repo.root);

        let mut parent_hashes = head_hash.into_iter().collect::<Vec<String>>();

        if merge_head_path.exists() {
            parent_hashes.push(trimmed_file_content(&merge_head_path)?);
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

        let commit_hash = commit.store(&repo.root)?;
        head.advance(&repo.root, &commit_hash)?;

        Ok(Output { hash: commit_hash })
    }
}
