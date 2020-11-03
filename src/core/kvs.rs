use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::common::{error, info, ErrorKind, Result};
use crate::core::{Config, Request};

#[derive(Default)]
pub(crate) struct Builder {
    config: Option<Config>,
    request_channel_buffer: usize,
}

impl Builder {
    pub(crate) fn from_config(config: Config) -> Self {
        let mut builder = Builder::new();
        builder.config = Some(config);
        builder
    }

    pub(crate) fn build(self) -> Result<Kvs> {
        let (send, recv) = mpsc::channel(self.request_channel_buffer);
        Ok(Kvs {
            request_send: send,
            request_recv: recv,
        })
    }

    fn new() -> Self {
        Self {
            request_channel_buffer: 1024,
            ..Default::default()
        }
    }
}

pub(crate) struct Kvs {
    request_recv: Receiver<Request>,
    request_send: Sender<Request>,
}

impl Kvs {
    pub fn request_channel(&self) -> Sender<Request> {
        self.request_send.clone()
    }

    pub(crate) async fn run(mut self) {
        info!("Kvs running");

        while let Some(request) = self.request_recv.recv().await {
            // TODO: middleware, dispatcher
            if let Err(err) = self.handle_request(request).await {
                error!("Handle request {}", err);
            }
        }
    }

    pub(crate) async fn handle_request(&mut self, request: Request) -> Result<()> {
        match request {
            Request::Ping(ping) => {
                use chrono::Utc;
                // mock network latency.
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                ping.response_sender
                    .send(Utc::now())
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()))?;
            }
        }

        Ok(())
    }
}
