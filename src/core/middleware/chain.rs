use crate::common::Result;
use crate::core::middleware::{Authenticator, Authorizer, Dispatcher, Logger, Middleware};
use crate::core::{Config, UnitOfWork};

pub(crate) struct MiddlewareChain {
    root: Logger<Authenticator<Authorizer<Dispatcher>>>,
}

impl MiddlewareChain {
    pub(crate) fn new(config: &Config) -> Self {
        let dispatcher = Dispatcher::new();

        let authorizer = Authorizer::new(dispatcher);

        let authenticator = Authenticator::new(config.users.clone(), authorizer);

        let logger = Logger::new(authenticator);

        Self { root: logger }
    }

    pub(crate) async fn apply(&mut self, uow: UnitOfWork) -> Result<()> {
        self.root.apply(uow).await
    }
}
