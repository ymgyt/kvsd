pub(crate) mod internal;

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum KvsError {
    // The Key exceeds the maximum number of bytes specified in the protocol.
    MaxKeyBytes { key: String, max_bytes: usize },
    MaxValueBytes { max_bytes: usize },
    Io(io::Error),
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KvsError::MaxKeyBytes { max_bytes, .. } => {
                write!(f, "key exceeds maximum bytes({})", max_bytes)
            }
            KvsError::MaxValueBytes { max_bytes, .. } => {
                write!(f, "value exceeds maximum bytes({})", max_bytes)
            }
            KvsError::Io(err) => err.fmt(f),
            KvsError::Internal(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> Self {
        KvsError::Io(err)
    }
}

impl From<self::internal::Error> for KvsError {
    fn from(err: self::internal::Error) -> Self {
        KvsError::Internal(Box::new(err))
    }
}

impl From<&str> for KvsError {
    fn from(s: &str) -> Self {
        KvsError::from(s.to_owned())
    }
}

impl From<String> for KvsError {
    fn from(s: String) -> Self {
        KvsError::Internal(Box::<dyn std::error::Error + Send + Sync>::from(s))
    }
}

impl std::error::Error for KvsError {}
