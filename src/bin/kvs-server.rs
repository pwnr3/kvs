use clap::{Arg, Command};
use kvs::{ErrorKind, KvStore, KvsEngine, Logger, Message, Result, SledKvsEngine};
use log;
use std::env::current_dir;
use std::io::{BufRead, Read, Write};
use std::net::TcpListener;

fn main() -> Result<()> {
    Logger::init().map_err(|e| ErrorKind::Other(format!("{:?}", e)))?;
    //if let Err(_) = Logger::init() {
    //    return Err(ErrorKind::Other("Fail to load logger".into()));
    //}

    let matches = Command::new("kvs-server")
        .author("unknown")
        .version("0.1.0")
        .about("key-value store server")
        .max_term_width(100)
        .disable_help_flag(true)
        //.disable_version_flag(true)
        //.arg_required_else_help(true)
        .allow_external_subcommands(true)
        .arg(
            Arg::new("addr")
                .long("addr")
                .value_name("IP:PORT")
                .help("ip address and port number, with the format `IP:PORT`")
                .exclusive(false)
                .default_value("127.0.0.1:4000")
                .num_args(1),
        )
        .arg(
            Arg::new("engine")
                .long("engine")
                .value_name("ENGINE-NAME")
                .help("backend engine (kvs or sled)")
                .default_value("kvs"),
        )
        .get_matches();

    let engine: &String = matches.get_one::<String>("engine").unwrap();
    if engine != "kvs" && engine != "sled" {
        eprintln!("wrong engine option, please use `kvs` or `sled`");
        std::process::exit(1);
    }

    if std::path::Path::new("engine.log").exists() {
        let f = std::fs::File::open("engine.log")?;
        let mut reader = std::io::BufReader::new(f);
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        if &buf != engine {
            eprintln!("Engine option doesn't match exist one");
            std::process::exit(1);
        }
    } else {
        let f = std::fs::File::create("engine.log")?;
        let mut writer = std::io::BufWriter::new(f);
        writer.write_all(engine.as_bytes())?;
        writer.flush()?;
    }

    //let addr = matches.get_one::<String>("addr").ok_or("127.0.0.1:4000")?;
    let addr = matches.get_one::<String>("addr").unwrap();
    let listener = TcpListener::bind(addr)?;
    log::info!("start kvs-server 0.1.0 at {}", addr);

    // `impl trait` as argument type or return type
    if engine == "kvs" {
        run(KvStore::open(current_dir()?)?, listener)?;
    } else {
        run(SledKvsEngine::open(current_dir()?)?, listener)?;
    }
    Ok(())

    // use `trait objects` with a runtime cost
    //let mut store: Box<dyn KvsEngine> = if engine == "kvs" {
    //    Box::new(KvStore::open(current_dir()?)?)
    //} else {
    //    Box::new(SledKvsEngine::new())
    //};

    //loop {
    //    let (mut socket, addr) = listener.accept()?;
    //    log::info!("Connection from {}", addr);

    //    let mut msg = vec![0; 128];
    //    let len = socket.read(&mut msg)?;
    //    let msg: Message = serde_json::from_slice(&msg[..len])?;
    //    log::info!("{:?}", msg);

    //    match msg {
    //        Message::Get { key } => {
    //            if let Some(val) = store.get(key)? {
    //                socket.write(val.as_bytes())?;
    //            } else {
    //                socket.write(b"Key not found")?;
    //            }
    //        }
    //        Message::Set { key, val } => {
    //            store.set(key, val)?;
    //        }
    //        Message::Rm { key } => {
    //            match store.remove(key) {
    //                Ok(_) => {}
    //                Err(ErrorKind::KeyNotFound) => {
    //                    socket.write(b"Key not found")?;
    //                }
    //                Err(e) => return Err(e),
    //            }
    //        }
    //    }
    //}
}

// or `store: impl KvsEngine`
fn run<T: KvsEngine>(mut store: T, listener: TcpListener) -> Result<()> {
    loop {
        let (mut socket, addr) = listener.accept()?;
        log::info!("Connection from {}", addr);

        let mut msg = vec![0; 128];
        let len = socket.read(&mut msg)?;
        let msg: Message = serde_json::from_slice(&msg[..len])?;
        log::info!("{:?}", msg);

        match msg {
            Message::Get { key } => {
                if let Some(val) = store.get(key)? {
                    socket.write(val.as_bytes())?;
                } else {
                    socket.write(b"Key not found")?;
                }
            }
            Message::Set { key, val } => {
                store.set(key, val)?;
            }
            Message::Rm { key } => match store.remove(key) {
                Ok(_) => {}
                Err(ErrorKind::KeyNotFound) => {
                    socket.write(b"Key not found")?;
                }
                Err(e) => return Err(e),
            },
        }
    }
}
