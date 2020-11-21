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

impl Into<MessageFrames> for Set {
    fn into(self) -> MessageFrames {
        let mut frames = MessageFrames::with_capacity(MessageType::Set, 2);

        frames.push_string(self.key.into_string());
        frames.push_bytes(self.value.into_boxed_bytes());

        frames
    }
}
