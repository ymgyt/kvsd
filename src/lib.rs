#![allow(unused)]
use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;

static ROOT_DIR: &'static str = ".kvs";
static DATA_FILE: &'static str = "data";

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("kind: {:?} {}", .source.kind(), .source)]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("unknown err")]
    Unknown,
}

impl KvsError {
    fn is_eof(&self) -> bool {
        if let KvsError::Io { source } = self {
            source.kind() == io::ErrorKind::UnexpectedEof
        } else {
            false
        }
    }
}

pub type Result<T> = std::result::Result<T, KvsError>;

type Index = HashMap<String, u64>;

pub struct Kvs {
    data: fs::File,
}

impl Kvs {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // make sure root directory exists
        match fs::create_dir(ROOT_DIR) {
            Ok(_) => (),
            Err(err) => match err.kind() {
                io::ErrorKind::AlreadyExists => (),
                _ => return Err(KvsError::from(err)),
            },
        }

        let data = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .write(true)
            .open(&Path::new(ROOT_DIR).join(DATA_FILE))?;

        Ok(Kvs { data })
    }

    pub fn store<K: AsRef<str>, V: Serialize>(&mut self, key: K, value: V) -> Result<Option<V>> {
        Ok(None)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Entry {
    checksum: u32,
    key_len: u16,
    value_len: u32,
    key: String,
    value: Vec<u8>,
}

impl Entry {
    const HEADER_BYTES: usize = 32 + 16 + 32;

    fn encode<W: WriteBytesExt>(&self, mut w: W) -> Result<usize> {
        w.write_u32::<BE>(self.checksum)?;
        w.write_u16::<BE>(self.key_len)?;
        w.write_u32::<BE>(self.value_len)?;

        let mut n: usize = 0;
        n += w.write(self.key.as_bytes())?;
        debug_assert_eq!(n, self.key_len as usize, "encode key_len does not match");

        n += w.write(self.value.as_slice())?;
        debug_assert_eq!(
            n,
            self.key_len as usize + self.value_len as usize,
            "encode value_len does not match"
        );

        debug_assert_eq!(
            n + Entry::HEADER_BYTES,
            self.len(),
            "encode entry len does not match"
        );
        Ok(n + Entry::HEADER_BYTES)
    }

    fn decode<R: ReadBytesExt>(mut r: R) -> Result<Self> {
        let checksum = r.read_u32::<BE>()?;
        let key_len = r.read_u16::<BE>()?;
        let value_len = r.read_u32::<BE>()?;

        let mut key = String::with_capacity(key_len as usize);
        let got_key_len = r.by_ref().take(key_len as u64).read_to_string(&mut key)?;
        debug_assert_eq!(
            key_len as usize, got_key_len,
            "decode key_len does not match"
        );

        let mut value = Vec::with_capacity(value_len as usize);
        let got_value_ken = r.by_ref().take(value_len as u64).read_to_end(&mut value)?;
        debug_assert_eq!(
            value_len as usize, got_value_ken,
            "decode value_len does not match"
        );

        let entry = Self {
            checksum,
            key_len,
            value_len,
            key,
            value,
        };
        debug_assert_eq!(
            entry.len(),
            Entry::HEADER_BYTES + got_key_len + got_value_ken,
            "decode entry len does not match"
        );

        Ok(entry)
    }

    fn len(&self) -> usize {
        Entry::HEADER_BYTES + self.key_len as usize + self.value_len as usize
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Entries(Vec<Entry>);

impl Entries {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn decode<R: ReadBytesExt>(mut r: R) -> Result<Self> {
        let mut entries = Entries::new();
        loop {
            match Entry::decode(&mut r) {
                Ok(entry) => entries.0.push(entry),
                Err(err) => {
                    if err.is_eof() {
                        break;
                    }
                    return Err(err);
                }
            }
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn err_is_eof() {
        let err = KvsError::Io {
            source: io::Error::from(io::ErrorKind::UnexpectedEof),
        };
        assert!(err.is_eof());
    }

    #[test]
    fn entry_encode_decode() {
        let mut entry = Entry {
            checksum: 999,
            key_len: 3,
            value_len: 5,
            key: "abc".to_string(),
            value: vec![0x01, 0x02, 0x03, 0x04, 0x05],
        };
        let mut cursor = io::Cursor::new(vec![]);

        assert_eq!(
            Entry::HEADER_BYTES + 3 + 5,
            entry.encode(&mut cursor).unwrap()
        );

        cursor.set_position(0);
        let decoded = Entry::decode(&mut cursor).unwrap();
        assert_eq!(entry, decoded);
    }

    #[test]
    fn entries_decode() {
        let mut cursor = io::Cursor::new(vec![]);
        let entries = make_entries();
        entries.0.iter().for_each(|entry| {
            entry.encode(&mut cursor).unwrap();
        });
        cursor.set_position(0);

        let decoded = Entries::decode(&mut cursor).unwrap();
        assert_eq!(entries, decoded,);
    }

    fn make_entries() -> Entries {
        Entries(vec![
            Entry {
                checksum: 111,
                key_len: 3,
                value_len: 5,
                key: "abc".to_string(),
                value: vec![0x01, 0x02, 0x03, 0x04, 0x05],
            },
            Entry {
                checksum: 222,
                key_len: 4,
                value_len: 6,
                key: "abcd".to_string(),
                value: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            },
        ])
    }
}
