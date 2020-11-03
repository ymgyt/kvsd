use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::common::Result;
use crate::protocol::message::{Message, MessageType, Ping};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub(crate) async fn write_message(&mut self, message: Message) -> Result<()> {
        let mut writer =
            BufWriter::with_capacity(16 + message.encoded_len() as usize, &mut self.stream);
        Connection::encode_message(&mut writer, message).await?;
        Ok(writer.flush().await?)
    }

    async fn encode_message<W>(mut writer: W, message: Message) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        writer.write_u8(message.message_type().into()).await?;
        writer.write_u64(message.encoded_len()).await?;
        message.encode_to(writer).await?;

        Ok(())
    }

    pub(crate) async fn read_message(&mut self) -> Result<Message> {
        Connection::decode_message(&mut self.stream).await
    }

    async fn decode_message<R>(mut reader: R) -> Result<Message>
    where
        R: AsyncReadExt + Unpin,
    {
        use std::convert::TryInto;
        let message_type: MessageType = reader.read_u8().await?.try_into()?;
        let len = reader.read_u64().await?;
        let reader = reader.take(len);
        match message_type {
            MessageType::Ping => Ok(Message::Ping(Ping::decode_from(reader).await?)),
        }
    }
}
