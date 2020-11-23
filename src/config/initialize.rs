use std::future::Future;
use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::net::TcpListener;

use crate::common::{info, Result};
use crate::config::{filepath, Config};
use crate::core;
use crate::server::tcp::Server;
use crate::KvsError;

#[derive(Debug)]
pub struct Initializer {
    pub(crate) config: Config,
    listener: Option<TcpListener>,
}

impl Initializer {
    // Construct Initializer from config.
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            listener: None,
        }
    }

    pub fn set_root_dir(&mut self, root_dir: impl Into<PathBuf>) {
        self.config.kvs.root_dir = Some(root_dir.into());
    }

    pub fn set_listener(&mut self, listener: TcpListener) {
        self.listener = Some(listener);
    }

    pub(crate) async fn load_config_file(path: impl AsRef<Path>) -> Result<Self> {
        let f = fs::File::open(path).await?;
        let config = serde_yaml::from_reader::<_, Config>(f.into_std().await)?;

        Ok(Initializer::from_config(config))
    }

    pub async fn run_kvs(self, shutdown: impl Future) -> Result<(), KvsError> {
        let builder = core::Builder::from_config(self.config.kvs);
        let kvs = builder.build().await?;
        let request_sender = kvs.request_channel();

        tokio::spawn(kvs.run());

        let listener = match self.listener {
            Some(listener) => listener,
            None => {
                let addr = self.config.server.listen_addr();
                info!(%addr, "Listening");
                TcpListener::bind(addr).await?
            }
        };

        let server = Server::new(self.config.server);

        server.run(request_sender, listener, shutdown).await?;

        Ok(())
    }

    pub async fn init_dir(&mut self) -> Result<(), KvsError> {
        let root_dir = self.config.kvs.root_dir.clone().unwrap();

        info!(path=%root_dir.display(), "Initialize kvs root directory");

        // Create root kvs directory.
        tokio::fs::create_dir_all(root_dir.as_path()).await?;

        // Namespaces.
        let namespaces = root_dir.join(filepath::NAMESPACES);
        tokio::fs::create_dir_all(namespaces.as_path()).await?;

        let initial_namespaces = vec![
            namespaces.join(filepath::NS_SYSTEM),
            namespaces.join("default/default"),
        ];

        for ns in &initial_namespaces {
            tokio::fs::create_dir_all(ns).await?;
        }

        Ok(())
    }
}
