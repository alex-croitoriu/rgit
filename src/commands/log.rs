use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use anyhow::{Result, anyhow};

use crate::{
    commands,
    state::{Head, Object, Repository},
    utils::{date_from_timestamp, heads_dir_path, trimmed_file_content},
};

pub struct Command;

pub struct Output {
    head: Head,
    commits: Vec<CommitEntry>,
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(commit) = self.commits.first() else {
            return write!(f, "No commits yet");
        };

        let branches = commit.branches.clone().into_iter().flatten();

        match &self.head {
            Head::Branch { name, .. } => {
                writeln!(
                    f,
                    "{} ({})",
                    commit.hash,
                    std::iter::once(format!("HEAD -> {name}"))
                        .chain(branches.filter(|b| b != name))
                        .collect::<Vec<String>>()
                        .join(", ")
                )?;
            }
            Head::Detached { .. } => {
                writeln!(
                    f,
                    "{} ({})",
                    commit.hash,
                    std::iter::once(String::from("Detached HEAD"))
                        .chain(branches)
                        .collect::<Vec<String>>()
                        .join(", ")
                )?;
            }
        }

        writeln!(
            f,
            "{:<10} {}",
            "Date:",
            date_from_timestamp(commit.timestamp)
        )?;
        write!(f, "{:<10} {}", "Message:", commit.message)?;

        for commit in self.commits.iter().skip(1) {
            write!(f, "\n\n")?;
            if let Some(branches) = &commit.branches {
                writeln!(f, "{} ({})", commit.hash, branches.join(", "))?;
            } else {
                writeln!(f, "{}", commit.hash)?;
            }

            writeln!(
                f,
                "{:<10} {}",
                "Date:",
                date_from_timestamp(commit.timestamp)
            )?;
            write!(f, "{:<10} {}", "Message:", commit.message)?;
        }

        Ok(())
    }
}

#[derive(Eq, PartialEq)]
struct CommitEntry {
    hash: String,
    message: String,
    timestamp: u64,
    branches: Option<Vec<String>>,
}

impl std::cmp::PartialOrd for CommitEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for CommitEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.hash.cmp(&other.hash))
    }
}

impl commands::Command for Command {
    type Args = ();
    type Output = Output;

    fn execute(repo: &Repository, (): ()) -> Result<Self::Output> {
        let mut commits = BinaryHeap::new();
        let mut branches = HashMap::<String, Vec<String>>::new();

        for entry in heads_dir_path(&repo.root).read_dir()?.flatten() {
            let commit_hash = trimmed_file_content(&entry.path())?;
            let branch_name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow!("Invalid UTF-8"))?;
            branches
                .entry(commit_hash)
                .and_modify(|e| e.push(branch_name.clone()))
                .or_insert(vec![branch_name]);
        }
        let head = Head::load(&repo.root)?;

        let mut queue = head.hash().into_iter().collect::<VecDeque<String>>();
        let mut visited = HashSet::new();

        while let Some(hash) = queue.pop_front() {
            if !visited.insert(hash.clone()) {
                continue;
            }
            if let Object::Commit(commit) = Object::load(&repo.root, &hash)? {
                for parent_hash in commit.parent_hashes {
                    queue.push_back(parent_hash);
                }
                commits.push(CommitEntry {
                    hash: hash.clone(),
                    message: commit.message,
                    timestamp: commit.timestamp,
                    branches: branches.get(&hash).cloned(),
                });
            }
        }

        let mut commits = commits.into_sorted_vec();
        commits.reverse();

        Ok(Output { head, commits })
    }
}
