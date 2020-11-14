use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{timeout, Duration};

use crate::common::{error, info, trace, warn, Result};
use crate::core::{Principal, UnitOfWork};
use crate::protocol::connection::Connection;
use crate::protocol::message::{Fail, FailCode, Message, Success};

// Server configuration.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct Config {
    // Max tcp connections.
    max_tcp_connections: Option<u32>,
    // Size of buffer allocated per tcp connection.
    connection_tcp_buffer_bytes: Option<usize>,
    // Timeout duration for reading authenticate message.
    authenticate_timeout_milliseconds: Option<u64>,
    // tcp listen host.
    listen_host: Option<String>,
    // tcp listen port.
    listen_port: Option<String>,
}

impl Config {
    const DEFAULT_MAX_TCP_CONNECTIONS: u32 = 1024 * 10;
    const DEFAULT_CONNECTION_TCP_BUFFER_BYTES: usize = 1024 * 4;
    const DEFAULT_AUTHENTICATE_TIMEOUT_MILLISECONDS: u64 = 300;
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
    pub(crate) fn set_authenticate_timeout_milliseconds(&mut self, val: Option<u64>) {
        if let Some(val) = val {
            self.authenticate_timeout_milliseconds = Some(std::cmp::max(val, 10));
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
        self.set_authenticate_timeout_milliseconds(other.authenticate_timeout_milliseconds);
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

    fn authenticate_timeout(&self) -> Duration {
        Duration::from_millis(
            self.authenticate_timeout_milliseconds
                .unwrap_or(Config::DEFAULT_AUTHENTICATE_TIMEOUT_MILLISECONDS),
        )
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

type ShutdownSignal = ();
type ShutdownCompleteSignal = ();

// Handle graceful shutdown.
struct GracefulShutdown {
    notify_shutdown: broadcast::Sender<ShutdownSignal>,
    shutdown_complete_tx: mpsc::Sender<ShutdownCompleteSignal>,
    shutdown_complete_rx: mpsc::Receiver<ShutdownCompleteSignal>,
}

impl GracefulShutdown {
    fn new() -> Self {
        let (notify_shutdown, _) = broadcast::channel(1);
        let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

        Self {
            notify_shutdown,
            shutdown_complete_tx,
            shutdown_complete_rx,
        }
    }

    // Notify handlers of the shutdown and wait for it to be completed.
    async fn shutdown(mut self) {
        // Notify shutdown to all handler.
        drop(self.notify_shutdown);

        // Drop final Sender so the Receiver below can complete.
        drop(self.shutdown_complete_tx);

        // Wait for all handler to finish.
        let _ = self.shutdown_complete_rx.recv().await;
    }
}

pub(crate) struct Server {
    config: Config,
    graceful_shutdown: GracefulShutdown,
}

impl Server {
    // Construct Server from config.
    pub(crate) fn new(config: Config) -> Self {
        Self {
            config,
            graceful_shutdown: GracefulShutdown::new(),
        }
    }

    // Utility serve wrapper for handle systemcalls.
    pub(crate) async fn run(mut self, request_sender: mpsc::Sender<UnitOfWork>) -> Result<()> {
        let addr = self.config.listen_addr();
        info!(%addr, "Listening...");

        let listener = tokio::net::TcpListener::bind(addr).await?;

        tokio::select! {
            result = self.serve(listener, request_sender) => {
                if let Err(err) = result {
                    error!(cause = %err, "Failed to accept");
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received");
            }
        }

        info!("Notify shutdown to all handlers");

        self.graceful_shutdown.shutdown().await;

        info!("Shutdown successfully completed");

        Ok(())
    }

    pub(crate) async fn serve(
        &mut self,
        listener: TcpListener,
        request_sender: mpsc::Sender<UnitOfWork>,
    ) -> Result<()> {
        info!("Server running. {:?}", self.config);

        let mut listener = SemaphoreListener::new(listener, self.config.max_tcp_connections());

        loop {
            let (socket, addr) = listener.accept().await?;
            info!(
                available = listener.max_connections.available_permits(),
                "Connection accepted"
            );

            let connection =
                Connection::new(socket, Some(self.config.connection_tcp_buffer_bytes()));

            let handler = Handler::new(
                connection,
                request_sender.clone(),
                ShutdownSubscriber::new(
                    self.graceful_shutdown.notify_shutdown.subscribe(),
                    self.graceful_shutdown.shutdown_complete_tx.clone(),
                ),
                listener.max_connections.clone(),
                self.config.authenticate_timeout(),
            )
            .with_socket_addr(addr);

            tokio::spawn(handler.handle());
        }
    }
}

struct Handler {
    principal: Arc<Principal>,
    remote_addr: Option<std::net::SocketAddr>,
    connection: Connection,
    request_sender: mpsc::Sender<UnitOfWork>,
    shutdown: ShutdownSubscriber,
    max_connections: Arc<Semaphore>,
    authenticate_timeout: Duration,
}

impl Handler {
    fn new(
        connection: Connection,
        request_sender: mpsc::Sender<UnitOfWork>,
        shutdown: ShutdownSubscriber,
        max_connections: Arc<Semaphore>,
        authenticate_timeout: Duration,
    ) -> Self {
        Self {
            principal: Arc::new(Principal::AnonymousUser),
            connection,
            remote_addr: None,
            request_sender,
            shutdown,
            max_connections,
            authenticate_timeout,
        }
    }

    fn with_socket_addr(mut self, addr: std::net::SocketAddr) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    async fn handle(mut self) {
        match self.authenticate().await {
            // Successfully client authenticated.
            Ok(true) => {
                if let Err(err) = self.handle_message().await {
                    error!(?self.remote_addr, "Handle message {}", err);
                }
            }
            // Authentication failed, do nothing, just close connection.
            Ok(false) => (),
            Err(err) if err.is_timeout() => warn!("authentication timeout {}", err),
            Err(err) => error!("{}", err),
        }
    }

    async fn authenticate(&mut self) -> Result<bool> {
        match self
            .connection
            .read_message_with_timeout(self.authenticate_timeout)
            .await?
        {
            Some(Message::Authenticate(auth)) => {
                let (work, rx) = UnitOfWork::new_authenticate(self.principal.clone(), auth.clone());

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
                            .write_message(Fail::from(FailCode::Unauthenticated))
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
        }
    }

    async fn handle_message(&mut self) -> Result<()> {
        // select! can't detect shutdown reliably, so explicitly check shutdown before tcp read.
        while !self.shutdown.is_shutdown() {
            let maybe_message = tokio::select! {
                msg = self.connection.read_message() => msg?,
                _ = self.shutdown.recv() => {
                    return Ok(())
                }
            };

            let message = match maybe_message {
                Some(message) => message,
                // peer closed the socket.
                None => return Ok(()),
            };

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
                                .write_message(Fail::new(FailCode::Unauthenticated))
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

        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.max_connections.add_permits(1);
    }
}

struct SemaphoreListener {
    inner: TcpListener,
    max_connections: Arc<Semaphore>,
}

impl SemaphoreListener {
    fn new(listener: TcpListener, max_connections: u32) -> Self {
        Self {
            inner: listener,
            max_connections: Arc::new(Semaphore::new(max_connections as usize)),
        }
    }

    async fn accept(&mut self) -> std::io::Result<(TcpStream, std::net::SocketAddr)> {
        self.max_connections.acquire().await.forget();
        self.inner.accept().await
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

struct ShutdownSubscriber {
    shutdown: bool,
    notify: broadcast::Receiver<ShutdownSignal>,
    // Notify completing shutdown process by dropping.
    _complete_tx: mpsc::Sender<ShutdownCompleteSignal>,
}

impl ShutdownSubscriber {
    fn new(
        notify: broadcast::Receiver<ShutdownSignal>,
        complete_tx: mpsc::Sender<ShutdownCompleteSignal>,
    ) -> Self {
        Self {
            shutdown: false,
            notify,
            _complete_tx: complete_tx,
        }
    }

    fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    async fn recv(&mut self) {
        if self.shutdown {
            return;
        }

        match self.notify.recv().await {
            Ok(_) | Err(broadcast::error::RecvError::Closed) => (), // ok
            Err(err) => error!("shutdown notify receive error {}", err),
        }

        self.shutdown = true;
    }
}
