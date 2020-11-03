use clap::{App, ArgMatches, SubCommand};

use crate::cli::{server_addr, PING};
use crate::client::tcp::Client;
use crate::Result;

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(PING).about("Ping to kvs server")
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let addr = server_addr(m);
    let mut client = Client::from_addr(addr).await?;
    client.ping().await
}
