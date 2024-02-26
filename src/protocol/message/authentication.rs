use crate::common::Result;
use crate::core::{Credential, CredentialProvider, Password};
use crate::protocol::message::{MessageFrames, MessageType, Parse};

// Authenticate is a message in which client requests the server
// to perform authentication process.
// TODO: impl custom debug for mask credentials.
#[derive(Debug, Clone, PartialEq)]
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

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Self> {
        let username = parse.next_string()?;
        let password = parse.next_string()?;

        Ok(Authenticate::new(username, password))
    }
}

impl From<Authenticate> for MessageFrames {
    fn from(auth: Authenticate) -> Self {
        let mut frames = MessageFrames::with_capacity(MessageType::Authenticate, 2);

        frames.push_string(auth.username);
        frames.push_string(auth.password);

        frames
    }
}

impl CredentialProvider for Authenticate {
    fn credential(&self) -> Credential {
        Credential::Password(Password {
            username: std::borrow::Cow::Borrowed(&self.username),
            password: std::borrow::Cow::Borrowed(&self.password),
        })
    }
}
