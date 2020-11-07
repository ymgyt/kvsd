#![allow(dead_code)]
#![allow(clippy::module_inception)]

mod client;
mod config;
mod core;
mod server;

pub mod cli;
pub mod error;
pub mod protocol;

pub use crate::error::KvsError;
pub type Result<T, E = crate::error::KvsError> = std::result::Result<T, E>;

pub(crate) mod common {
    pub(crate) type Result<T, E = crate::error::internal::Error> = std::result::Result<T, E>;

    pub(crate) type Error = crate::error::internal::Error;
    pub(crate) type ErrorKind = crate::error::internal::ErrorKind;

    pub use crate::error::KvsError;

    pub(crate) type Time = chrono::DateTime<chrono::Utc>;

    pub use tracing::{debug, error, info, trace, warn};
}
