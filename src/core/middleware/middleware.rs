use async_trait::async_trait;

use crate::common::Result;
use crate::core::UnitOfWork;

#[async_trait]
pub(crate) trait Middleware {
    async fn apply(&mut self, uow: UnitOfWork) -> Result<()>;
}
