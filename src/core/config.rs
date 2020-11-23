use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub users: Vec<UserEntry>,
    pub root_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserEntry {
    pub username: String,
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            users: Vec::new(),
            root_dir: None,
        }
    }
}
