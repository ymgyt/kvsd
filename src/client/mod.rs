use tokio::net::TcpStream;

use crate::Result;

mod tcp;

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: impl Into<TcpStream>) -> Self {
        Self {
            stream: stream.into(),
        }
    }

    pub async fn echo(&mut self, message: impl Into<String>) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let message = message.into();
        self.stream.write_all(message.as_bytes()).await?;

        let mut buff = Vec::with_capacity(message.len());
        self.stream.read_exact(buff.as_mut_slice()).await?;

        println!("{}", String::from_utf8_lossy(buff.as_slice()));

        Ok(())
    }
}
