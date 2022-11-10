use clap::{Args, Parser, Subcommand};
use kvs::{Message, Result};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    disable_help_subcommand = true,
    disable_help_flag = true,
    subcommand_required = true,
)]
struct Arg {
    //#[arg(
    //    long = "addr",
    //    value_name = "IP:PORT",
    //    default_value = "127.0.0.1:4000",
    //)]
    //addr: String,
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

    #[arg(
        long = "addr",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: String,
}

#[derive(Args)]
struct SetCommand {
    /// A string key.
    key: String,
    /// The string value of the key.
    val: String,

    #[arg(
        long = "addr",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: String,
}

#[derive(Args)]
struct RemoveCommand {
    /// A string key.
    key: String,

    #[arg(
        long = "addr",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: String,
}

fn main() -> Result<()> {
    let args = Arg::parse();

    match args.command {
        Commands::Get(cmd) => {
            let mut stream = TcpStream::connect(cmd.addr)?;
            stream.write(&serde_json::to_vec(&Message::Get { key: cmd.key })?)?;
            let mut msg = vec![0; 128];
            let len = stream.read(&mut msg)?;
            println!("{}", std::str::from_utf8(&msg[..len])?);
        }
        Commands::Set(cmd) => {
            let mut stream = TcpStream::connect(cmd.addr)?;
            stream.write(&serde_json::to_vec(&Message::Set {
                key: cmd.key,
                val: cmd.val,
            })?)?;
        }
        Commands::Rm(cmd) => {
            let mut stream = TcpStream::connect(cmd.addr)?;
            stream.write(&serde_json::to_vec(&Message::Rm { key: cmd.key })?)?;
            let mut msg = vec![0; 128];
            let len = stream.read(&mut msg)?;
            if len > 0 {
                eprintln!("{}", std::str::from_utf8(&msg[..len])?);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
