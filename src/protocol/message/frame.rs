use std::convert::TryFrom;

use bytes::Buf;

use crate::common::{self, Time};
use crate::protocol::message::{MessageType, DELIMITER};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Frame {
    MessageType(MessageType),
    String(String),
    Bytes(Vec<u8>),
    Time(Time),
    Null,
}

pub(crate) mod frameprefix {
    pub(crate) const MESSAGE_FRAMES: u8 = b'*';
    pub(crate) const MESSAGE_TYPE: u8 = b'#';
    pub(crate) const STRING: u8 = b'+';
    pub(crate) const BYTES: u8 = b'$';
    pub(crate) const TIME: u8 = b'T';
    pub(crate) const NULL: u8 = b'|';
}

#[derive(Debug)]
pub(crate) enum Error {
    /// Not enough data is available to decode a message frames from buffer.
    Incomplete,
    Invalid(String),
    Other(common::Error),
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct MessageFrames(Vec<Frame>);

type ByteCursor<'a> = std::io::Cursor<&'a [u8]>;

impl MessageFrames {
    pub(crate) fn with_capacity(mt: MessageType, n: usize) -> Self {
        let mut v = Vec::with_capacity(n + 1);
        v.push(Frame::MessageType(mt));
        Self(v)
    }

    pub(crate) fn push_string(&mut self, s: impl Into<String>) {
        self.0.push(Frame::String(s.into()))
    }

    pub(crate) fn push_bytes(&mut self, bytes: impl Into<Vec<u8>>) {
        self.0.push(Frame::Bytes(bytes.into()));
    }

    pub(crate) fn push_time(&mut self, time: Time) {
        self.0.push(Frame::Time(time));
    }
    pub(crate) fn push_time_or_null(&mut self, time: Option<Time>) {
        match time {
            Some(t) => self.push_time(t),
            None => self.push_null(),
        }
    }
    pub(crate) fn push_null(&mut self) {
        self.0.push(Frame::Null);
    }

    pub(crate) fn len(&self) -> u64 {
        self.0.iter().fold(0, |acc, frame| acc + frame.len())
    }

    pub(crate) fn check_parse(src: &mut ByteCursor) -> Result<(), Error> {
        let frames_len = MessageFrames::ensure_prefix_format(src)?;

        for _ in 0..frames_len {
            Frame::check(src)?;
        }

        Ok(())
    }

    pub(crate) fn parse(src: &mut ByteCursor) -> Result<MessageFrames, Error> {
        let frames_len = (MessageFrames::ensure_prefix_format(src)? - 1) as usize;

        if cursor::get_u8(src)? != frameprefix::MESSAGE_TYPE {
            return Err(Error::Invalid("message type expected".into()));
        }
        let message_type = cursor::get_u8(src)?;
        let message_type = MessageType::try_from(message_type).map_err(Error::Other)?;

        let mut frames = MessageFrames::with_capacity(message_type, frames_len);

        for _ in 0..frames_len {
            frames.0.push(Frame::parse(src)?);
        }

        Ok(frames)
    }

    fn ensure_prefix_format(src: &mut ByteCursor) -> Result<u64, Error> {
        if cursor::get_u8(src)? != frameprefix::MESSAGE_FRAMES {
            return Err(Error::Invalid("message frames prefix expected".into()));
        }

        cursor::get_decimal(src)
    }
}

impl Frame {
    fn len(&self) -> u64 {
        // impl when Frame::Array added
        1
    }
    fn check(src: &mut ByteCursor) -> Result<(), Error> {
        match cursor::get_u8(src)? {
            frameprefix::MESSAGE_TYPE => {
                cursor::get_u8(src)?;
                Ok(())
            }
            frameprefix::STRING => {
                cursor::get_line(src)?;
                Ok(())
            }
            frameprefix::BYTES => {
                let len = cursor::get_decimal(src)? as usize;
                // skip bytes length + delimiter
                cursor::skip(src, len + 2)
            }
            frameprefix::TIME => {
                cursor::get_line(src)?;
                Ok(())
            }
            frameprefix::NULL => Ok(()),
            _ => unreachable!(),
        }
    }
    fn parse(src: &mut ByteCursor) -> Result<Frame, Error> {
        match cursor::get_u8(src)? {
            frameprefix::MESSAGE_TYPE => {
                Err(Error::Invalid("unexpected message type frame".into()))
            }
            frameprefix::STRING => {
                let line = cursor::get_line(src)?.to_vec();
                let string = String::from_utf8(line).map_err(|e| Error::Invalid(e.to_string()))?;
                Ok(Frame::String(string))
            }
            frameprefix::BYTES => {
                let len = cursor::get_decimal(src)? as usize;
                let n = len + 2;
                if src.remaining() < n {
                    return Err(Error::Incomplete);
                }
                let value = Vec::from(&src.chunk()[..len]);

                cursor::skip(src, n)?;

                Ok(Frame::Bytes(value))
            }
            frameprefix::TIME => {
                use chrono::{DateTime, Utc};
                let line = cursor::get_line(src)?.to_vec();
                let string = String::from_utf8(line).map_err(|e| Error::Invalid(e.to_string()))?;
                Ok(Frame::Time(
                    DateTime::parse_from_rfc3339(&string)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap(),
                ))
            }
            frameprefix::NULL => Ok(Frame::Null),
            _ => unreachable!(),
        }
    }
}

impl IntoIterator for MessageFrames {
    type Item = Frame;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// cursor utilities.
mod cursor {
    use super::*;

    pub(super) fn get_u8(src: &mut ByteCursor) -> Result<u8, Error> {
        if !src.has_remaining() {
            return Err(Error::Incomplete);
        }
        Ok(src.get_u8())
    }

    pub(super) fn skip(src: &mut ByteCursor, n: usize) -> Result<(), Error> {
        if src.remaining() < n {
            return Err(Error::Incomplete);
        }
        src.advance(n);
        Ok(())
    }

    pub(super) fn get_decimal(src: &mut ByteCursor) -> Result<u64, Error> {
        let line = get_line(src)?;

        atoi::atoi::<u64>(line)
            .ok_or_else(|| Error::Invalid("invalid protocol decimal format".into()))
    }

    pub(super) fn get_line<'a>(src: &'a mut ByteCursor) -> Result<&'a [u8], Error> {
        let start = src.position() as usize;
        let end = src.get_ref().len() - 1;

        for i in start..end {
            if src.get_ref()[i] == DELIMITER[0] && src.get_ref()[i + 1] == DELIMITER[1] {
                src.set_position((i + 2) as u64);

                return Ok(&src.get_ref()[start..i]);
            }
        }

        Err(Error::Incomplete)
    }
}
