use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::common::{error, info, Result};
use crate::core::middleware::MiddlewareChain;
use crate::core::{Config, UnitOfWork};

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

        let mw = MiddlewareChain::new(&self.config.unwrap_or_default());

        Ok(Kvs {
            request_send: send,
            request_recv: recv,
            middlewares: mw,
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
    request_recv: Receiver<UnitOfWork>,
    request_send: Sender<UnitOfWork>,
    middlewares: MiddlewareChain,
}

impl Kvs {
    pub fn request_channel(&self) -> Sender<UnitOfWork> {
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

    pub(crate) async fn handle_request(&mut self, uow: UnitOfWork) -> Result<()> {
        return self.middlewares.apply(uow).await;
    }
}
