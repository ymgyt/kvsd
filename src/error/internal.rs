use std::error;
use std::fmt;
use std::io;

use backtrace::Backtrace;

#[derive(Debug)]
pub(crate) struct Error {
    kind: ErrorKind,
    backtrace: Option<Backtrace>,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    Io(io::Error),
    MaxKeyBytes { key: String, max_bytes: usize },
    EntryDecode { description: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind() {
            ErrorKind::Io(err) => err.fmt(f),
            ErrorKind::MaxKeyBytes { key, max_bytes, .. } => {
                write!(f, "max key bytes({}) exceeded. key: {:30}", max_bytes, key)
            }
            ErrorKind::EntryDecode { description, .. } => {
                write!(f, "entry decode error. {}", description)
            }
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
            backtrace: Some(Backtrace::new()),
        }
    }
}

impl error::Error for Error {}
