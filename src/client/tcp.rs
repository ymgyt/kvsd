use tokio::net::TcpStream;

use crate::common::info;
use crate::Result;

pub struct Client {
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

        if message.is_empty() {
            return Ok(());
        }

        self.stream.write_all(message.as_bytes()).await?;

        let mut buf = [0u8; 256];
        let mut sum = 0;
        while sum < message.len() {
            let n = self.stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }

            sum = sum.saturating_add(n);
            info!("read {} bytes ({}/{})", n, sum, message.len());

            println!("{}", String::from_utf8_lossy(&buf[..n]));
        }

        Ok(())
    }
}
