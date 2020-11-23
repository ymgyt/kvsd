use std::convert::{TryFrom, TryInto};

use chrono::Utc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::common::{Error, ErrorKind, KvsError, Result};
use crate::protocol::{Key, KeyValue, Value};

// Entry represent unit of data that is subject to an operation.
#[derive(PartialEq, Debug)]
pub(super) struct Entry {
    header: Header,
    body: Body,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Invalid = 0,
    Active = 1,
    Deleted = 2,
}

// store mata value for entry.
#[derive(PartialEq, Debug)]
struct Header {
    // key length.
    key_bytes: usize,
    // value length.
    value_bytes: usize,
    // entry crated timestamp.
    // milliseconds since January 1,1970 UTC
    timestamp_ms: i64,
    // entry state. for support delete operation.
    state: State,
    // check data integrity.
    crc_checksum: Option<u32>,
}

// actual data provided by user.
#[derive(PartialEq, Debug)]
struct Body {
    key: String,
    value: Option<Box<[u8]>>,
}

impl TryFrom<KeyValue> for Entry {
    type Error = Error;
    fn try_from(kv: KeyValue) -> Result<Self, Self::Error> {
        Entry::new(kv.key, kv.value)
    }
}

impl Entry {
    const HEADER_BYTES: usize = 8 // key_bytes
        + 8 // value_bytes
        + 8 // timestamp_ms
        + 1 // state
        + 4 // crc_checksum
    ;

    pub(super) fn new(key: Key, value: Value) -> Result<Self> {
        let header = Header {
            key_bytes: key.len(),
            value_bytes: value.len(),
            timestamp_ms: Utc::now().timestamp_millis(),
            state: State::Active,
            crc_checksum: None,
        };

        let body = Body {
            key: key.into_string(),
            value: Some(value.into_boxed_bytes()),
        };

        let mut entry = Self { header, body };
        entry.header.crc_checksum = Some(entry.calc_crc_checksum());

        Ok(entry)
    }

    pub(super) fn mark_deleted(&mut self) -> Option<Box<[u8]>> {
        let value = self.body.value.take();

        self.header.value_bytes = 0;
        self.header.timestamp_ms = Utc::now().timestamp_millis();
        self.header.state = State::Deleted;
        self.header.crc_checksum = Some(self.calc_crc_checksum());

        value
    }

    fn try_from_key_value<T>(kv: T) -> Result<Self>
    where
        T: TryInto<KeyValue, Error = KvsError>,
    {
        let kv = kv.try_into()?;
        Entry::try_from(kv)
    }

    // Write binary expression to writer.
    // return written bytes.
    // flush is left to the caller.
    pub(crate) async fn encode_to<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> Result<usize> {
        // Assuming that the validation is done at the timeout entry construction.
        debug_assert!(self.assert());

        let mut n: usize = Entry::HEADER_BYTES;
        // Header
        writer.write_u64(self.header.key_bytes as u64).await?;
        writer.write_u64(self.header.value_bytes as u64).await?;
        writer.write_i64(self.header.timestamp_ms as i64).await?;
        writer.write_u8(self.header.state as u8).await?;
        writer
            .write_u32(self.header.crc_checksum.unwrap_or(0))
            .await?;

        // Body
        writer.write_all(self.body.key.as_bytes()).await?;
        if let Some(value) = &self.body.value {
            writer.write_all(value).await?;
        }
        n += self.body.len();

        Ok(n)
    }

    // Construct Entry from reader.
    pub(super) async fn decode_from<R: AsyncReadExt + Unpin>(
        mut reader: R,
    ) -> Result<(usize, Self)> {
        // Assuming reader is buffered.
        //
        // calling order is important.
        // We can't like this for eval_order_dependence(https://rust-lang.github.io/rust-clippy/master/index.html#eval_order_dependence)
        // Header {
        //   key_bytes: reader.read_u64().await        <-- second
        //   value_bytes: reader.read_u64().await      <-- first
        // }
        let key_bytes = reader.read_u64().await? as usize;
        let value_bytes = reader.read_u64().await? as usize;
        let timestamp_ms = reader.read_i64().await?;
        let state = State::from(reader.read_u8().await?);
        let crc_checksum = reader
            .read_u32()
            .await
            .map(|n| if n == 0 { None } else { Some(n) })?;

        let header = Header {
            key_bytes,
            value_bytes,
            timestamp_ms,
            state,
            crc_checksum,
        };

        let mut buf = Vec::with_capacity(header.body_len());
        reader
            .take(header.body_len() as u64)
            .read_to_end(buf.as_mut())
            .await?;

        let value = buf.split_off(header.key_bytes);

        let key = String::from_utf8(buf).map_err(|e| ErrorKind::EntryDecode {
            description: e.to_string(),
        })?;

        let value = if !value.is_empty() {
            Some(value.into_boxed_slice())
        } else {
            None
        };

        let entry = Self {
            header,
            body: Body { key, value },
        };

        Ok((entry.encoded_len(), entry))
    }

    pub(super) fn is_active(&self) -> bool {
        self.header.state == State::Active
    }

    pub(super) fn take_key(self) -> String {
        self.body.key
    }

    pub(super) fn take_key_value(self) -> (String, Box<[u8]>) {
        (self.body.key, self.body.value.unwrap())
    }

    fn calc_crc_checksum(&self) -> u32 {
        let mut h = crc32fast::Hasher::new();
        h.update(
            [
                self.header.key_bytes.to_be_bytes(),
                self.header.value_bytes.to_be_bytes(),
                self.header.timestamp_ms.to_be_bytes(),
            ]
            .concat()
            .as_ref(),
        );

        h.update((self.header.state as u8).to_be_bytes().as_ref());
        h.update(self.body.key.as_bytes());
        if let Some(value) = &self.body.value {
            h.update(value);
        }
        h.finalize()
    }

    // Assert entry data consistency.
    fn assert(&self) -> bool {
        self.header.key_bytes == self.body.key.len()
            && self.header.value_bytes == self.body.value.as_ref().map(|v| v.len()).unwrap_or(0)
            && self.header.crc_checksum.unwrap_or(0) == self.calc_crc_checksum()
    }

    // Return assuming encoded bytes length.
    fn encoded_len(&self) -> usize {
        Entry::HEADER_BYTES + self.body.len()
    }
}

impl From<u8> for State {
    fn from(n: u8) -> Self {
        match n {
            1 => State::Active,
            2 => State::Deleted,
            _ => State::Invalid,
        }
    }
}

impl Header {
    fn body_len(&self) -> usize {
        self.key_bytes + self.value_bytes
    }
}

impl Body {
    fn len(&self) -> usize {
        self.key.len()
            + match &self.value {
                Some(value) => value.len(),
                None => 0,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::table::index::Index;
    use std::io::Cursor;

    #[test]
    fn from_key_value() {
        let entry = Entry::try_from_key_value(("key", b"hello")).unwrap();

        assert_eq!(entry.header.key_bytes, 3);
        assert_eq!(entry.header.value_bytes, 5);
        assert!(entry.assert())
    }

    #[test]
    fn encode_decode() {
        tokio_test::block_on(async move {
            let entry = Entry::try_from_key_value(("key", "hello")).unwrap();

            let mut buf = Cursor::new(Vec::new());
            let written = entry.encode_to(&mut buf).await.unwrap();
            assert_eq!(written, entry.encoded_len());

            buf.set_position(0);
            let (_, decoded) = Entry::decode_from(&mut buf).await.unwrap();

            assert_eq!(entry, decoded);
            assert!(decoded.assert());
        })
    }

    #[test]
    fn delete() {
        tokio_test::block_on(async move {
            let mut entry1 = Entry::try_from_key_value(("kv1", "value1")).unwrap();
            entry1.mark_deleted();

            let mut buf = Cursor::new(Vec::new());
            entry1.encode_to(&mut buf).await.unwrap();

            buf.set_position(0);

            let (_, decoded) = Entry::decode_from(&mut buf).await.unwrap();

            assert_eq!(entry1, decoded);
            assert_eq!(decoded.header.state, State::Deleted);
        })
    }

    #[test]
    fn construct_index() {
        tokio_test::block_on(async move {
            let mut entry1 = Entry::try_from_key_value(("key1", "value1")).unwrap();
            let entry2 = Entry::try_from_key_value(("key2", "value2")).unwrap();

            entry1.mark_deleted();

            let mut buf = Cursor::new(Vec::new());
            entry1.encode_to(&mut buf).await.unwrap();
            entry2.encode_to(&mut buf).await.unwrap();

            buf.set_position(0);

            let index = Index::from_reader(&mut buf).await.unwrap();

            let entry2_offset = index.lookup_offset("key2").unwrap();
            buf.set_position(entry2_offset as u64);

            let (_, decoded) = Entry::decode_from(&mut buf).await.unwrap();
            assert_eq!(entry2, decoded);

            assert_eq!(None, index.lookup_offset("key1"))
        })
    }
}
