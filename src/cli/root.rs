use clap::{App, AppSettings, Arg, ArgMatches};

use crate::cli;
use crate::client::tcp::UnauthenticatedClient;
use crate::client::Api;
use crate::server::DEFAULT_PORT;

pub(super) const MUST_ARG_HOST: &str = "host";
pub(super) const MUST_ARG_PORT: &str = "port";
pub(super) const MUST_ARG_USERNAME: &str = "username";
pub(super) const MUST_ARG_PASSWORD: &str = "password";
pub(super) const MUST_ARG_DISABLE_TLS: &str = "disable-tls";

/// new return top level clap App.
pub fn new() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name(MUST_ARG_HOST)
                .long("host")
                .env("KVSD_HOST")
                .default_value("127.0.0.1")
                .help("Remote kvsd server host")
                .global(true),
        )
        .arg(
            Arg::with_name(MUST_ARG_PORT)
                .long("port")
                .takes_value(true)
                .default_value(&DEFAULT_PORT)
                .help("Server listening port")
                .global(true),
        )
        .arg(
            Arg::with_name(MUST_ARG_USERNAME)
                .long("username")
                .env("KVSD_USERNAME")
                .takes_value(true)
                .help("Username")
                .default_value("kvsduser")
                .global(true),
        )
        .arg(
            Arg::with_name(MUST_ARG_PASSWORD)
                .long("password")
                .env("KVSD_PASSWORD")
                .takes_value(true)
                .help("Password")
                .default_value("secret")
                .global(true),
        )
        .arg(
            Arg::with_name(MUST_ARG_DISABLE_TLS)
                .long("disable-tls")
                .env("KVSD_DISABLE_TLS")
                .takes_value(false)
                .help("disable tls connections")
                .global(true),
        )
        .subcommand(cli::ping::subcommand())
        .subcommand(cli::server::subcommand())
        .subcommand(cli::set::subcommand())
        .subcommand(cli::get::subcommand())
        .subcommand(cli::delete::subcommand())
        .settings(&[
            AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands,
            AppSettings::DeriveDisplayOrder,
        ])
        .global_settings(&[AppSettings::ColoredHelp, AppSettings::ColorAuto])
}

pub(crate) async fn authenticate(m: &ArgMatches<'_>) -> crate::Result<Box<dyn Api>> {
    let (host, port) = (
        m.value_of(MUST_ARG_HOST).unwrap(),
        m.value_of(MUST_ARG_PORT)
            .and_then(|port| port.parse::<u16>().ok())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid port"))?,
    );
    let (user, pass) = (
        m.value_of(MUST_ARG_USERNAME).unwrap(),
        m.value_of(MUST_ARG_PASSWORD).unwrap(),
    );
    let disable_tls = m.is_present(MUST_ARG_DISABLE_TLS);

    let client: Box<dyn Api> = if disable_tls {
        UnauthenticatedClient::insecure_from_addr(host, port)
            .await?
            .authenticate(user, pass)
            .await
            .map(Box::new)?
    } else {
        UnauthenticatedClient::from_addr(host, port)
            .await?
            .authenticate(user, pass)
            .await
            .map(Box::new)?
    };

    Ok(client)
}
