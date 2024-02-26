use crate::common::Result;
use crate::protocol::message::{MessageFrames, MessageType, Parse};
use crate::protocol::{Key, Value};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Set {
    pub(crate) key: Key,
    pub(crate) value: Value,
}

impl Set {
    pub(crate) fn new(key: Key, value: Value) -> Self {
        Self { key, value }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Self> {
        let key = Key::new(parse.next_string()?)?;
        let value = Value::new(parse.next_bytes()?)?;

        Ok(Set::new(key, value))
    }
}
impl From<Set> for MessageFrames {
    fn from(set: Set) -> Self {
        let mut frames = MessageFrames::with_capacity(MessageType::Set, 2);

        frames.push_string(set.key.into_string());
        frames.push_bytes(set.value.into_boxed_bytes());

        frames
    }
}
