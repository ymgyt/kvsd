pub mod set;

use std::fmt;
use std::ops::Deref;

pub use crate::protocol::command::set::Set;

#[derive(Debug)]
pub struct Key(String);

impl Deref for Key {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> From<S> for Key
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Key(s.into())
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Key {
    pub fn into_string(self) -> String {
        self.0
    }
}

// Value represents binary data given by user.
// It does not have to be Vec<u8> because we do not mutate.
pub type Value = Box<[u8]>;

pub struct KeyValue {
    pub key: Key,
    pub value: Value,
}

impl<K, V> From<(K, V)> for KeyValue
where
    K: Into<String>,
    V: AsRef<[u8]>,
{
    fn from(kv: (K, V)) -> Self {
        KeyValue {
            key: Key(kv.0.into()),
            value: Box::<[u8]>::from(kv.1.as_ref()),
        }
    }
}

#[derive(Debug)]
pub enum CommandError {
    Etc(String),
}

#[derive(Debug)]
pub enum Command {
    Set(Set),
}
