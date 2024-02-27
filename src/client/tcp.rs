use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::{convert::TryFrom, io};

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::{
    client::TlsStream,
    rustls::{
        client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
        pki_types, SignatureScheme,
    },
};
use tokio_rustls::{rustls, TlsConnector};

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
        let tls_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(DangerousServerCertVerifier::new()))
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(tls_config));

        // TODO: remove hard code
        let domain = pki_types::ServerName::try_from("localhost")
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid host"))?
            .to_owned();

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

#[derive(Debug)]
struct DangerousServerCertVerifier {}

impl DangerousServerCertVerifier {
    fn new() -> Self {
        Self {}
    }
}

impl ServerCertVerifier for DangerousServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &pki_types::CertificateDer<'_>,
        _intermediates: &[pki_types::CertificateDer<'_>],
        _server_name: &pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::prelude::v1::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
    {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}
