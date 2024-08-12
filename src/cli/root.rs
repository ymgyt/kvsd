use clap::{ArgAction, Args, Parser, Subcommand};

use crate::cli::{delete, get, ping, server, set};
use crate::client::tcp::UnauthenticatedClient;
use crate::client::Api;
use crate::server::DEFAULT_PORT;

/// Kvsd command
#[derive(Parser, Debug)]
#[command(version, propagate_version = true, subcommand_required = true)]
pub struct KvsdCommand {
    /// Client options
    #[command(flatten)]
    pub client: ClientOptions,
    /// Subcommand
    #[command(subcommand)]
    pub command: Command,
}

/// Client options
#[derive(Args, Debug)]
pub struct ClientOptions {
    /// Remote kvsd server host
    #[arg(long, env = "KVSD_HOST", default_value = "127.0.0.1", global = true)]
    pub host: String,
    /// Server listening port
    #[arg(long, default_value = DEFAULT_PORT, global = true)]
    pub port: u16,
    /// Username
    #[arg(long, env = "KVSD_USERNAME", default_value = "kvsduser", global = true)]
    pub username: String,
    /// Password
    #[arg(long, env = "KVSD_PASSWORD", default_value = "secret", global = true)]
    pub password: String,
    /// Disable tls connections
    #[arg(long,env = "KVSD_DISABLE_TLS", action = ArgAction::SetTrue, global = true)]
    pub disable_tls: bool,
}

/// Subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Ping
    Ping(ping::PingCommand),
    /// Delete
    Delete(delete::DeleteCommand),
    /// Get
    Get(get::GetCommand),
    /// Set
    Set(set::SetCommand),
    /// Server
    Server(server::ServerCommand),
}

/// Parse command line args
pub fn parse() -> KvsdCommand {
    KvsdCommand::parse()
}

/// Authenticate client
pub async fn authenticate(options: ClientOptions) -> crate::Result<Box<dyn Api>> {
    let ClientOptions {
        host,
        port,
        username,
        password,
        disable_tls,
    } = options;

    let client: Box<dyn Api> = if disable_tls {
        UnauthenticatedClient::insecure_from_addr(host, port)
            .await?
            .authenticate(username, password)
            .await
            .map(Box::new)?
    } else {
        UnauthenticatedClient::from_addr(host, port)
            .await?
            .authenticate(username, password)
            .await
            .map(Box::new)?
    };
    Ok(client)
}
