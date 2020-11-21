use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::common::{error, info, Result, debug};
use crate::config::filepath;
use crate::core::middleware::{Dispatcher, MiddlewareChain};
use crate::core::table::Table;
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

    pub(crate) async fn build(mut self) -> Result<Kvs> {
        let (send, recv) = mpsc::channel(self.request_channel_buffer);

        let dispatcher = self.build_dispatcher().await?;

        let mw = MiddlewareChain::new(&self.config.unwrap_or_default(), dispatcher);

        Ok(Kvs {
            request_send: send,
            request_recv: recv,
            middlewares: mw,
        })
    }

    async fn build_dispatcher(&mut self) -> Result<Dispatcher> {
        // TODO configure channel size
        let (tx, rx) = mpsc::channel(1024);

        let root_dir = self.config.as_ref().unwrap().root_dir.as_ref().unwrap();
        let default_table = root_dir
            .join(filepath::NAMESPACES)
            .join(filepath::NS_DEFAULT)
            .join("default/default.kvs");
        debug!("Open default table file {}", default_table.display());
        let default_table = Table::from_path(rx, default_table).await?;

        tokio::spawn(default_table.run());

        let mut dispatcher = Dispatcher::new();
        dispatcher.add_table("default", "default", tx);

        Ok(dispatcher)
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
