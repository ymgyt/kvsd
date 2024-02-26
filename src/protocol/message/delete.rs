use crate::common::Result;
use crate::protocol::message::{MessageFrames, MessageType, Parse};
use crate::protocol::Key;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Delete {
    pub(crate) key: Key,
}

impl Delete {
    pub(crate) fn new(key: Key) -> Self {
        Self { key }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Self> {
        let key = Key::new(parse.next_string()?)?;

        parse.expect_consumed()?;

        Ok(Delete::new(key))
    }
}

impl From<Delete> for MessageFrames {
    fn from(delete: Delete) -> Self {
        let mut frames = MessageFrames::with_capacity(MessageType::Delete, 1);

        frames.push_string(delete.key.into_string());

        frames
    }
}
