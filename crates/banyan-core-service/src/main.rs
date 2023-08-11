use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod api;
mod app_state;
mod config;
mod db;
mod extractors;
mod health_check;
mod http_server;
mod util;

use app_state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stderr());
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let stderr_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(non_blocking_writer)
        .with_filter(env_filter);

    tracing_subscriber::registry().with(stderr_layer).init();

    let config = config::parse_arguments()?;
    let app_state = AppState::from_config(&config).await?;

    http_server::run(app_state).await?;

    Ok(())
}
