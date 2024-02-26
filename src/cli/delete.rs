use clap::{Arg, ArgMatches, Command};

use crate::cli::{authenticate, DELETE};
use crate::protocol::Key;
use crate::Result;

const MUST_ARG_KEY: &str = "key";

pub(super) fn subcommand() -> Command {
    Command::new(DELETE).about("Delete value").arg(
        Arg::new(MUST_ARG_KEY)
            .index(1)
            .required(true)
            .help("Key")
            .value_name("KEY"),
    )
}

/// launch the delete command.
pub async fn run(m: &ArgMatches) -> Result<()> {
    let key = m.get_one::<String>(MUST_ARG_KEY).unwrap();

    let key = Key::new(key)?;

    let mut client = authenticate(m).await?;

    match client.delete(key).await? {
        Some(value) => {
            println!("OK old value: {:?}", value);
        }
        None => {
            println!("OK");
        }
    }

    Ok(())
}
