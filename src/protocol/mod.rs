//! protocol module specifies communication between the server and client.

pub(crate) mod connection;

pub(crate) mod message;

use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;

use crate::common::{KvsdError, Result};

/// Maximum number of bytes in Key.
/// if it's not in ascii, Len  is misleading, so using Bytes explicitly.
pub const MAX_KYE_BYTES: usize = 1024;

/// Maximum number of bytes in Value.
pub const MAX_VALUE_BYTES: usize = 1024 * 1024 * 10;

/// Key represents a string that meets the specifications of the kvsd protocol.
/// other components can handle Key without checking the length.
#[derive(Debug, Clone, PartialEq)]
pub struct Key(String);

impl Deref for Key {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Key {
    type Error = KvsdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Key::new(s)
    }
}

impl Key {
    /// Construct Key from given string.
    pub fn new(s: impl Into<String>) -> Result<Self, KvsdError> {
        let s = s.into();
        if s.len() > MAX_KYE_BYTES {
            Err(KvsdError::MaxKeyBytes {
                key: s,
                max_bytes: MAX_KYE_BYTES,
            })
        } else {
            Ok(Self(s))
        }
    }

    /// Convert into String.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Value represents binary data given by user.
/// It does not have to be Vec<u8> because we do not mutate.
#[derive(Clone, PartialEq)]
pub struct Value(Box<[u8]>);

impl Deref for Value {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Value {
    /// Construct Value.
    /// if given value exceed the maximum bytes, return error.
    pub fn new(v: impl Into<Box<[u8]>>) -> Result<Self, KvsdError> {
        let v = v.into();
        if v.len() > MAX_VALUE_BYTES {
            Err(KvsdError::MaxValueBytes {
                max_bytes: MAX_VALUE_BYTES,
            })
        } else {
            Ok(Value(v))
        }
    }

    pub(crate) fn new_unchecked(v: impl Into<Box<[u8]>>) -> Self {
        Value(v.into())
    }

    /// Convert into Box<[u8]>
    pub fn into_boxed_bytes(self) -> Box<[u8]> {
        self.0
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len() > 1024 {
            write!(f, "{}", String::from_utf8_lossy(&self.deref()[..1024]))
        } else {
            write!(f, "{}", String::from_utf8_lossy(self.deref()))
        }
    }
}

pub(crate) struct KeyValue {
    pub(crate) key: Key,
    pub(crate) value: Value,
}

impl<K, V> TryFrom<(K, V)> for KeyValue
where
    K: Into<String>,
    V: AsRef<[u8]>,
{
    type Error = KvsdError;
    fn try_from(kv: (K, V)) -> Result<Self, Self::Error> {
        Ok(KeyValue {
            key: Key::new(kv.0)?,
            value: Value::new(Box::<[u8]>::from(kv.1.as_ref()))?,
        })
    }
}
