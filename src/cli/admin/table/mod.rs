mod dump;

use crate::Result;
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct TableCommand {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Dump(dump::DumpCommand),
}

impl TableCommand {
    pub async fn run(self) -> Result<()> {
        let TableCommand { command } = self;

        match command {
            Command::Dump(dump) => dump.run().await,
        }
    }
}
