mod authentication;
pub(crate) use authentication::Authenticate;

mod ping;
pub(crate) use ping::Ping;

mod success;
pub(crate) use success::Success;

mod fail;
pub(crate) use fail::{Fail, FailCode};

mod message;
pub(crate) use message::{Message, MessageType};

mod frame;
pub(crate) use frame::{frameprefix, Error as FrameError, Frame, MessageFrames};

mod parse;
pub(crate) use parse::{Parse, ParseError};

mod set;
pub(crate) use set::Set;

pub(crate) const DELIMITER: &[u8] = b"\r\n";
