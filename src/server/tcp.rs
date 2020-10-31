use tokio::net::{TcpListener, TcpStream};

use crate::common::{error, info};
use crate::Result;

impl Default for Server {
    fn default() -> Self {
        Self {}
    }
}

pub(crate) struct Server {}

impl Server {
    pub(crate) async fn run(self, listener: TcpListener) -> Result<()> {
        loop {
            let (socket, addr) = listener.accept().await?;
            info!("Accept from {}", addr);

            self.handle(socket);
        }
    }

    fn handle(&self, mut stream: TcpStream) {
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let mut buff = [0u8; 1024];
            loop {
                match stream.read(&mut buff).await {
                    Ok(0) => return,
                    Ok(n) => {
                        if let Err(err) = stream.write_all(&buff[..n]).await {
                            error!("write error {}", err);
                            return;
                        };
                    }
                    Err(err) => {
                        error!("read error {}", err);
                        return;
                    }
                }
            }
        });
    }
}
