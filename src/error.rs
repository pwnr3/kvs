use sled;
use std::io;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

/// KvStore error type
#[derive(Debug)]
pub enum ErrorKind {
    /// I/O error
    Io(std::io::Error),
    /// Serde serializatioin or deserialization error
    Serde(serde_json::Error),
    /// Seld error
    Sled(sled::Error),
    /// Utf8 for &str
    Str(Utf8Error),
    /// FromUtf8 for String
    String(FromUtf8Error),
    /// Not read successfully
    ReadFail,
    /// Key does not exist so fail removing
    KeyNotFound,
    /// Other
    Other(String),
}

impl From<sled::Error> for ErrorKind {
    fn from(err: sled::Error) -> ErrorKind {
        ErrorKind::Sled(err)
    }
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

impl From<Utf8Error> for ErrorKind {
    fn from(err: Utf8Error) -> ErrorKind {
        ErrorKind::Str(err)
    }
}

impl From<FromUtf8Error> for ErrorKind {
    fn from(err: FromUtf8Error) -> ErrorKind {
        ErrorKind::String(err)
    }
}

/// A specialized `Result` type for KvStore operations.
pub type Result<T> = std::result::Result<T, ErrorKind>;
