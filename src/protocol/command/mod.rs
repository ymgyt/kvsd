pub mod set;

pub use crate::protocol::command::set::Set;

#[derive(Debug)]
pub enum CommandError {
    Etc(String),
}

#[derive(Debug)]
pub enum Command {
    Set(Set),
}
