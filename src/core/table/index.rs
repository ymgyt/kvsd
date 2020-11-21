use std::collections::HashMap;

use tokio::prelude::io::AsyncReadExt;

use crate::common::Result;
use crate::core::table::entry::Entry;

#[derive(Debug)]
pub(super) struct Index {
    // key to file offset mapping.
    entry_offsets: HashMap<String, usize>,
}

impl Index {
    pub(super) async fn from_reader<R: AsyncReadExt + Unpin>(mut reader: R) -> Result<Self> {
        let mut entries = HashMap::new();
        let mut pos: usize = 0;
        loop {
            match Entry::decode_from(&mut reader).await {
                Ok((n, entry)) => {
                    entries.insert(entry.take_key(), pos);
                    pos = pos.checked_add(n).unwrap();
                }
                Err(err) if err.is_eof() => {
                    return Ok(Index::new(entries));
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }
    pub(super) fn add(&mut self, key: String, offset: usize) -> Option<usize>{
       self.entry_offsets.insert(key, offset)
    }

    pub(super) fn lookup_offset(&self, key: &str) -> Option<usize> {
        self.entry_offsets.get(key).cloned()
    }

    fn new(entry_offsets: HashMap<String, usize>) -> Self {
        Self { entry_offsets }
    }
}
