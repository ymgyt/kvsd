use tokio::net::{TcpListener, TcpStream};

use crate::common::{info, Result};

struct Server {}

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
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }
                        stream.write_all(&buff[0..n]).await.unwrap();
                    }
                    Err(err) => {
                        eprintln!("err! {}", err);
                        return;
                    }
                }
            }
        });
    }
}
