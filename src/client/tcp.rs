use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::connection::Connection;
use crate::protocol::message::{Authenticate, Message, Ping};
use crate::Result;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub fn new(stream: impl Into<TcpStream>) -> Self {
        Self {
            connection: Connection::new(stream.into(), Some(1024 * 4)),
        }
    }
    pub async fn from_addr(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(Client::new(tokio::net::TcpStream::connect(addr).await?))
    }

    // Return ping latency.
    pub async fn ping(&mut self) -> Result<chrono::Duration> {
        todo!()
    }

    pub async fn authenticate<S1, S2>(&mut self, username: S1, password: S2) -> Result<()>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        todo!()
    }
}
