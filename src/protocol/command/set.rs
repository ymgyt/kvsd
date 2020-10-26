use crate::protocol::command::Key;
use bytes::Bytes;

pub struct Set {
    key: Key,
    value: Bytes,
}
