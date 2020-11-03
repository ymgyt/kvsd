use std::convert::TryFrom;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::common::Result;
use crate::protocol::message::{Header, Message, MessageType};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub(crate) async fn write_message(&mut self, message: Message) -> Result<()> {
        Connection::encode_message(&mut self.stream, message).await
    }

    async fn encode_message<W>(mut writer: W, message: Message) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        writer.write_u8(message.header.message_type as u8).await?;
        writer.write_u16(message.header.flag.to_u16()).await?;
        writer.write_u64(message.header.body_bytes as u64).await?;
        writer.write_all(message.body.as_ref()).await?;

        Ok(())
    }

    pub(crate) async fn read_message(&mut self) -> Result<Message> {
        Connection::decode_message(&mut self.stream).await
    }

    async fn decode_message<R>(mut reader: R) -> Result<Message>
    where
        R: AsyncReadExt + Unpin,
    {
        let message_type = reader.read_u8().await?;
        let flag = reader.read_u16().await?.into();
        let body_bytes = reader.read_u64().await?;

        let mut buf = Vec::with_capacity(body_bytes as usize);
        reader.take(body_bytes).read_to_end(buf.as_mut()).await?;

        let header = Header {
            message_type: MessageType::try_from(message_type)?,
            flag,
            body_bytes: body_bytes as usize,
        };

        Ok(Message::with(header, buf))
    }
}
