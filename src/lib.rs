#![allow(clippy::module_inception)]
#![deny(missing_docs)]

//! Kvsd is an asynchronous key value store with tokio runtime.
//! The key value is saved by appending it to a file and keeps the offset in memory.

mod server;

pub mod cli;
pub mod client;
pub mod config;
pub mod core;
pub mod error;
pub mod protocol;

pub use crate::error::KvsdError;
/// A specialized Result type for kvsd operations.
pub type Result<T, E = crate::error::KvsdError> = std::result::Result<T, E>;

pub use protocol::{Key, Value};

pub(crate) mod common {
    pub(crate) type Result<T, E = crate::error::internal::Error> = std::result::Result<T, E>;

    pub(crate) type Error = crate::error::internal::Error;
    pub(crate) type ErrorKind = crate::error::internal::ErrorKind;

    pub use crate::error::KvsdError;

    pub(crate) type Time = chrono::DateTime<chrono::Utc>;

    pub use tracing::{debug, error, info, trace, warn};
}
