use std::convert::TryFrom;

use tokio::io::AsyncWriteExt;

use crate::common::{Error, ErrorKind, Result};
use crate::protocol::message::{Authenticate, Ping};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageType {
    Ping = 1,
    Authenticate = 2,
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
            _ => Err(Error::from(ErrorKind::UnknownMessageType {
                message_type: n,
            })),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Message {
    Ping(Ping),
    Authenticate(Authenticate),
}
