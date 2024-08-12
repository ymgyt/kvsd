use crate::Result;
use clap::{Parser, Subcommand};

mod table;

/// Kvsadmin command
#[derive(Parser, Debug)]
#[command(version, propagate_version = true, subcommand_required = true)]
pub struct KvsadminCommand {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Table(table::TableCommand),
}

impl KvsadminCommand {
    pub async fn run(self) -> Result<()> {
        let KvsadminCommand { command } = self;

        match command {
            Command::Table(table) => table.run().await,
        }
    }
}

pub fn parse() -> KvsadminCommand {
    KvsadminCommand::parse()
}
