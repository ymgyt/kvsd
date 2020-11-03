use crate::common::Result;
use crate::protocol::message::{IntoMessage, MessageType};

#[derive(Debug)]
pub(crate) struct Ping {
    client_timestamp: Option<i64>,
    server_timestamp: Option<i64>,
}

impl Ping {
    pub fn new() -> Self {
        Self {
            client_timestamp: None,
            server_timestamp: None,
        }
    }

    pub fn record_client_time(mut self) -> Self {
        self.client_timestamp = Some(Ping::timestamp());
        self
    }

    pub fn timestamp() -> i64 {
        chrono::Utc::now().timestamp()
    }

    pub(crate) fn from_reader<R>(mut reader: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;

        let client_timestamp = match i64::from_be_bytes(buf) {
            0 => None,
            n => Some(n),
        };
        reader.read_exact(&mut buf)?;
        let server_timestamp = match i64::from_be_bytes(buf) {
            0 => None,
            n => Some(n),
        };

        Ok(Ping {
            client_timestamp,
            server_timestamp,
        })
    }
}

impl IntoMessage for Ping {
    fn message_type(&self) -> MessageType {
        MessageType::Ping
    }

    fn into_body(self) -> Result<Vec<u8>> {
        use std::io::Write;
        let mut buf = Vec::with_capacity(16);
        buf.write_all(&self.client_timestamp.unwrap_or(0).to_be_bytes())?;
        buf.write_all(&self.server_timestamp.unwrap_or(0).to_be_bytes())?;

        Ok(buf)
    }
}
