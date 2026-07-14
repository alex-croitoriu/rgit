mod args;
mod commands;
mod state;
mod utils;

use args::{BranchSubcommands, Cli, Commands};
use clap::Parser;

use crate::{commands::*, state::Repository};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            let mut command = InitCommand {};
            match command.execute() {
                Ok(output) => println!("{:?}", output),
                Err(e) => println!("{e}"),
            }
        }
        command => {
            let repository = match Repository::load() {
                Ok(repository) => repository,
                Err(e) => {
                    println!("{e}");
                    return;
                }
            };

            let mut command: Box<dyn Command> = match command {
                Commands::Init => unreachable!(),
                Commands::Status => Box::new(StatusCommand {}),
                Commands::Add { paths } => Box::new(AddCommand { paths }),
                Commands::Commit { message } => Box::new(CommitCommand { message }),
                Commands::Diff { target } => Box::new(DiffCommand { target }),
                Commands::Branch(BranchSubcommands::List) => Box::new(BranchListCommand {}),
                Commands::Branch(BranchSubcommands::Create { name }) => {
                    Box::new(BranchCreateCommand { name })
                }
                Commands::Branch(BranchSubcommands::Delete { name }) => {
                    Box::new(BranchDeleteCommand { name })
                }
                Commands::Checkout { target } => Box::new(CheckoutCommand { target }),
                Commands::Merge { target } => Box::new(MergeCommand { target }),
            };

            match command.execute(&repository) {
                Ok(output) => println!("{:?}", output),
                Err(e) => println!("{e}"),
            }
        }
    }
}
