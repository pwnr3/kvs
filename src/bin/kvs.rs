extern crate clap;

use clap::{Args, Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    disable_help_subcommand = true,
    disable_help_flag = true,
)]
struct Arg {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the string value of a given string key
    Get(GetCommand),
    /// Set the value of a string key to a string
    Set(SetCommand),
    /// Remove a given key
    Rm(RemoveCommand),
}

#[derive(Args)]
struct GetCommand {
    /// A string key.
    key: String,
}

#[derive(Args)]
struct SetCommand {
    /// A string key.
    key: String,
    /// The string value of the key.
    value: String,
}

#[derive(Args)]
struct RemoveCommand {
    /// A string key.
    key: String,
}

fn main() {
    let args = Arg::parse();

    match &args.command {
        Commands::Get(_) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        Commands::Set(_) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
        Commands::Rm(_) => {
            eprintln!("unimplemented");
            process::exit(1);
        }
    }
}
