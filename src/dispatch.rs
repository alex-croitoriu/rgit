use anyhow::Result;

use crate::{
    args::{BranchSubcommands, Commands},
    commands::*,
    state::Repository,
};

pub fn dispatch(command: Commands) -> Result<Box<dyn std::fmt::Display>> {
    match command {
        Commands::Init => {
            let output = init::Command::execute(())?;
            Ok(Box::new(output))
        }
        Commands::Status => {
            let repository = Repository::load()?;
            let output = status::Command::execute(&repository, ())?;
            Ok(Box::new(output))
        }
        Commands::Add(args) => {
            let repository = Repository::load()?;
            let output = add::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
        Commands::Commit(args) => {
            let repository = Repository::load()?;
            let output = commit::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
        Commands::Log => {
            // let repository = Repository::load()?;
            // let output = status::Command::execute(&repository, ())?;
            Ok(Box::new(String::from("not implemented")))
        }
        Commands::Diff(args) => {
            let repository = Repository::load()?;
            let output = diff::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
        Commands::Switch(args) => {
            let repository = Repository::load()?;
            let output = switch::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
        Commands::Branch(BranchSubcommands::List) => {
            let repository = Repository::load()?;
            let output = branch::list::Command::execute(&repository, ())?;
            Ok(Box::new(output))
        }
        Commands::Branch(BranchSubcommands::Create(args)) => {
            let repository = Repository::load()?;
            let output = branch::create::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
        Commands::Branch(BranchSubcommands::Delete(args)) => {
            let repository = Repository::load()?;
            let output = branch::delete::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
        Commands::Merge(args) => {
            let repository = Repository::load()?;
            let output = merge::Command::execute(&repository, args)?;
            Ok(Box::new(output))
        }
    }
}
