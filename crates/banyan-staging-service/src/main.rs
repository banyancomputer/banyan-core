#![allow(dead_code)]
#![allow(unused_imports)]

use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod api;
mod app;
mod database;
mod extractors;
mod health_check;
mod http_server;
mod tasks;

use app::{Config, Error, Version};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stderr());
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let stderr_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(non_blocking_writer)
        .with_filter(env_filter);

    tracing_subscriber::registry().with(stderr_layer).init();

    let version = Version::new();
    tracing::info!(
        build_profile = ?version.build_profile,
        features = ?version.features,
        version = ?version.version,
        "service starting up"
    );

    register_panic_logger();

    let config = Config::parse_cli_arguments()?;
    http_server::run(config).await?;

    tracing::info!("shutting down normally");

    Ok(())
}

/// Sets up system panics to use the tracing infrastructure to log reported issues. This doesn't
/// prevent the panic from taking out the service but ensures that it and any available information
/// is properly reported using the standard logging mechanism.
fn register_panic_logger() {
    std::panic::set_hook(Box::new(|panic| {
        match panic.location() {
            Some(loc) => {
                tracing::error!(
                    message = %panic,
                    panic.file = loc.file(),
                    panic.line = loc.line(),
                    panic.column = loc.column(),
                );
            },
            None => tracing::error!(message = %panic),
        }
    }));
}

#[cfg(test)]
mod tests;
