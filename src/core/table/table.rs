use std::path::Path;

use tokio::fs;
use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt, AsyncWrite, SeekFrom};
use tokio::sync::mpsc::Receiver;

use crate::common::{debug, error, info, trace, ErrorKind, Result};
use crate::core::table::entry::Entry;
use crate::core::table::index::Index;
use crate::core::UnitOfWork;
use crate::protocol::Value;

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

                let current = self.file.seek(SeekFrom::Current(0)).await?;
                trace!("Seek {}", current);

                let entry = Entry::new(set.request.key.clone(), set.request.value)?;
                entry.encode_to(&mut self.file).await?;

                // TODO: return previous value.
                self.index
                    .add(set.request.key.into_string(), current as usize);

                set.response_sender
                    .expect("response already sent")
                    .send(Ok(None))
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()))?;

                Ok(())
            }
            UnitOfWork::Get(get) => {
                info!("{}", get.request);

                let maybe_offset = self.index.lookup_offset(&get.request.key);

                let offset = match maybe_offset {
                    Some(offset) => offset,
                    None => {
                        return get
                            .response_sender
                            .expect("response already sent")
                            .send(Ok(None))
                            .map_err(|_| {
                                ErrorKind::Internal("send to resp channel".to_owned()).into()
                            })
                    }
                };

                let current = self.file.seek(SeekFrom::Current(0)).await?;

                self.file.seek(SeekFrom::Start(offset as u64)).await?;
                let (_, entry) = Entry::decode_from(&mut self.file).await?;
                self.file.seek(SeekFrom::Start(current)).await?;

                let (key, value) = entry.take_key_value();

                debug_assert_eq!(*get.request.key, key);

                get.response_sender
                    .expect("response already sent")
                    .send(Ok(Some(Value::new(value)?)))
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()).into())
            }
            _ => unreachable!(),
        }
    }
}
