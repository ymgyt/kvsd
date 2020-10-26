pub mod set;

pub use crate::protocol::command::set::Set;

#[derive(Debug)]
pub struct Key(String);

impl<S> From<S> for Key
    where S: Into<String> {
    fn from(s: S) -> Self { Key(s.into()) }
}

#[derive(Debug)]
pub enum CommandError {
    Etc(String),
}

#[derive(Debug)]
pub enum Command {
    Set(Set),
}
