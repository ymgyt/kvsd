#![allow(unused)]
use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::hash::Hasher;
use std::io::{self, Read, Seek, SeekFrom, Write};
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
    #[error("bincode: {}", .source)]
    Serialize {
        #[from]
        source: bincode::Error,
    },
    #[error("max key length({}) exceeded", Entry::MAX_KEY_LENGTH)]
    MaxKeyLength,
    #[error("max value bytes({}) exceeded", Entry::MAX_VALUE_BYTES)]
    MaxValueBytes,
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

pub struct Kvs {
    data: fs::File,
    index: Index,
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

        let mut data = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .write(true)
            .open(&Path::new(ROOT_DIR).join(DATA_FILE))?;

        let index = Index::construct_from(&mut data)?;

        Ok(Kvs { data, index })
    }

    pub fn store<K: AsRef<str>, V: Serialize>(&mut self, key: K, value: V) -> Result<()> {
        let value = bincode::serialize(&value)?;
        let entry = Entry::new(key.as_ref(), value)?;
        let current_position = self.data.seek(SeekFrom::Current(0))?;
        entry.encode(&mut self.data)?; // by_ref does not work for multiple candidate
        self.index.hm.insert(entry.key, current_position as usize);
        Ok(())
    }

    pub fn get<D: DeserializeOwned>(&mut self, key: &str) -> Result<Option<D>> {
        if let (Some(&offset)) = self.index.hm.get(key) {
            let current_position = self.data.seek(SeekFrom::Current(0))?;
            self.data.seek(SeekFrom::Start(offset as u64))?;
            let entry = Entry::decode(&self.data)?;
            let value = bincode::deserialize_from(io::Cursor::new(entry.value))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct Index {
    hm: HashMap<String, usize>,
}

impl Index {
    fn construct_from<R: Read>(mut r: R) -> Result<Self> {
        let mut hm = HashMap::new();
        let mut position = 0;
        let err = loop {
            if let Err(err) = Entry::decode(r.by_ref()).map(|entry| {
                let entry_len = entry.len();
                hm.insert(entry.key, position);
                position += entry_len;
            }) {
                break err;
            }
        };
        if err.is_eof() {
            Ok(Self { hm })
        } else {
            Err(err)
        }
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
    const MAX_KEY_LENGTH: usize = u16::max_value() as usize;
    const MAX_VALUE_BYTES: usize = u32::max_value() as usize;

    fn new<K: Into<String>>(key: K, value: Vec<u8>) -> Result<Self> {
        let key = key.into();
        if key.len() > Entry::MAX_KEY_LENGTH {
            return Err(KvsError::MaxKeyLength);
        }
        let key_len = key.len() as u16;

        if value.len() > Entry::MAX_VALUE_BYTES {
            return Err(KvsError::MaxValueBytes);
        }
        let value_len = value.len() as u32;

        // create crc32
        let mut h = crc32fast::Hasher::new();
        let mut buff = Vec::with_capacity(6);
        buff.write_u16::<BE>(key_len)?;
        buff.write_u32::<BE>(value_len)?;
        h.update(&buff);
        h.update(key.as_bytes());
        h.update(value.as_slice());
        let checksum = h.finalize();

        Ok(Self {
            checksum,
            key_len,
            value_len,
            key: key.to_string(),
            value,
        })
    }

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

    fn encode<W: Write>(&self, mut w: W) -> Result<(usize)> {
        let bytes = self
            .0
            .iter()
            .map(|entry| entry.encode(&mut w))
            .collect::<Result<Vec<usize>>>()?;
        Ok(bytes.iter().sum())
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
        let mut entry = Entry::new("abc", vec![0x01, 0x02, 0x03, 0x04, 0x05]).unwrap();
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
    fn entries_encode_decode() {
        let mut cursor = io::Cursor::new(vec![]);
        let entries = make_entries();
        entries.encode(&mut cursor).unwrap();
        cursor.set_position(0);

        let decoded = Entries::decode(&mut cursor).unwrap();
        assert_eq!(entries, decoded,);
    }

    fn make_entries() -> Entries {
        Entries(vec![
            Entry::new("abc", vec![0x01, 0x02, 0x03, 0x04, 0x05]).unwrap(),
            Entry::new("abcd", vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]).unwrap(),
        ])
    }

    #[test]
    fn index_construct_from() {
        let mut cursor = io::Cursor::new(vec![]);
        let entries = make_entries();
        entries.encode(&mut cursor).unwrap();
        cursor.set_position(0);

        let index = Index::construct_from(&mut cursor).unwrap();
        let mut n = 0;
        for entry in entries.0.iter() {
            assert_eq!(Some(&n), index.hm.get(&entry.key));
            n += entry.len();
        }
    }
}
