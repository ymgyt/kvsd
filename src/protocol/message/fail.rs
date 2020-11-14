use crate::common::Result;
use crate::protocol::message::{MessageFrames, MessageType, Parse};

const UNDEFINED: &str = "UNDEFINED";
const UNAUTHENTICATED: &str = "UNAUTHENTICATED";
const UNEXPECTED_MESSAGE: &str = "UNEXPECTED_MESSAGE";

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum FailCode {
    Undefined,
    Unauthenticated,
    UnexpectedMessage,
}

impl FailCode {
    fn to_string(&self) -> String {
        match self {
            FailCode::Undefined => UNDEFINED.into(),
            FailCode::Unauthenticated => UNAUTHENTICATED.into(),
            FailCode::UnexpectedMessage => UNEXPECTED_MESSAGE.into(),
        }
    }
}

impl From<String> for FailCode {
    fn from(s: String) -> Self {
        match s.as_str() {
            UNAUTHENTICATED => FailCode::Unauthenticated,
            UNEXPECTED_MESSAGE => FailCode::UnexpectedMessage,
            _ => FailCode::Undefined,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Fail {
    code: FailCode,
    message: String,
}

impl Fail {
    pub(crate) fn new(code: FailCode) -> Self {
        Self {
            code,
            message: "".into(),
        }
    }

    pub(crate) fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Self> {
        let code = parse.next_string()?;
        let message = parse.next_string()?;

        Ok(Fail::new(FailCode::from(code)).with_message(message))
    }
}

impl Into<MessageFrames> for Fail {
    fn into(self) -> MessageFrames {
        let mut frames = MessageFrames::with_capacity(MessageType::Fail, 1);

        frames.push_string(self.code.to_string());
        frames.push_string(self.message);

        frames
    }
}

impl From<FailCode> for Fail {
    fn from(code: FailCode) -> Self {
        Fail::new(code)
    }
}
