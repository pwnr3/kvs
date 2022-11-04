extern crate clap;

use clap::{Args, Parser, Subcommand};
use kvs::{ErrorKind, KvStore, Result};
use std::env::current_dir;
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
    val: String,
}

#[derive(Args)]
struct RemoveCommand {
    /// A string key.
    key: String,
}

fn main() -> Result<()> {
    let args = Arg::parse();

    match &args.command {
        Commands::Get(cmd) => {
            let key = cmd.key.clone();
            let mut store = KvStore::open(current_dir()?)?;
            if let Some(val) = store.get(key)? {
                println!("{}", val);
            } else {
                println!("Key not found");
            }
        }
        Commands::Set(cmd) => {
            let mut store = KvStore::open(current_dir()?)?;
            store.set(cmd.key.clone(), cmd.val.clone())?;
        }
        Commands::Rm(cmd) => {
            let key = cmd.key.clone();
            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key) {
                Ok(_) => {}
                Err(ErrorKind::KeyNotFound) => {
                    println!("Key not found");
                    process::exit(1);
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}
