mod args;
mod commands;

use args::{Cli, Commands};
use clap::Parser;

use crate::commands::init::init;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            match init() {
                Ok(path) => println!("Initialized repository in {} ", path.display()),
                Err(e) => println!("{e}")
            }
        }
        Commands::Add { paths } => {
            for path in paths {
                println!("{path}");
            }
        }
        _ => println!("Other command"),
    }
}
