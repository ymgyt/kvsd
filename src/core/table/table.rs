use std::path::Path;

use tokio::fs;
use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt, AsyncWrite, SeekFrom};
use tokio::sync::mpsc::Receiver;

use crate::common::{error, info, ErrorKind, Result};
use crate::core::table::entry::Entry;
use crate::core::table::index::Index;
use crate::core::UnitOfWork;

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
        // TODO: Buffering
        let index = Index::from_reader(&mut file).await?;

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
                info!("Set {}", set.request);

                let current = self.file.seek(SeekFrom::Current(0)).await?;
                info!("Seek {}", current);

                let entry = Entry::new(set.request.key.clone(), set.request.value)?;
                entry.encode_to(&mut self.file).await?;

                // TODO: return previous value.
                self.index
                    .add(set.request.key.into_string(), current as usize);

                set.response_sender
                    .send(Ok(None))
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()))?;

                Ok(())
            }
            _ => unreachable!(),
        }
    }
}
