use std::path::PathBuf;

use clap::Args;

use crate::Result;

#[derive(Args, Debug)]
pub struct DumpCommand {
    /// Path to kvsd file
    #[arg()]
    path: PathBuf,

    /// Dump index
    #[arg(long)]
    index: bool,
}

impl DumpCommand {
    pub async fn run(self) -> Result<()> {
        // Construct table from path
        // Prepare writer
        // call table.dump() method
        Ok(())
    }
}
