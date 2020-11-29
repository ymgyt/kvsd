//! Define the errors that occur in the processing of kvsd.

pub(crate) mod internal;

use std::fmt;
use std::io;

/// Kvsd user facing error.
#[derive(Debug)]
pub enum KvsdError {
    /// The Key exceeds the maximum number of bytes specified in the protocol.
    MaxKeyBytes {
        /// Given key.
        key: String,
        /// Maximum bytes.
        max_bytes: usize,
    },
    /// The value exceeds the maximum number of bytes specified in the protocol.
    MaxValueBytes {
        /// Maximum bytes.
        max_bytes: usize,
    },
    /// I/O related error.
    Io(io::Error),
    /// Unauthenticated user request operations that require authentication.
    Unauthenticated,
    /// Etc error, maybe bug.
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
