use std::path::Path;

use tokio::fs;
use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt, AsyncWrite, SeekFrom};
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;

use crate::common::{debug, error, info, trace, ErrorKind, Result};
use crate::core::table::entry::Entry;
use crate::core::table::index::Index;
use crate::core::UnitOfWork;
use crate::protocol::{Key, Value};

pub(crate) struct Table<File = fs::File> {
    file: File,
    index: Index,
    receiver: Receiver<UnitOfWork>,
}

impl Table<fs::File> {
    pub(crate) async fn from_path(
        receiver: Receiver<UnitOfWork>,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.as_ref())
            .await?;

        Table::new(receiver, f).await
    }
}

impl<File> Table<File>
where
    File: AsyncWrite + AsyncRead + AsyncSeek + Unpin,
{
    pub(crate) async fn new(receiver: Receiver<UnitOfWork>, mut file: File) -> Result<Self> {
        let pos = file.seek(SeekFrom::Current(0)).await?;
        info!("initial pos {}", pos);

        // TODO: Buffering
        let index = Index::from_reader(&mut file).await?;
        // TODO: summary
        debug!("{:?}", index);

        Ok(Self {
            file,
            index,
            receiver,
        })
    }

    pub(crate) async fn run(mut self) {
        while let Some(uow) = self.receiver.recv().await {
            if let Err(err) = self.handle_uow(uow).await {
                error!("handle uow {}", err);
            }
        }
    }

    async fn handle_uow(&mut self, uow: UnitOfWork) -> Result<()> {
        match uow {
            UnitOfWork::Set(set) => {
                info!("{}", set.request);

                let old_value = match self.lookup_entry(&set.request.key).await? {
                    Some(entry) => {
                        let (_, value) = entry.take_key_value();
                        Some(value)
                    }
                    None => None,
                };

                let current = self.file.seek(SeekFrom::Current(0)).await?;
                trace!("Seek {}", current);

                let entry = Entry::new(set.request.key.clone(), set.request.value)?;
                entry.encode_to(&mut self.file).await?;

                self.index
                    .add(set.request.key.into_string(), current as usize);

                self.send_value(set.response_sender, Ok(old_value.map(Value::new_unchecked)))
            }
            UnitOfWork::Get(get) => {
                info!("{}", get.request);

                let entry = match self.lookup_entry(&get.request.key).await? {
                    Some(entry) => entry,
                    None => return self.send_value(get.response_sender, Ok(None)),
                };

                let (key, value) = entry.take_key_value();

                debug_assert_eq!(*get.request.key, key);

                self.send_value(get.response_sender, Ok(Some(Value::new_unchecked(value))))
            }
            UnitOfWork::Delete(delete) => {
                info!("{}", delete.request);

                let mut entry = match self.lookup_entry(&delete.request.key).await? {
                    Some(entry) => entry,
                    None => return self.send_value(delete.response_sender, Ok(None)),
                };

                let value = entry.mark_deleted();
                entry.encode_to(&mut self.file).await?;

                self.index.remove(delete.request.key.as_str());

                self.send_value(
                    delete.response_sender,
                    Ok(Some(Value::new(value.unwrap())?)),
                )
            }
            _ => unreachable!(),
        }
    }

    fn send_value(
        &self,
        sender: Option<oneshot::Sender<Result<Option<Value>>>>,
        value: Result<Option<Value>>,
    ) -> Result<()> {
        sender
            .expect("response already sent")
            .send(value)
            .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()).into())
    }

    async fn lookup_entry(&mut self, key: &Key) -> Result<Option<Entry>> {
        let maybe_offset = self.index.lookup_offset(key);

        let offset = match maybe_offset {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let current = self.file.seek(SeekFrom::Current(0)).await?;

        self.file.seek(SeekFrom::Start(offset as u64)).await?;
        let (_, entry) = Entry::decode_from(&mut self.file).await?;
        self.file.seek(SeekFrom::Start(current)).await?;

        Ok(Some(entry))
    }
}
