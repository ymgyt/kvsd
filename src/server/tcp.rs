use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use crate::common::{error, info, trace, warn, Result};

// Server configuration.
#[derive(Debug)]
pub(crate) struct Config {
    max_tcp_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_tcp_connections: 1024 * 10,
        }
    }
}

impl Config {
    pub(crate) fn set_max_tcp_connections(mut self, n: Option<u32>) -> Self {
        if let Some(n) = n {
            self.max_tcp_connections = std::cmp::max(n, 1);
        }
        self
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

    pub(crate) async fn run(self, listener: TcpListener) -> Result<()> {
        info!("Server running. {:?}", self.config);

        let mut listener = MaxConnAwareListener::new(listener, self.config.max_tcp_connections);

        loop {
            let (socket, _, done) = listener.accept().await?;
            let handler = Handler::new(socket, done);
            tokio::spawn(handler.handle());
        }
    }
}

struct Handler {
    stream: TcpStream,
    done: mpsc::Sender<()>,
}

impl Handler {
    fn new(stream: TcpStream, done: mpsc::Sender<()>) -> Self {
        Self { stream, done }
    }

    async fn handle(mut self) {
        if let Err(err) = self.echo().await {
            error!("echo error {}", err);
        }
        if let Err(err) = self.done.send(()).await {
            error!("send completion signal error {}", err);
        }
    }

    async fn echo(&mut self) -> Result<()> {
        let stream = &mut self.stream;
        let mut buff = [0u8; 1024];
        loop {
            match stream.read(&mut buff).await {
                Ok(0) => return Ok(()),
                Ok(n) => stream.write_all(&buff[..n]).await?,
                Err(err) => return Err(err.into()),
            }
        }
    }
}

// Tcp listener to manage connection limits.
struct MaxConnAwareListener {
    inner: TcpListener,
    max_connections: u32,
    current_connections: Arc<AtomicU32>,
    // Signal worker completion.
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
                trace!("Work done signal received");
                current_connection_clone.fetch_sub(1, Ordering::Relaxed);
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
                "Accept from {} ({}/{})",
                stream.1, current_conns, self.max_connections
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
                warn!("Write max conn message {}", err);
            }
        }
    }
}
