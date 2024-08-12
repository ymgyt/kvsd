use kvsd::{
    self,
    cli::{self, authenticate, Command},
    config, KvsdError,
};

fn main() {
    // Install global collector configured based on KVS_LOG env var.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env(
            config::env::LOG_DIRECTIVE,
        ))
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .with_thread_ids(true)
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .on_thread_start(|| tracing::trace!("thread start"))
        .on_thread_stop(|| tracing::trace!("thread stop"))
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async {
            run().await;
        })
}

async fn run() {
    if let Err(err) = run_inner().await {
        let code = match err {
            KvsdError::Unauthenticated => {
                eprintln!("unauthenticated");
                2
            }
            _ => {
                eprintln!("{}", err);
                1
            }
        };
        std::process::exit(code);
    };
}

async fn run_inner() -> kvsd::Result<()> {
    // let m = cli::new().get_matches();
    let cli::KvsdCommand { client, command } = cli::parse();
    match command {
        Command::Ping(ping) => ping.run(authenticate(client).await?).await,
        Command::Delete(delete) => delete.run(authenticate(client).await?).await,
        Command::Get(get) => get.run(authenticate(client).await?).await,
        Command::Set(set) => set.run(authenticate(client).await?).await,
        Command::Server(server) => server.run(client.disable_tls).await,
    }
}
