use std::path::Path;
use std::str::FromStr;

use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli::SERVER;
use crate::common::debug;
use crate::config::Initializer;
use crate::server::tcp::Config as ServerConfig;
use crate::{KvsError, Result};

const ARG_MAX_CONN: &str = "server_max_connection";
const ARG_CONNECTION_TCP_BUFFER_BYTES: &str = "server_connection_tcp_buffer_bytes";
const ARG_AUTHENTICATE_TIMEOUT_MILLISECONDS: &str = "server_authenticate_timeout_milliseconds";
const ARG_CONFIG_PATH: &str = "server_config_path";
const ARG_HOST: &str = "server_host";
const ARG_PORT: &str = "server_port";
const MUST_ARG_KVS_DIR: &str = "kvs_dir";

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SERVER)
        .about("Running kvs server")
        .arg(
            Arg::with_name(ARG_MAX_CONN)
                .long("max-connections")
                .takes_value(true)
                .env("KVS_SERVER_MAX_CONNECTIONS")
                .help("Max tcp connections"),
        )
        .arg(
            Arg::with_name(ARG_CONNECTION_TCP_BUFFER_BYTES)
                .long("connection-tcp-buffer-bytes")
                .takes_value(true)
                .env("KVS_SERVER_CONNECTION_TCP_BUFFER_BYTES")
                .help("Buffer bytes assigned to each tcp connection"),
        )
        .arg(
            Arg::with_name(ARG_AUTHENTICATE_TIMEOUT_MILLISECONDS)
                .long("authenticate-timeout-milliseconds")
                .takes_value(true)
                .env("KVS_SERVER_AUTHENTICATE_TIMEOUT_MILLISECONDS")
                .help("Authenticate timeout."),
        )
        .arg(
            Arg::with_name(ARG_CONFIG_PATH)
                .long("config")
                .short("C")
                .default_value("./config.yaml")
                .takes_value(true)
                .env("KVS_SERVER_CONFIG_PATH")
                .help("Configuration file path"),
        )
        .arg(
            Arg::with_name(ARG_HOST)
                .long("bind-host")
                .takes_value(true)
                .env("KVS_SERVER_HOST")
                .help("Tcp binding address host(ex 0.0.0.0, localhost)"),
        )
        .arg(
            Arg::with_name(ARG_PORT)
                .long("bind-port")
                .takes_value(true)
                .env("KVS_SERVER_PORT")
                .help("Tcp binding address port"),
        )
        .arg(
            Arg::with_name(MUST_ARG_KVS_DIR)
                .long("kvs-dir")
                .takes_value(true)
                .default_value(".kvs")
                .env("KVS_DIR")
                .help("root directory where kvs store it's data"),
        )
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let config_path = m
        .value_of(ARG_CONFIG_PATH)
        .and_then(|s| std::path::PathBuf::from_str(s).ok())
        .unwrap();

    let root_dir = m
        .value_of(MUST_ARG_KVS_DIR)
        .map(Path::new)
        .and_then(|p| p.canonicalize().ok())
        .unwrap();

    let mut initializer = Initializer::load_config_file(config_path).await?;

    initializer
        .config
        .server
        .override_merge(&mut read_server_config(m));

    initializer.config.kvs.root_dir = Some(root_dir);

    debug!("{:?}", initializer);

    initializer.init_dir().await?;

    initializer.run_kvs().await.map_err(KvsError::from)
}

fn read_server_config(m: &ArgMatches<'_>) -> ServerConfig {
    let mut config = ServerConfig::default();

    config.set_max_tcp_connections(m.value_of(ARG_MAX_CONN).and_then(|s| s.parse().ok()));
    config.set_connection_tcp_buffer_bytes(
        m.value_of(ARG_CONNECTION_TCP_BUFFER_BYTES)
            .and_then(|s| s.parse().ok()),
    );
    config.set_authenticate_timeout_milliseconds(
        m.value_of(ARG_AUTHENTICATE_TIMEOUT_MILLISECONDS)
            .and_then(|s| s.parse().ok()),
    );
    config.set_listen_host(&mut m.value_of(ARG_HOST).map(String::from));
    config.set_listen_port(&mut m.value_of(ARG_PORT).map(String::from));

    config
}
