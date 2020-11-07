mod authentication;
pub(crate) use authentication::Authenticate;

mod ping;
pub(crate) use ping::Ping;

mod message;
pub(crate) use message::{Message, MessageType};

mod frame;
pub(crate) use frame::{frameprefix, Error as FrameError, Frame, MessageFrames};

pub(crate) const DELIMITER: &[u8] = b"\r\n";
