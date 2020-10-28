use std::convert::TryFrom;

use chrono::Utc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::common::{Error, ErrorKind, Result};
use crate::protocol::command::{Key, KeyValue, Value};

// Entry represent unit of data that is subject to an operation.
#[derive(PartialEq, Debug)]
struct Entry {
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
    value: Box<[u8]>,
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

    fn new(key: Key, value: Value) -> Result<Self> {
        let header = Header {
            key_bytes: key.len(),
            value_bytes: value.len(),
            timestamp_ms: Utc::now().timestamp_millis(),
            state: State::Active,
            crc_checksum: None,
        };

        let body = Body {
            key: key.into_string(),
            value,
        };

        Ok(Self { header, body })
    }

    // Write binary expression to writer.
    // return written bytes.
    // flush is left to the caller.
    async fn encode_to<W: AsyncWriteExt + Unpin>(&self, mut writer: W) -> Result<usize> {
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
        writer.write_all(self.body.value.as_ref()).await?;
        n += self.body.len();

        Ok(n)
    }

    // Construct Entry from reader.
    async fn decode_from<R: AsyncReadExt + Unpin>(mut reader: R) -> Result<Self> {
        // Assuming reader is buffered.
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

        Ok(Self {
            header,
            body: Body {
                key,
                value: value.into_boxed_slice(),
            },
        })
    }

    // Assert entry data consistency.
    fn assert(&self) -> bool {
        self.header.key_bytes == self.body.key.len()
            && self.header.value_bytes == self.body.value.len()
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
        self.key.len() + self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn from_key_value() {
        let kv = KeyValue::from(("key", b"hello"));
        let entry = Entry::try_from(kv).unwrap();

        assert_eq!(entry.header.key_bytes, 3);
        assert_eq!(entry.header.value_bytes, 5);
    }

    #[test]
    fn encode_decode() {
        tokio_test::block_on(async move {
            let kv = KeyValue::from(("key", "hello"));
            let entry = Entry::try_from(kv).unwrap();

            let mut buf = Cursor::new(Vec::new());
            let written = entry.encode_to(&mut buf).await.unwrap();
            assert_eq!(written, entry.encoded_len());

            // Seek to start. equivalent to seek(SeekFrom::Start(0)).
            buf.set_position(0);
            let decoded = Entry::decode_from(&mut buf).await.unwrap();

            assert_eq!(entry, decoded);
        })
    }
}
