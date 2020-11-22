use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli::{authenticate, GET};
use crate::protocol::Key;
use crate::Result;

const MUST_ARG_KEY: &str = "key";

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(GET).about("Get value").arg(
        Arg::with_name(MUST_ARG_KEY)
            .index(1)
            .required(true)
            .help("Key")
            .value_name("KEY"),
    )
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let key = m.value_of(MUST_ARG_KEY).unwrap();

    let key = Key::new(key)?;

    let mut client = authenticate(m).await?;

    match client.get(key).await? {
        Some(value) => {
            println!("{:?}", value);
        }
        None => {
            println!("Not Found");
        }
    }

    Ok(())
}
