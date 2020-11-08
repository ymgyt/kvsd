use std::sync::Arc;

use tokio::sync::oneshot;

use crate::common::{Result, Time};
use crate::core::{credential, Principal};

pub(crate) enum UnitOfWork {
    Authenticate(Work<Box<dyn credential::Provider + Send>, Result<Option<Principal>>>),
    Ping(Work<(), Result<Time>>),
}

pub(crate) struct Work<Req, Res> {
    pub(crate) principal: Arc<Principal>,
    pub(crate) request: Req,
    pub(crate) response_sender: oneshot::Sender<Res>,
}

impl UnitOfWork {
    pub(crate) fn new_authenticate(
        principal: Arc<Principal>,
        provider: impl credential::Provider + Send + 'static,
    ) -> (UnitOfWork, oneshot::Receiver<Result<Option<Principal>>>) {
        let (tx, rx) = oneshot::channel();
        (
            UnitOfWork::Authenticate(Work {
                principal,
                request: Box::new(provider),
                response_sender: tx,
            }),
            rx,
        )
    }

    pub(crate) fn new_ping(
        principal: Arc<Principal>,
    ) -> (UnitOfWork, oneshot::Receiver<Result<Time>>) {
        let (tx, rx) = oneshot::channel();
        (
            UnitOfWork::Ping(Work {
                principal,
                request: (),
                response_sender: tx,
            }),
            rx,
        )
    }
}
