use bytes::Bytes;
use tokio::sync::oneshot;

use crate::protocol::command::{Key, CommandError};

#[derive(Debug)]
pub struct Set {
    pub key: Key,
    pub value: Bytes,
    pub result_sender: oneshot::Sender<Result<(), CommandError>>,
}
