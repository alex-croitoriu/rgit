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
    /// Initialize an empty repository
    Init,
    /// Show the working tree status
    Status,
    /// Add file contents to the index
    Add {
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Record changes to the repository
    Commit { message: String },
    /// Show changes between index and the previous commit or a branch
    Diff { target: Option<String> },
    /// Switch to another branch
    Checkout { target: String },
    /// Create or delete branches
    #[command(subcommand)]
    Branch(BranchSubcommands),
    /// Merge another branch into the current one
    Merge { target: String },
}

#[derive(Subcommand)]
pub enum BranchSubcommands {
    List,
    Create { name: String },
    Delete { name: String },
}
