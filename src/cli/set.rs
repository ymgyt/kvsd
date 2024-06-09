use clap::Args;

use crate::client::Api;
use crate::protocol::{Key, Value};
use crate::Result;

#[derive(Args, Debug)]
pub struct SetCommand {
    #[arg(value_name = "KEY", index = 1)]
    key: String,
    #[arg(value_name = "VALUE", index = 2)]
    value: String,
}

impl SetCommand {
    pub async fn run(self, mut client: Box<dyn Api>) -> Result<()> {
        let SetCommand { key, value } = self;

        let key = Key::new(key)?;
        let value = Value::new(value.as_bytes())?;

        if client.set(key, value).await.is_ok() {
            println!("OK");
        }

        Ok(())
    }
}
