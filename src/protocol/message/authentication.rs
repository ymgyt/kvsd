use crate::protocol::message::{MessageFrames, MessageType};

// Authenticate is a message in which client requests the server
// to perform authentication process.
// TODO: impl custom debug for mask credentials.
#[derive(Debug)]
pub(crate) struct Authenticate {
    username: String,
    password: String,
}

impl Authenticate {
    pub(crate) fn new<S1, S2>(username: S1, password: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl Into<MessageFrames> for Authenticate {
    fn into(self) -> MessageFrames {
        let mut frames = MessageFrames::with_capacity(MessageType::Authenticate, 2);

        frames.push_string(self.username);
        frames.push_string(self.password);

        frames
    }
}