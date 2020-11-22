mod set;
pub(crate) use self::set::Set;

mod get;
pub(crate) use self::get::Get;

use std::fmt;
use std::sync::Arc;

use tokio::sync::oneshot;

use crate::common::{Result, Time};
use crate::core::{credential, Principal};
use crate::protocol::Value;

pub(crate) enum UnitOfWork {
    Authenticate(Work<Box<dyn credential::Provider + Send>, Option<Principal>>),
    Ping(Work<(), Time>),
    Set(Work<Set, Option<Value>>),
    Get(Work<Get, Option<Value>>),
}

pub(crate) struct Work<Req, Res> {
    pub(crate) principal: Arc<Principal>,
    pub(crate) request: Req,
    // Wrap with option so that response can be sent via mut reference.
    pub(crate) response_sender: Option<oneshot::Sender<Result<Res>>>,
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
                response_sender: Some(tx),
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
                response_sender: Some(tx),
            }),
            rx,
        )
    }

    pub(crate) fn new_set(
        principal: Arc<Principal>,
        set: Set,
    ) -> (UnitOfWork, oneshot::Receiver<Result<Option<Value>>>) {
        let (tx, rx) = oneshot::channel();
        (
            UnitOfWork::Set(Work {
                principal,
                request: set,
                response_sender: Some(tx),
            }),
            rx,
        )
    }

    pub(crate) fn new_get(
        principal: Arc<Principal>,
        get: Get,
    ) -> (UnitOfWork, oneshot::Receiver<Result<Option<Value>>>) {
        let (tx, rx) = oneshot::channel();
        (
            UnitOfWork::Get(Work {
                principal,
                request: get,
                response_sender: Some(tx),
            }),
            rx,
        )
    }
}

impl fmt::Debug for UnitOfWork {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnitOfWork::Authenticate(_) => {
                write!(f, "Authenticate")
            }
            UnitOfWork::Ping(_) => {
                write!(f, "Ping")
            }
            UnitOfWork::Set(set) => {
                write!(f, "{}", set.request)
            }
            UnitOfWork::Get(get) => {
                write!(f, "{}", get.request)
            }
        }
    }
}
