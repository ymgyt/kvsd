use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::connection::Connection;
use crate::protocol::message::{Message, Ping};
use crate::Result;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub fn new(stream: impl Into<TcpStream>) -> Self {
        Self {
            connection: Connection::new(stream.into()),
        }
    }
    pub async fn from_addr(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(Client::new(tokio::net::TcpStream::connect(addr).await?))
    }

    // Return ping latency.
    pub async fn ping(&mut self) -> Result<chrono::Duration> {
        let ping = Ping::new().record_client_time();
        let message = Message::Ping(ping);
        self.connection.write_message(message).await?;

        let message = self.connection.read_message().await?;
        match message {
            Message::Ping(ping) => Ok(ping
                .latency()
                .expect("client/server timestamp not recorded")),
        }
    }
}
