use async_trait::async_trait;

use crate::common::Result;
use crate::core::middleware::Middleware;
use crate::core::UnitOfWork;

pub(crate) struct Authorizer<MW> {
    next: MW,
}

impl<MW> Authorizer<MW> {
    pub(crate) fn new(next: MW) -> Self {
        Self { next }
    }
}

#[async_trait]
impl<MW> Middleware for Authorizer<MW>
where
    MW: Middleware + Send + 'static,
{
    async fn apply(&mut self, uow: UnitOfWork) -> Result<()> {
        self.next.apply(uow).await
    }
}
