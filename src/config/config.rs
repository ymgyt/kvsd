use serde::Deserialize;

use crate::core;
use crate::server::tcp;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub server: tcp::Config,
    pub kvsd: core::Config,
}
