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
        command => {
            let repo = Repository::load()?;

            match command {
                Commands::Init => unreachable!(),
                Commands::Status => {
                    let output = status::Command::execute(&repo, ())?;
                    Ok(Box::new(output))
                }
                Commands::Add(args) => {
                    let output = add::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
                Commands::Commit(args) => {
                    let output = commit::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
                Commands::Log => {
                    // let output = status::Command::execute(&repo, ())?;
                    Ok(Box::new(String::from("not implemented")))
                }
                Commands::Diff(args) => {
                    let output = diff::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
                Commands::Switch(args) => {
                    let output = switch::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
                Commands::Branch(BranchSubcommands::List) => {
                    let output = branch::list::Command::execute(&repo, ())?;
                    Ok(Box::new(output))
                }
                Commands::Branch(BranchSubcommands::Create(args)) => {
                    let output = branch::create::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
                Commands::Branch(BranchSubcommands::Delete(args)) => {
                    let output = branch::delete::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
                Commands::Merge(args) => {
                    let output = merge::Command::execute(&repo, args)?;
                    Ok(Box::new(output))
                }
            }
        }
    }
}
