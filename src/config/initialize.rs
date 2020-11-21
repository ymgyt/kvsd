use std::path::Path;

use tokio::fs;

use crate::common::{info,Result};
use crate::config::{filepath,Config};
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
        let kvs = builder.build().await?;
        let request_sender = kvs.request_channel();

        tokio::spawn(kvs.run());

        let server = Server::new(self.config.server);
        server.run(request_sender).await
    }

    pub(crate) async fn init_dir(&mut self) -> Result<()> {
        let root_dir = self.config.kvs.root_dir.clone().unwrap();

        info!(path=%root_dir.display(), "Initialize kvs root directory");

        // Create root kvs directory.
        tokio::fs::create_dir_all(root_dir.as_path()).await?;

        // Namespaces.
        let namespaces = root_dir.join(filepath::NAMESPACES);
        tokio::fs::create_dir_all(namespaces.as_path()).await?;

        let initial_namespaces = vec![
            namespaces.join(filepath::NS_SYSTEM),
            namespaces.join("default/default")
        ];

        for ns in &initial_namespaces {
            tokio::fs::create_dir_all(ns).await?;
        }

        // Make sure default table exists at initialization.
        tokio::fs::File::create(namespaces.join("default/default/default.kvs")).await?;

        Ok(())
    }
}
