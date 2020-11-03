pub(crate) mod ping;

pub(crate) use ping::Ping;

use std::convert::TryFrom;

use tokio::io::AsyncWriteExt;

use crate::common::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageType {
    Ping = 1,
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
            _ => Err(Error::from(ErrorKind::UnknownMessageType {
                message_type: n,
            })),
        }
    }
}

pub(crate) enum Message {
    Ping(Ping),
}

impl Message {
    pub(crate) fn message_type(&self) -> MessageType {
        match self {
            Message::Ping(_) => MessageType::Ping,
        }
    }

    pub(crate) fn encoded_len(&self) -> u64 {
        match self {
            Message::Ping(ref ping) => ping.encoded_len(),
        }
    }

    pub(crate) async fn encode_to<W>(&self, writer: W) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        match self {
            Message::Ping(ref ping) => ping.encode_to(writer).await,
        }
    }
}
