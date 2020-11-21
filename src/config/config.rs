use serde::Deserialize;

use crate::core;
use crate::server::tcp;

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) server: tcp::Config,
    pub(crate) kvs: core::Config,
}

