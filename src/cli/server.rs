use std::path::PathBuf;

use clap::Args;

use crate::common::debug;
use crate::config::Initializer;
use crate::server::tcp::Config as ServerConfig;
use crate::{KvsdError, Result};

/// Running kvsd server
#[derive(Args, Debug)]
pub struct ServerCommand {
    /// Max tcp connections
    #[arg(long, env = "KVSD_SERVER_MAX_CONNECTIONS")]
    max_connections: Option<u32>,
    /// Buffer bytes assigned to each tcp connection
    #[arg(long, env = "KVSD_SERVER_CONNECTION_TCP_BUFFER_BYTES")]
    connection_tcp_buffer_bytes: Option<usize>,
    /// Authenticate timeout
    #[arg(long, env = "KVSD_SERVER_AUTHENTICATE_TIMEOUT_MILLISECONDS")]
    authenticate_timeout_milliseconds: Option<u64>,
    /// Configuration file path
    #[arg(
        long,
        short = 'C',
        default_value = "./files/config.yaml",
        env = "KVSD_SERVER_CONFIG_PATH"
    )]
    config: PathBuf,
    /// Tcp binding address host(e.g. 0.0.0.0, localhost)
    #[arg(long, env = "KVSD_SERVER_HOST")]
    bind_host: Option<String>,
    /// Tcp binding address port
    #[arg(long, env = "KVSD_SERVER_PORT")]
    bind_port: Option<String>,
    /// Root directory where kvsd store it's data
    #[arg(long, env = "KVSD_DIR", default_value = ".kvsd")]
    kvsd_dir: PathBuf,
    /// Tls server certificate file path
    #[arg(long, env = "KVSD_TLS_CERT", default_value = "./files/localhost.pem")]
    cert: PathBuf,
    /// Tls server private key file path
    #[arg(long, env = "KVSD_TLS_KEY", default_value = "./files/localhost.key")]
    key: PathBuf,
}

impl ServerCommand {
    pub async fn run(self, disable_tls: bool) -> Result<()> {
        let ServerCommand {
            max_connections,
            connection_tcp_buffer_bytes,
            authenticate_timeout_milliseconds,
            config,
            mut bind_host,
            mut bind_port,
            kvsd_dir,
            cert,
            key,
        } = self;

        tokio::fs::create_dir_all(&kvsd_dir).await?;
        let root_dir = kvsd_dir.canonicalize().unwrap();

        let mut initializer = Initializer::load_config_file(config).await?;

        let mut config = {
            let mut config = ServerConfig::default();

            config.set_max_tcp_connections(max_connections);
            config.set_connection_tcp_buffer_bytes(connection_tcp_buffer_bytes);
            config.set_authenticate_timeout_milliseconds(authenticate_timeout_milliseconds);
            config.set_listen_host(&mut bind_host);
            config.set_listen_port(&mut bind_port);
            config.set_disable_tls(&mut Some(disable_tls));
            config.set_tls_certificate(&mut Some(cert));
            config.set_tls_key(&mut Some(key));
            config
        };

        initializer.config.server.override_merge(&mut config);

        initializer.set_root_dir(root_dir);

        debug!("{:?}", initializer);

        initializer.init_dir().await?;

        initializer
            .run_kvsd(tokio::signal::ctrl_c())
            .await
            .map_err(KvsdError::from)
    }
}
