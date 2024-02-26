use clap::{Arg, ArgMatches, Command};

use crate::cli::{authenticate, SET};
use crate::protocol::{Key, Value};
use crate::Result;

const MUST_ARG_KEY: &str = "set_key";
const MUST_ARG_VALUE: &str = "set_value";

pub(super) fn subcommand() -> Command {
    Command::new(SET)
        .about("Set key value")
        .arg(
            Arg::new(MUST_ARG_KEY)
                .index(1)
                .required(true)
                .help("Key")
                .value_name("KEY"),
        )
        .arg(
            Arg::new(MUST_ARG_VALUE)
                .index(2)
                .required(true)
                .help("Value")
                .value_name("VALUE"),
        )
}

/// Launch the set command.
pub async fn run(m: &ArgMatches) -> Result<()> {
    let key = m.get_one::<String>(MUST_ARG_KEY).unwrap();
    let value = m.get_one::<String>(MUST_ARG_VALUE).unwrap();

    let key = Key::new(key)?;
    let value = Value::new(value.as_bytes())?;

    let mut client = authenticate(m).await?;

    if client.set(key, value).await.is_ok() {
        println!("OK");
    }

    Ok(())
}
