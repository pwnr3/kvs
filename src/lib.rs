#![deny(missing_docs)]
//! A simple key-val db.

pub use engines::{KvStore, KvsEngine, Message, SledKvsEngine};
pub use error::{ErrorKind, Result};
pub use logger::Logger;

mod engines;
mod error;
mod logger;
