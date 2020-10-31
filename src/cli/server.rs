use clap::{App, ArgMatches, SubCommand};

use crate::cli::{server_addr, SERVER};
use crate::common::info;
use crate::server::tcp::Server;
use crate::Result;

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SERVER).about("Running kvs server")
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let addr = server_addr(m);
    info!("Listening {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let server = Server::default();

    server.run(listener).await
}
