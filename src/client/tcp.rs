use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::ServerCertVerified;
use tokio_rustls::{rustls, webpki, TlsConnector};

use crate::client::Api;
use crate::common::info;
use crate::protocol::connection::Connection;
use crate::protocol::message::{Authenticate, Delete, Get, Message, Ping, Set};
use crate::protocol::{Key, Value};
use crate::{KvsdError, Result};

/// Implementation of client api by tcp.
pub struct Client<T> {
    connection: Connection<T>,
}

/// A client that is not authenticated by the server.
/// it provide processing allowed to clients that are not authenticate.
pub struct UnauthenticatedClient<T> {
    client: Client<T>,
}

impl<T> UnauthenticatedClient<T>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    /// Construct Client by given stream.
    pub fn new(stream: T) -> Self {
        Self {
            client: Client::new(stream),
        }
    }

    /// Try authenticate by given credential.
    pub async fn authenticate<S1, S2>(mut self, username: S1, password: S2) -> Result<Client<T>>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let authenticate = Authenticate::new(username.into(), password.into());
        self.client.connection.write_message(authenticate).await?;
        match self.client.connection.read_message().await? {
            Some(Message::Success(_)) => Ok(self.client),
            Some(Message::Fail(_)) => Err(KvsdError::Unauthenticated),
            // format!(..).into() does not work :(
            msg => Err(KvsdError::Internal(Box::<
                dyn std::error::Error + Send + Sync,
            >::from(format!(
                "unexpected message {:?}",
                msg
            )))),
        }
    }
}

impl UnauthenticatedClient<TcpStream> {
    /// Return client that is not protected by TLS for tcp communication.
    pub async fn insecure_from_addr(host: impl AsRef<str>, port: u16) -> Result<Self> {
        let addr = (host.as_ref(), port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

        info!(%addr, "Connecting");

        let stream = tokio::net::TcpStream::connect(addr).await?;

        Ok(UnauthenticatedClient::new(stream))
    }
}

impl UnauthenticatedClient<TlsStream<TcpStream>> {
    /// Return the client with a TLS connection to the given address.
    pub async fn from_addr(host: impl Into<String>, port: u16) -> Result<Self> {
        let host = host.into();
        let addr = (host.as_str(), port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

        let mut tls_config = rustls::ClientConfig::new();
        tls_config
            .dangerous()
            .set_certificate_verifier(Arc::new(DangerousServerCertVerifier::new()));

        let connector = TlsConnector::from(Arc::new(tls_config));

        // TODO: remove hard code
        let domain = webpki::DNSNameRef::try_from_ascii_str("localhost")
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid host"))?;

        info!(%addr,?domain, "Connecting");

        let stream = tokio::net::TcpStream::connect(addr).await?;

        Ok(UnauthenticatedClient::new(
            connector.connect(domain, stream).await?,
        ))
    }
}

impl<T> Client<T>
where
    T: AsyncWrite + AsyncRead + Unpin,
{
    fn new(stream: T) -> Self {
        Self {
            connection: Connection::new(stream, Some(1024 * 4)),
        }
    }
}

#[async_trait]
impl<T> Api for Client<T>
where
    T: AsyncWrite + AsyncRead + Unpin + Send,
{
    // Return ping latency.
    async fn ping(&mut self) -> Result<chrono::Duration> {
        let ping = Ping::new().record_client_time();
        self.connection.write_message(ping).await?;
        match self.connection.read_message().await? {
            Some(Message::Ping(ping)) => Ok(ping.latency().unwrap()),
            msg => Err(format!("unexpected message {:?}", msg).into()),
        }
    }

    async fn set(&mut self, key: Key, value: Value) -> Result<()> {
        let set = Set::new(key, value);
        self.connection.write_message(set).await?;
        match self.connection.read_message().await? {
            Some(Message::Success(_)) => Ok(()),
            msg => Err(KvsdError::Internal(Box::<
                dyn std::error::Error + Send + Sync,
            >::from(format!(
                "unexpected message: {:?}",
                msg
            )))),
        }
    }

    async fn get(&mut self, key: Key) -> Result<Option<Value>> {
        let get = Get::new(key);
        self.connection.write_message(get).await?;
        match self.connection.read_message().await? {
            Some(Message::Success(success)) => Ok(success.value()),
            _ => unreachable!(),
        }
    }

    async fn delete(&mut self, key: Key) -> Result<Option<Value>> {
        let delete = Delete::new(key);
        self.connection.write_message(delete).await?;
        match self.connection.read_message().await? {
            Some(Message::Success(success)) => Ok(success.value()),
            _ => unreachable!(),
        }
    }
}

struct DangerousServerCertVerifier {}

impl DangerousServerCertVerifier {
    fn new() -> Self {
        Self {}
    }
}

impl rustls::ServerCertVerifier for DangerousServerCertVerifier {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef<'_>,
        _oscp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}
