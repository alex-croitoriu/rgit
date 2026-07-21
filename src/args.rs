use clap::{Parser, Subcommand};

use crate::commands::{add, branch, commit, diff, merge, rm, switch};

#[derive(Parser)]
#[command(name = "rgit")]
#[clap(disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize empty repository
    Init,
    /// Show working tree status
    Status,
    /// Add file contents to index
    Add(add::Args),
    /// Remove file contents from index
    Rm(rm::Args),
    /// Record changes to repository
    Commit(commit::Args),
    /// Show commit history
    Log,
    /// Show changes between index and last commit or index and target
    Diff(diff::Args),
    /// Switch to target
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
