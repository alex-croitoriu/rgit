use std::path::PathBuf;

use anyhow::Result;

use crate::state::Repository;

pub mod add;
pub mod branch;
pub mod checkout;
pub mod commit;
pub mod diff;
pub mod init;
pub mod merge;
pub mod status;

// TODO: change where these leve
pub struct FileDiff {
    pub added: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
}

impl FileDiff {
    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.deleted.is_empty() && self.modified.is_empty()
    }
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

pub trait Command {
    type Args;
    type Output;

    fn execute(repository: &Repository, args: Self::Args) -> Result<Self::Output>;
}

pub trait StatelessCommand {
    type Args;
    type Output;

    fn execute(input: Self::Args) -> Result<Self::Output>;
}
