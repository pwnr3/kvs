use crate::Result;
use serde::{Deserialize, Serialize};

/// Trait for different engine.
pub trait KvsEngine: Send + 'static {
    /// Set the value of a string key to a string.
    ///
    /// # Errors
    ///
    /// Return an error if the value is not written successfully.
    fn set(&self, key: String, value: String) -> Result<()>;
    /// Get the string value of a string key.
    ///
    /// If the key does not exist, return None.
    ///
    /// # Errors
    ///
    /// Return an error if the value is not read successfully.
    fn get(&self, key: String) -> Result<Option<String>>;
    /// Remove a given key.
    ///
    /// # Errors
    ///
    /// Return an error if the key does not exist or is not removed successfully.
    fn remove(&self, key: String) -> Result<()>;
}

/// Message from client to server.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// get
    Get {
        /// key
        key: String,
    },
    /// set
    Set {
        /// key
        key: String,
        /// val
        val: String,
    },
    /// rm
    Rm {
        /// key
        key: String,
    },
}

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;

mod kvs;
mod sled;
