use anyhow::Result;

use crate::state::Repository;

pub mod add;
pub mod branch;
pub mod commit;
pub mod diff;
pub mod init;
pub mod merge;
pub mod rm;
pub mod status;
pub mod switch;

pub trait Command {
    type Args;
    type Output;

    fn execute(repo: &Repository, args: Self::Args) -> Result<Self::Output>;
}

pub trait StatelessCommand {
    type Args;
    type Output;

    fn execute(input: Self::Args) -> Result<Self::Output>;
}
