use std::convert::TryFrom;

use crate::common::{Error, ErrorKind, Result};
use crate::protocol::message::{Authenticate, Fail, Get, MessageFrames, Parse, Ping, Set, Success};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageType {
    Ping = 1,
    Authenticate = 2,
    Success = 3,
    Fail = 4,
    Set = 5,
    Get = 6,
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
            4 => Ok(MessageType::Fail),
            5 => Ok(MessageType::Set),
            6 => Ok(MessageType::Get),
            _ => Err(Error::from(ErrorKind::UnknownMessageType {
                message_type: n,
            })),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Message {
    Ping(Ping),
    Authenticate(Authenticate),
    Success(Success),
    Fail(Fail),
    Set(Set),
    Get(Get),
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
            MessageType::Ping => Message::Ping(Ping::parse_frames(&mut parse)?),
            MessageType::Success => Message::Success(Success::new()),
            MessageType::Fail => Message::Fail(Fail::parse_frames(&mut parse)?),
            MessageType::Set => Message::Set(Set::parse_frames(&mut parse)?),
            MessageType::Get => Message::Get(Get::parse_frames(&mut parse)?),
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
            Message::Fail(m) => m.into(),
            Message::Set(m) => m.into(),
            Message::Get(m) => m.into(),
        }
    }
}
