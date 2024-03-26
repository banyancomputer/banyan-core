use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod api;
mod app;
mod auth;
mod clients;
mod database;
mod email;
mod event_bus;
mod extractors;
mod health_check;
mod hooks;
mod http_server;
mod pricing;
mod tasks;
mod utils;

use app::Config;

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
        .with_ansi(cfg!(debug_assertions))
        .with_filter(env_filter);

    tracing_subscriber::registry().with(stderr_layer).init();

    register_panic_logger();

    http_server::run(config).await;
}

/// Sets up system panics to use the tracing infrastructure to log reported issues. This doesn't
/// prevent the panic from taking out the service but ensures that it and any available information
/// is properly reported using the standard logging mechanism.
fn register_panic_logger() {
    std::panic::set_hook(Box::new(|panic| match panic.location() {
        Some(loc) => {
            tracing::error!(
                message = %panic,
                panic.file = loc.file(),
                panic.line = loc.line(),
                panic.column = loc.column(),
            );
        }
        None => tracing::error!(message = %panic),
    }));
}
