use crate::common::Result;
use crate::protocol::message::{MessageFrames, MessageType, Parse};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Fail {
    message: String,
}

impl Fail {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Self> {
        let message = parse.next_string()?;

        Ok(Fail::new(message))
    }
}

impl Into<MessageFrames> for Fail {
    fn into(self) -> MessageFrames {
        let mut frames = MessageFrames::with_capacity(MessageType::Fail, 1);

        frames.push_string(self.message);

        frames
    }
}
