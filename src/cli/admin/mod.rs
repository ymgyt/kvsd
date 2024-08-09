use crate::Result;
use clap::{Args, Subcommand};

mod dump;

#[derive(Args, Debug)]
pub struct AdminCommand {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Dump table
    Dump(dump::DumpCommand),
}

impl AdminCommand {
    pub async fn run(self) -> Result<()> {
        let AdminCommand { command } = self;

        match command {
            Command::Dump(dump) => dump.run().await,
        }
    }
}
