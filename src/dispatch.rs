use anyhow::Result;

use crate::{
    args::{BranchSubcommands, Commands},
    commands::*,
    state::Repository,
};

pub fn dispatch(command: Commands) -> Result<String> {
    match command {
        Commands::Init => Ok(init::Command::execute(())?.to_string()),
        command => {
            let repo = Repository::load()?;

            match command {
                Commands::Init => unreachable!(),
                Commands::Status => Ok(status::Command::execute(&repo, ())?.to_string()),
                Commands::Add(args) => Ok(add::Command::execute(&repo, args)?.to_string()),
                Commands::Rm(args) => Ok(rm::Command::execute(&repo, args)?.to_string()),
                Commands::Commit(args) => Ok(commit::Command::execute(&repo, args)?.to_string()),
                Commands::Log => Ok(String::from("not implemented")),
                Commands::Diff(args) => Ok(diff::Command::execute(&repo, args)?.to_string()),
                Commands::Switch(args) => Ok(switch::Command::execute(&repo, args)?.to_string()),
                Commands::Branch(BranchSubcommands::List) => {
                    Ok(branch::list::Command::execute(&repo, ())?.to_string())
                }
                Commands::Branch(BranchSubcommands::Create(args)) => {
                    Ok(branch::create::Command::execute(&repo, args)?.to_string())
                }
                Commands::Branch(BranchSubcommands::Delete(args)) => {
                    Ok(branch::delete::Command::execute(&repo, args)?.to_string())
                }
                Commands::Merge(args) => Ok(merge::Command::execute(&repo, args)?.to_string()),
            }
        }
    }
}
