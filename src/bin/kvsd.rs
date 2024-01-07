use kvsd::{self, cli, KvsdError};

fn main() {
    // Install global collector configured based on KVS_LOG env var.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("KVSD_LOG"))
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
    let m = cli::new().get_matches();
    match m.subcommand() {
        (cli::PING, Some(sm)) => cli::ping(sm).await,
        (cli::SERVER, Some(sm)) => cli::server(sm).await,
        (cli::SET, Some(sm)) => cli::set(sm).await,
        (cli::GET, Some(sm)) => cli::get(sm).await,
        (cli::DELETE, Some(sm)) => cli::delete(sm).await,
        (_, _) => unreachable!(),
    }
}
