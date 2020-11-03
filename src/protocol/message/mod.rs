pub(crate) mod ping;

pub(crate) use ping::Ping;

use std::convert::TryFrom;

use crate::common::{Error, ErrorKind, Result};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum MessageType {
    Ping = 1,
}

impl TryFrom<u8> for MessageType {
    type Error = Error;
    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(MessageType::Ping),
            _ => Err(Error::from(ErrorKind::UnknownMessageType {
                message_type: n,
            })),
        }
    }
}

pub(crate) struct Flag(u16);

impl Flag {
    const REQUEST: u16 = 1;
    const RESPONSE: u16 = 1 << 1;

    pub(crate) fn to_u16(&self) -> u16 {
        self.0
    }

    fn request() -> Self {
        Self(Flag::REQUEST)
    }
}

impl From<u16> for Flag {
    fn from(n: u16) -> Self {
        Self(n)
    }
}

pub(crate) struct Header {
    pub(crate) message_type: MessageType,
    pub(crate) flag: Flag,
    pub(crate) body_bytes: usize,
}

pub(crate) struct Message {
    pub(crate) header: Header,
    pub(crate) body: Vec<u8>,
}

pub(crate) trait IntoMessage {
    fn message_type(&self) -> MessageType;
    fn into_body(self) -> Result<Vec<u8>>;
}

impl Message {
    pub fn new_request(m: impl IntoMessage) -> Result<Self> {
        let message_type = m.message_type();
        let body = m.into_body()?;

        let header = Header {
            message_type,
            flag: Flag::request(),
            body_bytes: body.len(),
        };

        Ok(Message::with(header, body))
    }

    pub(crate) fn message_type(&self) -> MessageType {
        self.header.message_type
    }

    pub(crate) fn with(header: Header, body: Vec<u8>) -> Self {
        Self { header, body }
    }
}
