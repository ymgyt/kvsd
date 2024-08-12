use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use clap::Args;
use serde_json::json;

use crate::{
    core::{EntryDump, Table},
    Result,
};

/// Dump table
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
        let DumpCommand { path, .. } = self;

        tracing::debug!("Dump {}", path.display());

        let mut table = Table::from_path(path).await?;
        let mut stdout = std::io::stdout();
        println!(r#"{{"entries": ["#);
        table
            .dump(|entry| {
                let v = dump(entry);
                serde_json::to_writer_pretty(&mut stdout, &v).unwrap();
            })
            .await?;
        println!(r#"]}}"#);

        Ok(())
    }
}

fn dump(entry: EntryDump) -> serde_json::Value {
    let time = Utc
        .timestamp_millis_opt(entry.timestamp_ns)
        .unwrap()
        .to_rfc3339();
    let value = String::from_utf8_lossy(&entry.value);

    json!({
        "time": time,
        "is_deleted": entry.is_deleted,
        "key": entry.key,
        "value": value,
    })
}
