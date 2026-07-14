mod args;
mod commands;
mod state;
mod utils;

use args::{BranchSubcommands, Cli, Commands};
use clap::Parser;

use crate::commands::*;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match init() {
            Ok(path) => println!("Initialized empty repository in '{}'", path.display()),
            Err(e) => println!("{e}"),
        },
        Commands::Status => match status() {
            Ok(status) => println!("{status}"),
            Err(e) => println!("{e}"),
        },
        Commands::Add { paths } => match add(paths) {
            Ok(()) => println!("Added paths to the index"),
            Err(e) => println!("{e}"),
        },
        Commands::Commit { message } => match commit(&message) {
            Ok(hash) => println!("Commit: {hash}"),
            Err(e) => println!("{e}"),
        },
        Commands::Diff { target } => match diff(target) {
            Ok(diff) => println!("{diff}"),
            Err(e) => println!("{e}"),
        },
        Commands::Branch(BranchSubcommands::List) => match list() {
            Ok(branches) => {
                for branch in branches {
                    println!("{branch}");
                }
            }
            Err(e) => println!("{e}"),
        },
        Commands::Branch(BranchSubcommands::Create { name }) => match create(&name) {
            Ok(()) => println!("Branch created: {name}"),
            Err(e) => println!("{e}"),
        },
        Commands::Branch(BranchSubcommands::Delete { name }) => match delete(&name) {
            Ok(()) => println!("Branch deleted: {name}"),
            Err(e) => println!("{e}"),
        },
        Commands::Checkout { target } => match checkout(&target) {
            Ok(()) => println!("Switched to branch: {target}"),
            Err(e) => println!("{e}"),
        },
        Commands::Merge { target } => match merge(&target) {
            Ok(hash) => println!("Merge successful: {hash}"),
            Err(e) => println!("{e}"),
        },
    }
}
