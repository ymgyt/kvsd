use serde::Deserialize;

use crate::core;
use crate::server::tcp;

/// kvsd configuration.
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    /// server configuration.
    pub server: tcp::Config,
    /// kvsd configuration.
    pub kvsd: core::Config,
}
