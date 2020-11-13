pub mod ping;
pub mod server;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::cli;
use crate::server::DEFAULT_PORT;

pub const PING: &str = "ping";
pub const SERVER: &str = "server";

pub(super) const MUST_ARG_HOST: &str = "host";
pub(super) const MUST_ARG_PORT: &str = "port";
pub(super) const MUST_ARG_USERNAME: &str = "username";
pub(super) const MUST_ARG_PASSWORD: &str = "password";

pub fn new() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name(MUST_ARG_HOST)
                .long("host")
                .env("KVS_HOST")
                .default_value("localhost")
                .help("Remote kvs server host")
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
                .env("KVS_USERNAME")
                .takes_value(true)
                .help("Username")
                .default_value("kvsuser")
                .global(true),
        )
        .arg(
            Arg::with_name(MUST_ARG_PASSWORD)
                .long("password")
                .env("KVS_PASSWORD")
                .takes_value(true)
                .help("Password")
                .default_value("secret")
                .global(true),
        )
        .subcommand(cli::ping::subcommand())
        .subcommand(cli::server::subcommand())
        .settings(&[
            AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands,
            AppSettings::DeriveDisplayOrder,
        ])
        .global_settings(&[AppSettings::ColoredHelp, AppSettings::ColorAuto])
}

pub(super) fn server_addr(m: &ArgMatches) -> String {
    let host = m.value_of(MUST_ARG_HOST).unwrap();
    let port = m.value_of(MUST_ARG_PORT).unwrap();
    let addr = format!("{}:{}", host, port);

    addr
}

pub(super) async fn authenticate(m: &ArgMatches<'_>) -> crate::Result<crate::client::tcp::Client> {
    crate::client::tcp::UnauthenticatedClient::from_addr(server_addr(m))
        .await?
        .authenticate(
            m.value_of(MUST_ARG_USERNAME).unwrap(),
            m.value_of(MUST_ARG_PASSWORD).unwrap(),
        )
        .await
}
