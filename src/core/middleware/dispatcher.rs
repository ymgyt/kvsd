use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::common::{info, ErrorKind, Result};
use crate::core::middleware::Middleware;
use crate::core::UnitOfWork;

pub(crate) struct Dispatcher {
    table: HashMap<String, HashMap<String, mpsc::Sender<UnitOfWork>>>,
}

impl Dispatcher {
    pub(crate) fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub(crate) fn add_table<S>(&mut self, namespace: S, table: S, sender: mpsc::Sender<UnitOfWork>)
    where
        S: Into<String>,
    {
        self.table
            .entry(namespace.into())
            .or_insert_with(HashMap::new)
            .insert(table.into(), sender);
    }

    fn lookup_table(&self, namespace: &str, table: &str) -> Result<&mpsc::Sender<UnitOfWork>> {
        self.table
            .get(namespace)
            .and_then(|tables| tables.get(table))
            .ok_or_else(|| ErrorKind::TableNotFound(format!("{}/{}", namespace, table)).into())
    }
}

#[async_trait]
impl Middleware for Dispatcher {
    async fn apply(&mut self, mut uow: UnitOfWork) -> Result<()> {
        match uow {
            UnitOfWork::Ping(ping) => {
                use chrono::Utc;
                // mock network latency.
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    100 + (rand::random::<f64>() * 100.0) as u64,
                ))
                .await;

                // TODO: handle unauthenticated error
                assert!(ping.principal.is_authenticated());
                info!(user=?ping.principal, "Ping");
                let response = if !ping.principal.is_authenticated() {
                    Err(ErrorKind::Unauthorized("unauthorized ping".to_owned()).into())
                } else {
                    Ok(Utc::now())
                };

                ping.response_sender
                    .expect("response already sent")
                    .send(response)
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()))?;

                Ok(())
            }
            UnitOfWork::Set(ref mut set) => {
                match self.lookup_table(&set.request.namespace, &set.request.table) {
                    Ok(sender) => Ok(sender.send(uow).await?),
                    Err(err) => set
                        .response_sender
                        .take()
                        .expect("response already sent")
                        .send(Err(err))
                        .map_err(|_| ErrorKind::Internal("send response".to_owned()).into()),
                }
            }
            UnitOfWork::Get(ref mut get) => {
                match self.lookup_table(&get.request.namespace, &get.request.table) {
                    Ok(sender) => Ok(sender.send(uow).await?),
                    Err(err) => get
                        .response_sender
                        .take()
                        .expect("response already sent")
                        .send(Err(err))
                        .map_err(|_| ErrorKind::Internal("send response".to_owned()).into()),
                }
            }
            _ => unreachable!(),
        }
    }
}
