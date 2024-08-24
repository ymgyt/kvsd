#[derive(Debug)]
pub(crate) struct EntryDump {
    pub(crate) timestamp_ns: i64,
    pub(crate) is_deleted: bool,
    pub(crate) key: String,
    pub(crate) value: Vec<u8>,
}
