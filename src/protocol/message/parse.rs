use std::vec;

use crate::common::{self, ErrorKind, Result, Time};
use crate::protocol::message::{Frame, MessageFrames, MessageType};

pub(crate) struct Parse {
    frames: vec::IntoIter<Frame>,
}

pub(crate) enum ParseError {
    EndOfStream,
    Other(common::Error),
}

impl Parse {
    pub(crate) fn new(message_frames: MessageFrames) -> Self {
        Self {
            frames: message_frames.into_iter(),
        }
    }

    pub(crate) fn message_type(&mut self) -> Option<MessageType> {
        self.next().ok().and_then(|frame| match frame {
            Frame::MessageType(mt) => Some(mt),
            _ => None,
        })
    }

    pub(crate) fn next_string(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Frame::String(s) => Ok(s),
            frame => Err(format!("parse frame error; expected string, got {:?}", frame).into()),
        }
    }

    pub(crate) fn next_bytes(&mut self) -> Result<Vec<u8>, ParseError> {
        match self.next()? {
            Frame::Bytes(val) => Ok(val),
            frame => Err(format!("unexpected frame. want bytes got {:?}", frame).into()),
        }
    }

    pub(crate) fn next_time_or_null(&mut self) -> Result<Option<Time>, ParseError> {
        match self.next()? {
            Frame::Time(time) => Ok(Some(time)),
            Frame::Null => Ok(None),
            frame => Err(format!("unexpected frame. want (time|null) got {:?} ", frame).into()),
        }
    }

    fn next(&mut self) -> Result<Frame, ParseError> {
        self.frames.next().ok_or(ParseError::EndOfStream)
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> ParseError {
        ParseError::Other(ErrorKind::NetworkFraming(src).into())
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        src.to_string().into()
    }
}
