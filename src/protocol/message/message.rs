use std::convert::TryFrom;

use crate::common::{Error, ErrorKind, Result};
use crate::protocol::message::{Authenticate, MessageFrames, Parse, Ping};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageType {
    Ping = 1,
    Authenticate = 2,
    Success = 3,
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for MessageType {
    type Error = Error;
    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(MessageType::Ping),
            2 => Ok(MessageType::Authenticate),
            3 => Ok(MessageType::Success),
            _ => Err(Error::from(ErrorKind::UnknownMessageType {
                message_type: n,
            })),
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub(crate) enum Message {
    Ping(Ping),
    Authenticate(Authenticate),
    Success(Success),
}

impl Message {
    pub(crate) fn from_frames(frames: MessageFrames) -> Result<Message> {
        let mut parse = Parse::new(frames);
        let message_type = parse
            .message_type()
            .ok_or_else(|| ErrorKind::NetworkFraming("message type not found".into()))?;

        let message = match message_type {
            MessageType::Authenticate => {
                Message::Authenticate(Authenticate::parse_frames(&mut parse)?)
            }
            _ => unreachable!(),
        };

        Ok(message)
    }
}

impl Into<MessageFrames> for Message {
    fn into(self) -> MessageFrames {
        match self {
            Message::Ping(m) => m.into(),
            Message::Authenticate(m) => m.into(),
            Message::Success(m) => m.into(),
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub(crate) struct Success {}

impl Success {
    pub(crate) fn new() -> Success {
        Self {}
    }
}

impl Into<MessageFrames> for Success {
    fn into(self) -> MessageFrames {
        MessageFrames::with_capacity(MessageType::Success, 0)
    }
}
