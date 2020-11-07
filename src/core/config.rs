use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
