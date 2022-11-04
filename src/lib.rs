#![deny(missing_docs)]
//! A simple key-val db.

pub use error::{ErrorKind, Result};
pub use kv::KvStore;

mod error;
mod kv;
