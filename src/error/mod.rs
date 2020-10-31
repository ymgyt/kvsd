pub(crate) mod internal;

use std::fmt;

#[derive(Debug)]
pub enum KvsError {
    // The Key exceeds the maximum number of bytes specified in the protocol.
    MaxKeyBytes { key: String, max_bytes: usize },
    MaxValueBytes { max_bytes: usize },
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
        }
    }
}

impl std::error::Error for KvsError {}
