use std::collections::{BinaryHeap, HashMap, VecDeque};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Object, Repository, resolve_head_hash},
    utils::{heads_dir_path, trimmed_file_content},
};

pub struct Command;

#[derive(clap::Args)]
pub struct Args {
    #[arg(required = true)]
    paths: Vec<String>,
}

pub struct Output {
    commits: Vec<CommitEntry>,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(commit) = self.commits.first() else {
            return write!(f, "No commits yet");
        };

        if let Some(branch) = &commit.branch {
            writeln!(f, "{} (HEAD -> {})", commit.hash, branch)?;
        } else {
            write!(f, "{} (Detached HEAD)", commit.hash)?;
        }

        writeln!(f, "{:<10} {}", "Date:", commit.timestamp)?;
        writeln!(f, "{:<10} {}", "Message:", commit.message)?;

        for commit in self.commits.iter().skip(1) {
            writeln!(f)?;
            if let Some(branch) = &commit.branch {
                writeln!(f, "{} ({})", commit.hash, branch)?;
            } else {
                writeln!(f, "{}", commit.hash)?;
            }

            writeln!(f, "{:<10} {}", "Date:", commit.timestamp)?;
            writeln!(f, "{:<10} {}", "Message:", commit.message)?;
        }

        Ok(())
    }
}

#[derive(Eq, PartialEq)]
struct CommitEntry {
    hash: String,
    message: String,
    timestamp: u64,
    branch: Option<String>,
}

impl std::cmp::Ord for CommitEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl std::cmp::PartialOrd for CommitEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}

impl commands::Command for Command {
    type Args = ();
    type Output = Output;

    fn execute(repo: &Repository, (): ()) -> Result<Self::Output> {
        let mut commits = BinaryHeap::new();
        let mut branches = HashMap::new();

        for entry in heads_dir_path(&repo.root).read_dir()?.flatten() {
            let commit_hash = trimmed_file_content(&entry.path())?;
            let branch_name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid UTF-8"))?;
            branches.insert(commit_hash, branch_name);
        }
        let head = resolve_head_hash(&repo.root)?;

        let mut queue = head.into_iter().collect::<VecDeque<String>>();

        while let Some(hash) = queue.pop_front() {
            if let Object::Commit(commit) = Object::load(&repo.root, &hash)? {
                queue.extend(commit.parent_hashes.clone());
                commits.push(CommitEntry {
                    hash: hash.clone(),
                    message: commit.message,
                    timestamp: commit.timestamp,
                    branch: branches.get(&hash).cloned(),
                });
            }
        }

        let mut commits = commits.into_sorted_vec();
        commits.reverse();

        Ok(Output { commits })
    }
}
