use crate::protocol::message::{MessageFrames, MessageType};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Success {}

impl Success {
    pub(crate) fn new() -> Success {
        Self {}
    }
}

impl Into<MessageFrames> for Success {
    fn into(self) -> MessageFrames {
        MessageFrames::with_capacity(MessageType::Success, 0)
    }
}
