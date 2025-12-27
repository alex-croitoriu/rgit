use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rgit")]
#[clap(disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create an empty repository
    Init,
    /// Show the working tree status
    Status,
    /// Add file contents to the index
    Add {
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Record changes to the repository
    Commit {
        name: String,
        message: String,
    },
    /// Show changes between objects (commits or branches)
    Diff {
        obj1: String,
        obj2: String,
    },
    /// Create or delete branches
    #[command(subcommand)]
    Branch(BranchSubcommands),
    /// Merge another branch into the current one
    Merge {
        branch: String
    },
}

#[derive(Subcommand)]
pub enum BranchSubcommands {
    Create,
    Delete,
}
