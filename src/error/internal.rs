use std::error;
use std::fmt;
use std::io;

use backtrace::Backtrace;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::oneshot::error::RecvError as OneshotRecvError;

use crate::common::KvsError;
use crate::protocol::message::{FrameError, ParseError};

#[derive(Debug)]
pub(crate) struct Error {
    kind: ErrorKind,
    backtrace: Option<Backtrace>,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(io::Error),
    Yaml(serde_yaml::Error),
    EntryDecode { description: String },
    UnknownMessageType { message_type: u8 },
    // Unintentional disconnection.
    ConnectionResetByPeer,
    NetworkFraming(String),
    Kvs(KvsError),
    Internal(String), // Box<dyn std::error::Error + Send + 'static> does not work :(
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind() {
            ErrorKind::Io(err) => err.fmt(f),
            ErrorKind::Yaml(err) => err.fmt(f),
            ErrorKind::EntryDecode { description, .. } => {
                write!(f, "entry decode error. {}", description)
            }
            ErrorKind::UnknownMessageType { message_type, .. } => {
                write!(f, "unknown message type {}", message_type)
            }
            ErrorKind::ConnectionResetByPeer => write!(f, "connection reset by peer"),
            ErrorKind::Kvs(err) => err.fmt(f),
            ErrorKind::Internal(err) => write!(f, "internal error {}", err),
            ErrorKind::NetworkFraming(err) => write!(f, "network framing {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::from(ErrorKind::Io(err))
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::with_backtrace(kind)
    }
}

impl From<KvsError> for Error {
    fn from(err: KvsError) -> Self {
        Error::from(ErrorKind::Kvs(err))
    }
}

impl From<FrameError> for Error {
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::Incomplete => Error::from(ErrorKind::NetworkFraming("incomplete".into())),
            FrameError::Invalid(s) => Error::from(ErrorKind::NetworkFraming(s)),
            FrameError::Other(err) => err,
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::Other(err) => err,
            ParseError::EndOfStream => Error::from(ErrorKind::NetworkFraming(
                "unexpected end of frame stream".into(),
            )),
        }
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::from(ErrorKind::Internal(err.to_string()))
    }
}

impl From<OneshotRecvError> for Error {
    fn from(err: OneshotRecvError) -> Self {
        Error::from(ErrorKind::Internal(err.to_string()))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::from(ErrorKind::Yaml(err))
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn is_eof(&self) -> bool {
        if let ErrorKind::Io(err) = self.kind() {
            err.kind().eq(&io::ErrorKind::UnexpectedEof)
        } else {
            false
        }
    }

    fn with_backtrace(kind: ErrorKind) -> Self {
        Self {
            kind,
            // backtrace: Some(Backtrace::new()),
            backtrace: None,
        }
    }
}

impl error::Error for Error {}
