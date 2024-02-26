use clap::{Arg, ArgMatches, Command};

use crate::cli::{authenticate, PING};
use crate::Result;

const MUST_ARG_PING_COUNT: &str = "count";

pub(super) fn subcommand() -> Command {
    Command::new(PING).about("Ping to kvsd server").arg(
        Arg::new(MUST_ARG_PING_COUNT)
            .long("count")
            .short('c')
            .default_value("1")
            .help("Ping counts"),
    )
}

/// Launch the ping command.
pub async fn run(m: &ArgMatches) -> Result<()> {
    let count = m.get_one::<i32>(MUST_ARG_PING_COUNT).unwrap_or(&1);
    let mut current = 1;

    let mut client = authenticate(m).await?;

    while current <= *count {
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
