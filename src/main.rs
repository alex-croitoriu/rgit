mod args;
mod commands;
mod index;
mod object_store;
mod utils;

use args::{Cli, Commands};
use clap::Parser;

use commands::*;

use crate::args::BranchSubcommands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => match init() {
            Ok(path) => println!("Initialized empty repository in '{path}'"),
            Err(e) => println!("{e}"),
        },
        Commands::Status => match status() {
            Ok(status) => println!("{status}"),
            Err(e) => println!("{e}"),
        },
        Commands::Add { paths } => {
            for path in paths {
                match add(&path) {
                    Ok(()) => println!("Added '{path}' to the index"),
                    Err(e) => println!("{e}"),
                }
            }
        }
        Commands::Commit { message } => match commit(&message) {
            Ok(hash) => println!("Commit: {hash}"),
            Err(e) => println!("{e}"),
        },
        // Commands::Diff { obj1, obj2 } => match diff() {},
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
        // Commands::Merge { branch }=> match merge() {},
        _ => (),
    }
}
