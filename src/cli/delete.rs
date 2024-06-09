use clap::Args;

use crate::client::Api;
use crate::protocol::Key;
use crate::Result;

#[derive(Args, Debug)]
pub struct DeleteCommand {
    #[arg(value_name = "KEY")]
    key: String,
}

impl DeleteCommand {
    pub async fn run(self, mut client: Box<dyn Api>) -> Result<()> {
        let DeleteCommand { key } = self;
        let key = Key::new(key)?;

        match client.delete(key).await? {
            Some(value) => {
                println!("OK old value: {:?}", value);
            }
            None => {
                println!("OK");
            }
        }
        Ok(())
    }
}
