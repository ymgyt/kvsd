use async_trait::async_trait;

use crate::common::{info, ErrorKind, Result};
use crate::core::middleware::Middleware;
use crate::core::UnitOfWork;

pub(crate) struct Dispatcher {}

impl Dispatcher {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Middleware for Dispatcher {
    async fn apply(&mut self, uow: UnitOfWork) -> Result<()> {
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
                    .send(response)
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()))?;

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
