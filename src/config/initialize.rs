use std::path::Path;

use tokio::fs;

use crate::common::Result;
use crate::config::Config;
use crate::core;
use crate::server::tcp::Server;

#[derive(Debug)]
pub(crate) struct Initializer {
    pub(crate) config: Config,
}

impl Initializer {
    pub(crate) async fn load_config_file(path: impl AsRef<Path>) -> Result<Self> {
        let f = fs::File::open(path).await?;
        let config = serde_yaml::from_reader::<_, Config>(f.into_std().await)?;

        Ok(Self { config })
    }

    pub(crate) async fn run_kvs(self) -> Result<()> {
        let builder = core::Builder::from_config(self.config.kvs);
        let kvs = builder.build()?;
        let request_sender = kvs.request_channel();

        tokio::spawn(kvs.run());

        let server = Server::new(self.config.server);
        server.run(request_sender).await
    }
}
