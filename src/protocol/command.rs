pub mod set;

pub use crate::protocol::command::set::Set;

pub struct Key(String);

pub enum Command {
    Set(Set),
}
