use clap::{Parser, Subcommand};

use crate::commands::*;

#[derive(Parser)]
#[command(name = "rgit")]
#[clap(disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize an empty repository
    Init,
    /// Show the working tree status
    Status,
    /// Add file contents to the index
    Add(add::Args),
    /// Record changes to the repository
    Commit(commit::Args),
    /// Show commit history
    Log,
    /// Show changes between the index and the last commit or between the index and a branch
    Diff(diff::Args),
    /// Switch to another branch
    Switch(switch::Args),
    /// List, create or delete branches
    #[command(subcommand)]
    Branch(BranchSubcommands),
    /// Merge another branch into the current one
    Merge(merge::Args),
}

#[derive(Subcommand)]
pub enum BranchSubcommands {
    List,
    Create(branch::create::Args),
    Delete(branch::delete::Args),
}
