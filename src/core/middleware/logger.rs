use async_trait::async_trait;

use crate::common::{info, Result};
use crate::core::middleware::Middleware;
use crate::core::UnitOfWork;

pub(crate) struct Logger<MW> {
    next: MW,
}

impl<MW> Logger<MW> {
    pub(crate) fn new(next: MW) -> Self {
        Self { next }
    }
}

#[async_trait]
impl<MW> Middleware for Logger<MW>
where
    MW: Middleware + Send + 'static,
{
    async fn apply(&mut self, uow: UnitOfWork) -> Result<()> {

        let start = tokio::time::Instant::now();
        let log = format!("{:?}", uow);

        let result = self.next.apply(uow).await;

        info!(uow=%log, elapsed=?start.elapsed(),?result ,"Uow done");

        result
    }
}
