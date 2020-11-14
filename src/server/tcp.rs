use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use crate::common::{error, info, trace, warn, Result};
use crate::core::{Principal, UnitOfWork};
use crate::protocol::connection::Connection;
use crate::protocol::message::{Fail, Message, Success};

// Server configuration.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct Config {
    // Max tcp connections.
    max_tcp_connections: Option<u32>,
    // Size of buffer allocated per tcp connection.
    connection_tcp_buffer_bytes: Option<usize>,
    // tcp listen host.
    listen_host: Option<String>,
    // tcp listen port.
    listen_port: Option<String>,
}

impl Config {
    const DEFAULT_MAX_TCP_CONNECTIONS: u32 = 1024 * 10;
    const DEFAULT_CONNECTION_TCP_BUFFER_BYTES: usize = 1024 * 4;
    const DEFAULT_LISTEN_HOST: &'static str = "127.0.0.1";
    const DEFAULT_LISTEN_PORT: &'static str = crate::server::DEFAULT_PORT;

    pub(crate) fn set_max_tcp_connections(&mut self, val: Option<u32>) {
        if let Some(val) = val {
            self.max_tcp_connections = Some(std::cmp::max(val, 1));
        }
    }
    pub(crate) fn set_connection_tcp_buffer_bytes(&mut self, val: Option<usize>) {
        if let Some(val) = val {
            self.connection_tcp_buffer_bytes = Some(std::cmp::max(val, 1));
        }
    }
    pub(crate) fn set_listen_host(&mut self, val: &mut Option<String>) {
        if let Some(val) = val.take() {
            self.listen_host = Some(val)
        }
    }
    pub(crate) fn set_listen_port(&mut self, val: &mut Option<String>) {
        if let Some(val) = val.take() {
            self.listen_port = Some(val)
        }
    }
    pub(crate) fn override_merge(&mut self, other: &mut Config) {
        self.set_max_tcp_connections(other.max_tcp_connections);
        self.set_connection_tcp_buffer_bytes(other.connection_tcp_buffer_bytes);
        self.set_listen_host(&mut other.listen_host);
        self.set_listen_port(&mut other.listen_port);
    }

    fn max_tcp_connections(&self) -> u32 {
        match self.max_tcp_connections {
            Some(val) => val,
            None => Config::DEFAULT_MAX_TCP_CONNECTIONS,
        }
    }

    fn connection_tcp_buffer_bytes(&self) -> usize {
        match self.connection_tcp_buffer_bytes {
            Some(val) => val,
            None => Config::DEFAULT_CONNECTION_TCP_BUFFER_BYTES,
        }
    }
    fn listen_addr(&self) -> String {
        format!(
            "{}:{}",
            self.listen_host
                .as_deref()
                .unwrap_or_else(|| Config::DEFAULT_LISTEN_HOST),
            self.listen_port
                .as_deref()
                .unwrap_or_else(|| Config::DEFAULT_LISTEN_PORT),
        )
    }
}

pub(crate) struct Server {
    config: Config,
}

impl Server {
    // Construct Server from config.
    pub(crate) fn new(config: Config) -> Self {
        Self { config }
    }

    pub(crate) async fn run(self, request_sender: mpsc::Sender<UnitOfWork>) -> Result<()> {
        let addr = self.config.listen_addr();
        info!(%addr, "Listening...");

        let listener = tokio::net::TcpListener::bind(addr).await?;

        self.serve(listener, request_sender).await
    }

    async fn serve(
        self,
        listener: TcpListener,
        request_sender: mpsc::Sender<UnitOfWork>,
    ) -> Result<()> {
        info!("Server running. {:?}", self.config);

        let mut listener = MaxConnAwareListener::new(listener, self.config.max_tcp_connections());

        loop {
            let (socket, addr, done) = listener.accept().await?;

            let connection =
                Connection::new(socket, Some(self.config.connection_tcp_buffer_bytes()));
            let handler =
                Handler::new(connection, done, request_sender.clone()).with_socket_addr(addr);

            tokio::spawn(handler.handle());
        }
    }
}

struct Handler {
    principal: Arc<Principal>,
    connection: Connection,
    done: mpsc::Sender<()>,
    request_sender: mpsc::Sender<UnitOfWork>,
    remote_addr: Option<std::net::SocketAddr>,
}

impl Handler {
    fn new(
        connection: Connection,
        done: mpsc::Sender<()>,
        request_sender: mpsc::Sender<UnitOfWork>,
    ) -> Self {
        Self {
            principal: Arc::new(Principal::AnonymousUser),
            connection,
            done,
            request_sender,
            remote_addr: None,
        }
    }

    fn with_socket_addr(mut self, addr: std::net::SocketAddr) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    async fn handle(mut self) {
        match self.authenticate().await {
            Ok(true) => {
                if let Err(err) = self.handle_message().await {
                    error!(?self.remote_addr, "Handle message {}", err);
                }
            }
            Ok(false) => (),
            Err(err) => error!("{}", err),
        }
        self.cleanup().await;
    }

    async fn authenticate(&mut self) -> Result<bool> {
        match timeout(Duration::from_millis(500), self.connection.read_message()).await {
            Ok(result) => match result {
                Ok(message) => match message {
                    Some(Message::Authenticate(auth)) => {
                        let (work, rx) =
                            UnitOfWork::new_authenticate(self.principal.clone(), auth.clone());

                        self.request_sender.send(work).await?;

                        let auth_result = rx.await??;
                        match auth_result {
                            Some(principal) => {
                                self.principal = Arc::new(principal);
                                self.connection.write_message(Success::new()).await?;
                                Ok(true)
                            }
                            None => {
                                info!(addr=?self.remote_addr, "unauthenticated {:?}", auth);
                                self.connection
                                    .write_message(Fail::new("unauthenticated"))
                                    .await?;
                                Ok(false)
                            }
                        }
                    }
                    Some(msg) => {
                        warn!("unexpected message {:?}", msg);
                        Ok(false)
                    }
                    None => Ok(false),
                },
                Err(err) => Err(err),
            },
            // read timeout
            Err(elapsed) => {
                warn!("authenticate timeout({})", elapsed);
                Ok(false)
            }
        }
    }

    async fn handle_message(&mut self) -> Result<()> {
        while let Some(message) = self.connection.read_message().await? {
            info!(?message, "Handle message");
            match message {
                Message::Ping(mut ping) => {
                    let (work, rx) = UnitOfWork::new_ping(self.principal.clone());

                    self.request_sender.send(work).await?;

                    let ping_result = rx.await?;
                    match ping_result {
                        Ok(time) => {
                            ping.record_server_time(time);
                            self.connection.write_message(ping).await?;
                        }
                        Err(err) if err.is_unauthorized() => {
                            self.connection
                                .write_message(Fail::new("unauthorized ping"))
                                .await?;
                        }
                        _ => unreachable!(),
                    }
                }
                Message::Authenticate(_) => unreachable!(),
                Message::Success(_) => unreachable!(),
                Message::Fail(_) => unreachable!(),
            }
        }
        trace!("Handle message complete");
        Ok(())
    }

    async fn cleanup(self) {
        if let Err(err) = self.done.send(()).await {
            error!("send completion signal error {}", err);
        }
    }
}

// Tcp listener to manage connection limits.
struct MaxConnAwareListener {
    inner: TcpListener,
    max_connections: u32,
    current_connections: Arc<AtomicU32>,
    // Signal handler completion.
    sender: mpsc::Sender<()>,
}

impl MaxConnAwareListener {
    // Construct and dispatch handler done watcher.
    fn new(listener: TcpListener, max_connections: u32) -> Self {
        let (tx, mut rx) = mpsc::channel(1024);
        let current_connections = Arc::new(AtomicU32::new(0));

        // Watch channel to which handlers notify completion and update the current connections.
        let current_connection_clone = Arc::clone(&current_connections);
        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                let prev = current_connection_clone.fetch_sub(1, Ordering::Relaxed);
                trace!(curren_conn = prev - 1, "Work done signal received.");
            }
        });

        Self {
            inner: listener,
            max_connections,
            current_connections,
            sender: tx,
        }
    }

    async fn accept(
        &mut self,
    ) -> std::io::Result<(TcpStream, std::net::SocketAddr, mpsc::Sender<()>)> {
        loop {
            let mut stream = self.inner.accept().await?;
            let current_conns = self.current_connections.load(Ordering::Relaxed) + 1;
            info!(
                addr = %stream.1, "Accept connection ({}/{})",
                current_conns, self.max_connections
            );

            if current_conns <= self.max_connections {
                self.current_connections.fetch_add(1, Ordering::Relaxed);
                return Ok((stream.0, stream.1, self.sender.clone()));
            }

            warn!(
                "Current connections reach max connections({}/{})",
                current_conns, self.max_connections
            );
            // Notify client that connection has reached max with timeout.
            if let Err(err) = timeout(
                Duration::from_millis(100),
                stream.0.write_all(b"Reach max connections"),
            )
            .await
            {
                warn!(%err, "Write max conn message.");
            }
        }
    }
}
