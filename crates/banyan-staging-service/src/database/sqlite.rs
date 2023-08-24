use std::str::FromStr;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous};

use crate::database::DatabaseSetupError;

static MIGRATOR: Migrator = sqlx::migrate!("migrations/sqlite");

pub(super) async fn configure_pool(url: &str) -> Result<SqlitePool, DatabaseSetupError> {
    let connection_options = SqliteConnectOptions::from_str(url)
        .map_err(|err| DatabaseSetupError::BadUrl(err))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .statement_cache_capacity(250)
        .synchronous(SqliteSynchronous::Normal);

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .idle_timeout(std::time::Duration::from_secs(90))
        .max_lifetime(std::time::Duration::from_secs(1_800))
        .min_connections(1)
        .max_connections(16)
        .connect_lazy_with(connection_options);

    Ok(pool)
}

pub(super) async fn run_migrations(pool: &SqlitePool) -> Result<(), DatabaseSetupError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|err| DatabaseSetupError::MigrationFailed(err))
}
