//! Define the features to support as a cli.

mod root;
pub use root::{authenticate, parse, Command, KvsdCommand};

mod delete;
mod get;
mod ping;
mod server;
mod set;
