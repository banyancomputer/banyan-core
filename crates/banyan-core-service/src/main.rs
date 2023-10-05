// #![feature(const_trait_impl)]

use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod api;
mod app;
mod auth;
mod db;
mod database;
mod error;
mod extractors;
mod health_check;
mod http_server;
mod utils;

use app::AppState;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("failed to build service config: {0}")]
    ConfigError(#[from] app::config::ConfigError),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = app::config::Config::from_env_and_args()
        .map_err(AppError::ConfigError)?;

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stderr());
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let stderr_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(non_blocking_writer)
        .with_filter(env_filter);

    tracing_subscriber::registry().with(stderr_layer).init();

    let state = AppState::from_config(&config).await.unwrap();

    http_server::run(state).await;

    Ok(())
}
