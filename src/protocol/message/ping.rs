use chrono::Utc;

use crate::common::Time;
use crate::protocol::message::{MessageFrames, MessageType};

#[derive(Debug,Clone,PartialEq)]
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

    pub(crate) fn record_client_time(&mut self) {
        self.client_timestamp = Some(Utc::now());
    }

    pub(crate) fn record_server_time(&mut self, time: Time) {
        self.server_timestamp = Some(time);
    }
}

impl Into<MessageFrames> for Ping {
    fn into(self) -> MessageFrames {
        let mut frames = MessageFrames::with_capacity(MessageType::Ping, 2);

        // TODO: impl time framing
        frames.push_string("dummy");
        frames.push_string("dummy");

        frames
    }
}
