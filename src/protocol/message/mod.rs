mod authentication;
pub(crate) use authentication::Authenticate;

mod ping;
pub(crate) use ping::Ping;

mod message;
pub(crate) use message::{Message, MessageType, Success};

mod frame;
pub(crate) use frame::{frameprefix, Error as FrameError, Frame, MessageFrames};

mod parse;
pub(crate) use parse::{Parse, ParseError};

pub(crate) const DELIMITER: &[u8] = b"\r\n";
