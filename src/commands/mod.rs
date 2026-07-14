mod add;
mod branch;
mod checkout;
mod commit;
mod diff;
mod init;
mod merge;
mod status;

pub use add::AddCommand;
pub use branch::{BranchCreateCommand, BranchDeleteCommand, BranchListCommand};
pub use checkout::CheckoutCommand;
pub use commit::CommitCommand;
pub use diff::DiffCommand;
pub use init::InitCommand;
pub use merge::MergeCommand;
pub use status::StatusCommand;

use std::path::PathBuf;

use anyhow::Result;

use crate::state::Repository;

pub struct FileDiff {
    added: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
    modified: Vec<PathBuf>,
}

pub struct TextDiffEntry {
    path: PathBuf,
    change: String,
}

pub struct TextDiff {
    added: Vec<TextDiffEntry>,
    deleted: Vec<TextDiffEntry>,
    modified: Vec<TextDiffEntry>,
}
#[derive(Debug)]
pub enum CommandOutput {
    Empty,
    List(Vec<String>),
    TextDiff(String),
    FileDiff(String),
    Hash(String),
    Path(PathBuf),
}
pub trait Command {
    fn execute(&mut self, repository: &Repository) -> Result<CommandOutput>;
}

pub trait StatelessCommand {
    fn execute(&mut self) -> Result<CommandOutput>;
}
