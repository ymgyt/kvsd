use tokio::io::{AsyncWrite, AsyncWriteExt, AsyncRead};
use tokio::sync::mpsc::Receiver;

use crate::core::UnitOfWork;
use crate::common::{info,Result};
use crate::core::table::index::Index;

pub(crate) struct Table<File> {
    file: File,
    index: Index,
    receiver: Receiver<UnitOfWork>,
}

impl <File> Table<File>
where File: AsyncWrite + AsyncRead + Unpin
{
    pub(crate) async fn new(receiver: Receiver<UnitOfWork>,  mut file: File) -> Result<Self> {
        // TODO: Buffering
       let index = Index::from_reader(&mut file).await?;

        Ok(Self {
            file,
            index,
            receiver
        })
    }

    pub(crate) async fn run(mut self) {
        while let Some(uow) = self.receiver.recv().await {
           match uow {
               _ => info!("Handle uow!!"),
           }
        }
    }
}

