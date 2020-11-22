use std::fmt;

use crate::protocol::{Key, Value};

pub struct Set {
    pub namespace: String,
    pub table: String,
    pub key: Key,
    pub value: Value,
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Set {namespace}/{table} {key} => {value:?}",
            namespace = &self.namespace,
            table = &self.table,
            key = &self.key,
            value = &self.value,
        )
    }
}
