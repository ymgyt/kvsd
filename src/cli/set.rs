use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli::{authenticate, SET};
use crate::protocol::{Key, Value};
use crate::Result;

const MUST_ARG_KEY: &str = "set_key";
const MUST_ARG_VALUE: &str = "set_value";

pub(super) fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SET)
        .about("Set key value")
        .arg(
            Arg::with_name(MUST_ARG_KEY)
                .index(1)
                .required(true)
                .help("Key")
                .value_name("KEY"),
        )
        .arg(
            Arg::with_name(MUST_ARG_VALUE)
                .index(2)
                .required(true)
                .help("Value")
                .value_name("VALUE"),
        )
}

pub async fn run(m: &ArgMatches<'_>) -> Result<()> {
    let key = m.value_of(MUST_ARG_KEY).unwrap();
    let value = m.value_of(MUST_ARG_VALUE).unwrap();

    let key = Key::new(key)?;
    let value = Value::new(value.as_bytes())?;

    let mut client = authenticate(m).await?;

    client.set(key, value).await?;

    Ok(())
}
