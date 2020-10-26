use std::sync::Arc;

use tokio::io::{AsyncWrite,AsyncWriteExt, AsyncReadExt};
use tokio::sync::mpsc::{self, Receiver};

use crate::protocol::command::{Set, Command, Key};

#[derive(Debug)]
pub enum TableError {
    Etc(Box<dyn std::error::Error + Send + 'static>),
}

struct TableHandler<W> {
    writer: W,
    receiver: Receiver<Command>,
}

impl<W> TableHandler<W>
    where W: AsyncWrite + Unpin {

    pub async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
           match cmd {
               Command::Set(set) => {
                   tracing::info!("handle set command");

                   self.writer.write_all(set.value.as_ref()).await.unwrap();

                   if let Err(e) = set.result_sender.send(Ok(())) {
                       tracing::error!("failed to send set command result {:?}", e);
                   }
               }
           }
        }
    }

    pub fn new(writer: W, receiver: Receiver<Command>) -> Result<Self, TableError> {
        Ok(Self {
            writer,
            receiver,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_handler() {
        tokio::runtime::Builder::new_multi_thread().build().unwrap().block_on(async move {
            let (mut reader, mut writer) = tokio::io::duplex(1024);
            let (mut tx, mut rx) = mpsc::channel(10);

            tokio::spawn(async move {
                let handler = TableHandler::new(writer, rx).unwrap();
                handler.run().await;
            });

            let (result_tx, result_rx) = tokio::sync::oneshot::channel();
            tx.send(Command::Set(Set{
                key: Key::from("key"),
                value: bytes::Bytes::from("hello"),
                result_sender: result_tx,
            })).await.unwrap();

            let result = result_rx.await.unwrap().unwrap();
            assert_eq!(result, ());

            let mut buf = [0_u8; 5];
            reader.read_exact(&mut buf).await.unwrap();

            assert_eq!(std::str::from_utf8(&buf).unwrap(), "hello")
        })

    }
}
