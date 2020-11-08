use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::connection::Connection;
use crate::protocol::message::{Authenticate, Message, Ping};
use crate::{KvsError, Result};

pub struct Client {
    connection: Connection,
}

pub struct UnauthenticatedClient {
    client: Client,
}

impl UnauthenticatedClient {
    pub fn new(stream: impl Into<TcpStream>) -> Self {
        Self {
            client: Client::new(stream),
        }
    }

    pub async fn from_addr(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(UnauthenticatedClient::new(
            tokio::net::TcpStream::connect(addr).await?,
        ))
    }

    pub async fn authenticate<S1, S2>(mut self, username: S1, password: S2) -> Result<Client>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let authenticate = Authenticate::new(username.into(), password.into());
        self.client.connection.write_message(authenticate).await?;
        match self.client.connection.read_message().await? {
            Some(Message::Success(_)) => Ok(self.client),
            // format!(..).into() does not work :(
            msg => Err(KvsError::Internal(
                Box::<dyn std::error::Error + Send + Sync>::from(format!(
                    "unexpected message {:?}",
                    msg
                )),
            )),
        }
    }
}

impl Client {
    fn new(stream: impl Into<TcpStream>) -> Self {
        Self {
            connection: Connection::new(stream.into(), Some(1024 * 4)),
        }
    }
    // Return ping latency.
    pub async fn ping(&mut self) -> Result<chrono::Duration> {
        let ping = Ping::new().record_client_time();
        self.connection.write_message(ping).await?;
        match self.connection.read_message().await? {
            Some(Message::Ping(ping)) => Ok(ping.latency().unwrap()),
            msg => Err(format!("unexpected message {:?}", msg).into()),
        }
    }
}
