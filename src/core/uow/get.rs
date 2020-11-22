use std::fmt;

use crate::protocol::Key;

pub struct Get {
    pub namespace: String,
    pub table: String,
    pub key: Key,
}

impl fmt::Display for Get {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Get {}/{} {}", self.namespace, self.table, self.key,)
    }
}
