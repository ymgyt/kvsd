use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli::{server_addr, ECHO};
use crate::client::tcp::Client;
use crate::Result;

pub const MUST_ECHO_MESSAGE: &str = "message";

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(ECHO)
        .arg(
            Arg::with_name(MUST_ECHO_MESSAGE)
                .takes_value(true)
                .default_value("Hello kvs!"),
        )
        .about("Echo message")
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let message = m.value_of(MUST_ECHO_MESSAGE).unwrap();
    let addr = server_addr(m);
    let stream = tokio::net::TcpStream::connect(&addr).await?;

    let mut client = Client::new(stream);

    client.echo(message).await?;

    Ok(())
}
