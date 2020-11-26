pub(crate) mod internal;

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum KvsdError {
    // The Key exceeds the maximum number of bytes specified in the protocol.
    MaxKeyBytes { key: String, max_bytes: usize },
    MaxValueBytes { max_bytes: usize },
    Io(io::Error),
    Unauthenticated,
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for KvsdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KvsdError::MaxKeyBytes { max_bytes, .. } => {
                write!(f, "key exceeds maximum bytes({})", max_bytes)
            }
            KvsdError::MaxValueBytes { max_bytes, .. } => {
                write!(f, "value exceeds maximum bytes({})", max_bytes)
            }
            KvsdError::Io(err) => err.fmt(f),
            KvsdError::Unauthenticated => write!(f, "unauthenticated"),
            KvsdError::Internal(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for KvsdError {
    fn from(err: io::Error) -> Self {
        KvsdError::Io(err)
    }
}

impl From<self::internal::Error> for KvsdError {
    fn from(err: self::internal::Error) -> Self {
        KvsdError::Internal(Box::new(err))
    }
}

impl From<&str> for KvsdError {
    fn from(s: &str) -> Self {
        KvsdError::from(s.to_owned())
    }
}

impl From<String> for KvsdError {
    fn from(s: String) -> Self {
        KvsdError::Internal(Box::<dyn std::error::Error + Send + Sync>::from(s))
    }
}

impl std::error::Error for KvsdError {}
