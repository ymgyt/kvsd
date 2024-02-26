use clap::{Arg, ArgMatches, Command};

use crate::cli;
use crate::client::tcp::UnauthenticatedClient;
use crate::client::Api;
use crate::server::DEFAULT_PORT;

pub(super) const MUST_ARG_HOST: &str = "host";
pub(super) const MUST_ARG_PORT: &str = "port";
pub(super) const MUST_ARG_USERNAME: &str = "username";
pub(super) const MUST_ARG_PASSWORD: &str = "password";
pub(super) const MUST_ARG_DISABLE_TLS: &str = "disable-tls";

/// new return top level clap Command.
pub fn new() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new(MUST_ARG_HOST)
                .long("host")
                .env("KVSD_HOST")
                .default_value("127.0.0.1")
                .help("Remote kvsd server host")
                .global(true),
        )
        .arg(
            Arg::new(MUST_ARG_PORT)
                .long("port")
                .default_value(DEFAULT_PORT)
                .help("Server listening port")
                .global(true),
        )
        .arg(
            Arg::new(MUST_ARG_USERNAME)
                .long("username")
                .env("KVSD_USERNAME")
                .help("Username")
                .default_value("kvsduser")
                .global(true),
        )
        .arg(
            Arg::new(MUST_ARG_PASSWORD)
                .long("password")
                .env("KVSD_PASSWORD")
                .help("Password")
                .default_value("secret")
                .global(true),
        )
        .arg(
            Arg::new(MUST_ARG_DISABLE_TLS)
                .long("disable-tls")
                .env("KVSD_DISABLE_TLS")
                .help("disable tls connections")
                .global(true),
        )
        .subcommand(cli::ping::subcommand())
        .subcommand(cli::server::subcommand())
        .subcommand(cli::set::subcommand())
        .subcommand(cli::get::subcommand())
        .subcommand(cli::delete::subcommand())
        .subcommand_required(true)
        .propagate_version(true)
}

pub(crate) async fn authenticate(m: &ArgMatches) -> crate::Result<Box<dyn Api>> {
    let (host, port) = (
        m.get_one::<String>(MUST_ARG_HOST).unwrap(),
        m.get_one::<u16>(MUST_ARG_PORT)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid port"))?,
    );
    let (user, pass) = (
        m.get_one::<String>(MUST_ARG_USERNAME).unwrap(),
        m.get_one::<String>(MUST_ARG_PASSWORD).unwrap(),
    );
    let disable_tls = m.contains_id(MUST_ARG_DISABLE_TLS);

    let client: Box<dyn Api> = if disable_tls {
        UnauthenticatedClient::insecure_from_addr(host, *port)
            .await?
            .authenticate(user, pass)
            .await
            .map(Box::new)?
    } else {
        UnauthenticatedClient::from_addr(host, *port)
            .await?
            .authenticate(user, pass)
            .await
            .map(Box::new)?
    };

    Ok(client)
}
