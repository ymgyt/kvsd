use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli::{server_addr, PING};
use crate::client::tcp::Client;
use crate::Result;

const MUST_ARG_PING_COUNT: &str = "count";

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(PING).about("Ping to kvs server").arg(
        Arg::with_name(MUST_ARG_PING_COUNT)
            .takes_value(true)
            .long("count")
            .short("c")
            .default_value("1")
            .help("Ping counts"),
    )
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let addr = server_addr(m);
    let count = m
        .value_of(MUST_ARG_PING_COUNT)
        .and_then(|n| n.parse().ok())
        .unwrap_or(1);
    let mut current = 1;

    while current <= count {
        // for now, server disconnects tcp connections every time after it process message.
        // so connects every time.
        let mut client = Client::from_addr(addr.clone()).await?;
        // TODO: get from user
        client.authenticate("user", "pass").await?;
        let latency = client.ping().await?;
        println!(
            "ping (latency {}ms) {}/{}",
            latency.num_milliseconds(),
            current,
            count
        );
        current += 1;
    }

    Ok(())
}
