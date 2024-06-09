use clap::Args;

use crate::client::Api;
use crate::Result;

#[derive(Args, Debug)]
pub struct PingCommand {
    /// Ping counts
    #[arg(long, short = 'c', default_value_t = 1)]
    count: usize,
}

impl PingCommand {
    pub async fn run(self, mut client: Box<dyn Api>) -> Result<()> {
        let PingCommand { count } = self;

        let mut current = 1;
        while current <= count {
            let latency = client.ping().await?;
            println!(
                "ping (latency {}ms) {}/{}",
                latency.num_milliseconds(),
                current,
                count
            );
            current += 1;
        }

        Ok(())
    }
}
