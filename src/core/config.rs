use std::path::PathBuf;

use serde::Deserialize;

/// kvsd configuration.
#[derive(Default, Debug, Deserialize)]
pub struct Config {
    /// authenticated principal users.
    pub users: Vec<UserEntry>,
    /// root directory to store kvsd data and state.
    pub root_dir: Option<PathBuf>,
}

/// Authenticated users.
#[derive(Debug, Deserialize, Clone)]
pub struct UserEntry {
    /// username.
    pub username: String,
    /// password.
    pub password: String,
}
