pub(crate) mod internal;

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum KvsError {
    // The Key exceeds the maximum number of bytes specified in the protocol.
    MaxKeyBytes { key: String, max_bytes: usize },
    MaxValueBytes { max_bytes: usize },
    Io(io::Error),
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
        }
    }
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> Self {
        KvsError::Io(err)
    }
}

impl std::error::Error for KvsError {}
