use std::fmt;

use crate::protocol::Key;

pub struct Delete {
    pub namespace: String,
    pub table: String,
    pub key: Key,
}

impl fmt::Display for Delete {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Delete {}/{} {}", self.namespace, self.table, self.key,)
    }
}
