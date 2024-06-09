use clap::Args;

use crate::client::Api;
use crate::protocol::Key;
use crate::Result;

#[derive(Args, Debug)]
pub struct GetCommand {
    #[arg(value_name = "KEY")]
    key: String,
}

impl GetCommand {
    pub async fn run(self, mut client: Box<dyn Api>) -> Result<()> {
        let GetCommand { key } = self;

        let key = Key::new(key)?;

        match client.get(key).await? {
            Some(value) => {
                println!("{:?}", value);
            }
            None => {
                println!("Not Found");
            }
        }
        Ok(())
    }
}
