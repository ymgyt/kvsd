use std::io::{self, Cursor};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::time::Duration;

use crate::common::{Error, Result};
use crate::error::internal::ErrorKind;
use crate::protocol::message::{frameprefix, Frame, FrameError, Message, MessageFrames, DELIMITER};

pub struct Connection<T = TcpStream> {
    stream: BufWriter<T>,
    // The buffer for reading frames.
    buffer: BytesMut,
}

impl<T> Connection<T>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    pub(crate) fn new(stream: T, buffer_size: Option<usize>) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(buffer_size.unwrap_or(4 * 1024)),
        }
    }

    pub(crate) async fn write_message(&mut self, message: impl Into<MessageFrames>) -> Result<()> {
        let frames = message.into();

        self.stream.write_u8(frameprefix::MESSAGE_FRAMES).await?;
        self.write_decimal(frames.len()).await?;

        for frame in frames {
            self.write_frame(frame).await?
        }

        self.stream.flush().await?;
        Ok(())
    }

    async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        match frame {
            Frame::MessageType(mt) => {
                self.stream.write_u8(frameprefix::MESSAGE_TYPE).await?;
                self.stream.write_u8(mt.into()).await?;
            }
            Frame::String(val) => {
                self.stream.write_u8(frameprefix::STRING).await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(DELIMITER).await?;
            }
            Frame::Bytes(val) => {
                self.stream.write_u8(frameprefix::BYTES).await?;
                self.write_decimal(val.len() as u64).await?;
                self.stream.write_all(val.as_ref()).await?;
                self.stream.write_all(DELIMITER).await?;
            }
            Frame::Time(val) => {
                self.stream.write_u8(frameprefix::TIME).await?;
                self.stream.write_all(val.to_rfc3339().as_bytes()).await?;
                self.stream.write_all(DELIMITER).await?;
            }
            Frame::Null => {
                self.stream.write_u8(frameprefix::NULL).await?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    pub(crate) async fn read_message_with_timeout(
        &mut self,
        duration: Duration,
    ) -> Result<Option<Message>> {
        match tokio::time::timeout(duration, self.read_message()).await {
            Ok(read_result) => read_result,
            Err(elapsed) => Err(Error::from(elapsed)),
        }
    }

    pub(crate) async fn read_message(&mut self) -> Result<Option<Message>> {
        match self.read_message_frames().await? {
            Some(message_frames) => Ok(Some(Message::from_frames(message_frames)?)),
            None => Ok(None),
        }
    }

    async fn read_message_frames(&mut self) -> Result<Option<MessageFrames>> {
        loop {
            if let Some(message_frames) = self.parse_message_frames()? {
                return Ok(Some(message_frames));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(ErrorKind::ConnectionResetByPeer.into())
                };
            }
        }
    }

    fn parse_message_frames(&mut self) -> Result<Option<MessageFrames>> {
        use FrameError::Incomplete;

        let mut buf = Cursor::new(&self.buffer[..]);

        match MessageFrames::check_parse(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let message_frames = MessageFrames::parse(&mut buf)?;
                self.buffer.advance(len);

                Ok(Some(message_frames))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(DELIMITER).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::{Authenticate, Fail, FailCode, Message, Ping, Set, Success};
    use crate::protocol::{Key, Value};

    #[test]
    fn message_frames() {
        tokio_test::block_on(async move {
            let (client, server) = tokio::io::duplex(1024);
            let mut client_conn = Connection::new(client, None);
            let mut server_conn = Connection::new(server, None);

            let messages: Vec<Message> = vec![
                Message::Authenticate(Authenticate::new("user", "pass")),
                Message::Ping(Ping::new().record_client_time()),
                Message::Success(Success::new()),
                Message::Fail(Fail::new(FailCode::Unauthenticated)),
                Message::Fail(
                    Fail::new(FailCode::UnexpectedMessage).with_message("unexpected message X"),
                ),
                Message::Set(Set::new(
                    Key::new("key1").unwrap(),
                    Value::new(b"value1".as_ref()).unwrap(),
                )),
            ];
            let messages_clone = messages.clone();

            let write_handle = tokio::spawn(async move {
                for message in messages {
                    client_conn.write_message(message).await.unwrap();
                }
            });

            let read_handle = tokio::spawn(async move {
                for want in messages_clone {
                    let got = server_conn.read_message().await.unwrap().unwrap();
                    assert_eq!(want, got);
                }
            });

            write_handle.await.unwrap();
            read_handle.await.unwrap();
        })
    }
}
