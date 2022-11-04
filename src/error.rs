use std::io;

/// KvStore error type
#[derive(Debug)]
pub enum ErrorKind {
    /// I/O error
    Io(io::Error),
    /// Serde serializatioin or deserialization error
    Serde(serde_json::Error),
    /// Not read successfully
    ReadFail,
    /// Key does not exist so fail removing
    KeyNotFound,
}

impl From<io::Error> for ErrorKind {
    fn from(err: io::Error) -> ErrorKind {
        ErrorKind::Io(err)
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> ErrorKind {
        ErrorKind::Serde(err)
    }
}

/// A specialized `Result` type for KvStore operations.
pub type Result<T> = std::result::Result<T, ErrorKind>;
