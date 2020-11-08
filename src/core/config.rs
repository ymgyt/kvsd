use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) users: Vec<UserEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UserEntry {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { users: Vec::new() }
    }
}
