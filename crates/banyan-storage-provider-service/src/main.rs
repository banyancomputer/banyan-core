#![allow(dead_code)]

mod http_server;

mod app;
mod api;
mod extractors;
mod database;
mod upload_store;
mod health_check;
mod tasks;
mod utils;

use app::{AppState, Config};

use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use axum::extract::Path;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, Server};
use futures::future::join_all;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};
use rand::Rng;
use serde::Serialize;
use time::ext::NumericalDuration;
use time::{Date, OffsetDateTime};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let config = match Config::parse_cli_arguments() {
        Ok(c) => c,
        Err(err) => {
            println!("failed load a valid config from arguments: {err}");
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