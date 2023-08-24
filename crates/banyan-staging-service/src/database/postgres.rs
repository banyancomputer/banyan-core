use std::str::FromStr;

use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPool};

use crate::database::DatabaseSetupError;

static MIGRATOR: Migrator = sqlx::migrate!("migrations/postgres");

pub(super) async fn configure_pool(url: &str) -> Result<PgPool, DatabaseSetupError> {
    let connection_options = PgConnectOptions::from_str(&url)
        .map_err(|err| DatabaseSetupError::BadUrl(err))?
        .application_name(env!("CARGO_PKG_NAME"))
        .statement_cache_capacity(250);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(90))
        .max_lifetime(std::time::Duration::from_secs(1_800))
        .min_connections(1)
        .max_connections(16)
        .connect_lazy_with(connection_options);

    Ok(pool)
}

pub(super) async fn run_migrations(pool: &PgPool) -> Result<(), DatabaseSetupError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|err| DatabaseSetupError::MigrationFailed(err))
}
