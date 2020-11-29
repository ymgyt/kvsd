//! Define the features to support as a cli.

mod root;
pub(crate) use root::authenticate;
pub use root::new;

mod delete;
pub use delete::run as delete;

mod get;
pub use get::run as get;

mod ping;
pub use ping::run as ping;

mod server;
pub use server::run as server;

mod set;
pub use set::run as set;

/// ping subcommand name.
pub const PING: &str = "ping";
/// server subcommand name.
pub const SERVER: &str = "server";
/// set subcommand name.
pub const SET: &str = "set";
/// get subcommand name.
pub const GET: &str = "get";
/// delete subcommand name.
pub const DELETE: &str = "delete";
