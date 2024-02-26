use chrono::Utc;

use crate::common::{Result, Time};
use crate::protocol::message::{MessageFrames, MessageType, Parse};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ping {
    client_timestamp: Option<Time>,
    server_timestamp: Option<Time>,
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

    pub(crate) fn record_server_time(&mut self, time: Time) {
        self.server_timestamp = Some(time);
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Self> {
        let mut ping = Ping::new();

        ping.client_timestamp = parse.next_time_or_null()?;
        ping.server_timestamp = parse.next_time_or_null()?;

        Ok(ping)
    }
}

impl From<Ping> for MessageFrames {
    fn from(ping: Ping) -> Self {
        let mut frames = MessageFrames::with_capacity(MessageType::Ping, 2);

        frames.push_time_or_null(ping.client_timestamp);
        frames.push_time_or_null(ping.server_timestamp);

        frames
    }
}
