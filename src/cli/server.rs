use std::path::Path;
use std::str::FromStr;

use clap::{Arg, ArgMatches, Command};

use crate::cli::{root::MUST_ARG_DISABLE_TLS, SERVER};
use crate::common::debug;
use crate::config::Initializer;
use crate::server::tcp::Config as ServerConfig;
use crate::{KvsdError, Result};

const ARG_MAX_CONN: &str = "server_max_connection";
const ARG_CONNECTION_TCP_BUFFER_BYTES: &str = "server_connection_tcp_buffer_bytes";
const ARG_AUTHENTICATE_TIMEOUT_MILLISECONDS: &str = "server_authenticate_timeout_milliseconds";
const ARG_CONFIG_PATH: &str = "server_config_path";
const ARG_HOST: &str = "server_host";
const ARG_PORT: &str = "server_port";
const MUST_ARG_KVSD_DIR: &str = "kvsd_dir";
const MUST_ARG_TLS_CERT: &str = "tls_cert";
const MUST_ARG_TLS_KEY: &str = "tls_key";

pub(super) fn subcommand() -> Command {
    Command::new(SERVER)
        .about("Running kvsd server")
        .arg(
            Arg::new(ARG_MAX_CONN)
                .long("max-connections")
                .env("KVSD_SERVER_MAX_CONNECTIONS")
                .help("Max tcp connections"),
        )
        .arg(
            Arg::new(ARG_CONNECTION_TCP_BUFFER_BYTES)
                .long("connection-tcp-buffer-bytes")
                .env("KVSD_SERVER_CONNECTION_TCP_BUFFER_BYTES")
                .help("Buffer bytes assigned to each tcp connection"),
        )
        .arg(
            Arg::new(ARG_AUTHENTICATE_TIMEOUT_MILLISECONDS)
                .long("authenticate-timeout-milliseconds")
                .env("KVSD_SERVER_AUTHENTICATE_TIMEOUT_MILLISECONDS")
                .help("Authenticate timeout."),
        )
        .arg(
            Arg::new(ARG_CONFIG_PATH)
                .long("config")
                .short('C')
                .default_value("./files/config.yaml")
                .env("KVSD_SERVER_CONFIG_PATH")
                .help("Configuration file path"),
        )
        .arg(
            Arg::new(ARG_HOST)
                .long("bind-host")
                .env("KVSD_SERVER_HOST")
                .help("Tcp binding address host(ex 0.0.0.0, localhost)"),
        )
        .arg(
            Arg::new(ARG_PORT)
                .long("bind-port")
                .env("KVSD_SERVER_PORT")
                .help("Tcp binding address port"),
        )
        .arg(
            Arg::new(MUST_ARG_KVSD_DIR)
                .long("kvsd-dir")
                .default_value(".kvsd")
                .env("KVSD_DIR")
                .help("root directory where kvsd store it's data"),
        )
        .arg(
            Arg::new(MUST_ARG_TLS_CERT)
                .long("cert")
                .default_value("./files/localhost.pem")
                .env("KVSD_TLS_CERT")
                .help("tls server certificate file path"),
        )
        .arg(
            Arg::new(MUST_ARG_TLS_KEY)
                .long("key")
                .default_value("./files/localhost.key")
                .env("KVSD_TLS_KEY")
                .help("tls server private key file path"),
        )
}

/// Launch the server command.
pub async fn run(m: &ArgMatches) -> Result<()> {
    let config_path = m
        .get_one::<String>(ARG_CONFIG_PATH)
        .and_then(|s| std::path::PathBuf::from_str(s).ok())
        .unwrap();

    // Canonicalize require path already exist.
    let root_dir = m
        .get_one::<String>(MUST_ARG_KVSD_DIR)
        .map(Path::new)
        .unwrap();
    tokio::fs::create_dir_all(root_dir).await?;
    let root_dir = root_dir.canonicalize().unwrap();

    let mut initializer = Initializer::load_config_file(config_path).await?;

    initializer
        .config
        .server
        .override_merge(&mut read_server_config(m));

    initializer.set_root_dir(root_dir);

    debug!("{:?}", initializer);

    initializer.init_dir().await?;

    initializer
        .run_kvsd(tokio::signal::ctrl_c())
        .await
        .map_err(KvsdError::from)
}

fn read_server_config(m: &ArgMatches) -> ServerConfig {
    let mut config = ServerConfig::default();

    config.set_max_tcp_connections(m.get_one::<u32>(ARG_MAX_CONN).cloned());
    config.set_connection_tcp_buffer_bytes(
        m.get_one::<usize>(ARG_CONNECTION_TCP_BUFFER_BYTES).cloned(),
    );
    config.set_authenticate_timeout_milliseconds(
        m.get_one::<u64>(ARG_AUTHENTICATE_TIMEOUT_MILLISECONDS)
            .cloned(),
    );
    config.set_listen_host(&mut m.get_one::<String>(ARG_HOST).map(String::from));
    config.set_listen_port(&mut m.get_one::<String>(ARG_PORT).map(String::from));
    config.set_disable_tls(&mut Some(m.contains_id(MUST_ARG_DISABLE_TLS)));
    config.set_tls_certificate(&mut m.get_one::<String>(MUST_ARG_TLS_CERT).map(String::from));
    config.set_tls_key(&mut m.get_one::<String>(MUST_ARG_TLS_KEY).map(String::from));

    config
}
