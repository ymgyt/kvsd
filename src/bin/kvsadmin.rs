use kvsd::{
    cli::{self, admin::KvsadminCommand},
    config,
};

fn init_tracing() {
    use tracing_subscriber::{
        filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt as _, Registry,
    };

    Registry::default()
        .with(
            fmt::Layer::new()
                .with_ansi(true)
                .with_file(false)
                .with_line_number(false)
                .with_target(true),
        )
        .with(
            EnvFilter::try_from_env(config::env::LOG_DIRECTIVE)
                .or_else(|_| EnvFilter::try_new("info"))
                .unwrap(),
        )
        .init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    let KvsadminCommand { command } = cli::admin::parse();

    let result = match command {
        cli::admin::Command::Table(table) => table.run().await,
    };

    if let Err(err) = result {
        tracing::error!("{err}");
    }
}
