mod args;
mod commands;
mod dispatch;
mod state;
mod utils;

use crate::args::Cli;
use crate::dispatch::dispatch;

use clap::Parser;

fn main() {
    let cli = Cli::parse();

    match dispatch(cli.command) {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
