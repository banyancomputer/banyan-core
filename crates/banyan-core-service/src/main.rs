use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod api;
mod app;
mod auth;
mod database;
mod email;
mod event_bus;
mod extractors;
mod health_check;
mod hooks;
mod http_server;
mod utils;
mod workers;

use app::{AppState, Config};

#[tokio::main]
async fn main() {
    let config = match Config::from_env_and_args() {
        Ok(c) => c,
        Err(err) => {
            println!("failed load a valid config: {err}");
            std::process::exit(1);
        }
    };

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let stderr_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(non_blocking_writer)
        .with_filter(env_filter);

    tracing_subscriber::registry().with(stderr_layer).init();

    let state = AppState::from_config(&config).await.unwrap();

    http_server::run(config.listen_addr(), state).await;
}
