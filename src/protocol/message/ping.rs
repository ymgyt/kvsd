use chrono::{DateTime, TimeZone, Utc};

use crate::common::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug)]
pub(crate) struct Ping {
    client_timestamp: Option<DateTime<Utc>>,
    server_timestamp: Option<DateTime<Utc>>,
}

impl Ping {
    pub(crate) fn new() -> Self {
        Self {
            client_timestamp: None,
            server_timestamp: None,
        }
    }
    pub(crate) fn latency(&self) -> Option<chrono::Duration> {
        if let (Some(client), Some(server)) = (self.client_timestamp, self.server_timestamp) {
            Some(server - client)
        } else {
            None
        }
    }

    pub(crate) fn record_client_time(mut self) -> Self {
        self.client_timestamp = Some(Utc::now());
        self
    }

    pub(crate) fn record_server_time(&mut self, time: DateTime<Utc>) {
        self.server_timestamp = Some(time);
    }

    pub(crate) fn encoded_len(&self) -> u64 {
        16
    }

    pub(crate) async fn encode_to<W>(&self, mut writer: W) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        writer
            .write_i64(
                self.client_timestamp
                    .map(|t| t.timestamp_nanos())
                    .unwrap_or(0),
            )
            .await?;
        writer
            .write_i64(
                self.server_timestamp
                    .map(|t| t.timestamp_nanos())
                    .unwrap_or(0),
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn decode_from<R>(mut reader: R) -> Result<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        let client_timestamp = Ping::parse_timestamp(reader.read_i64().await?);
        let server_timestamp = Ping::parse_timestamp(reader.read_i64().await?);

        Ok(Self {
            client_timestamp,
            server_timestamp,
        })
    }

    fn parse_timestamp(timestamp: i64) -> Option<DateTime<Utc>> {
        if timestamp == 0 {
            None
        } else {
            Some(Utc.timestamp_nanos(timestamp))
        }
    }
}
