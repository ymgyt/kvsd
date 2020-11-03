use chrono::{DateTime, Utc};
use tokio::sync::oneshot;

pub enum Request {
    Ping(Base<PingRequest, DateTime<Utc>>),
}

pub struct Base<Req, Res> {
    pub(super) response_sender: oneshot::Sender<Res>,
    pub(super) request: Req,
}

pub struct PingRequest {}

impl PingRequest {
    pub fn new_request() -> (oneshot::Receiver<DateTime<Utc>>, Request) {
        let (tx, rx) = oneshot::channel();
        let base = Base {
            response_sender: tx,
            request: PingRequest {},
        };

        (rx, Request::Ping(base))
    }
}
