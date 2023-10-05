use std::time::Duration;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::ConnectOptions;
use url::Url;

use crate::database::{DatabaseError, DbResult};

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn connect_sqlite(url: &Url) -> DbResult<SqlitePool> {
    let connection_options = SqliteConnectOptions::from_url(url)
        .map_err(|err| DatabaseError::DatabaseUnavailable(err))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .statement_cache_capacity(2_500)
        .synchronous(SqliteSynchronous::Normal);

    SqlitePoolOptions::new()
        .idle_timeout(Duration::from_secs(90))
        .max_lifetime(Duration::from_secs(1_800))
        .min_connections(1)
        .max_connections(16)
        .connect_with(connection_options)
        .await
        .map_err(|err| DatabaseError::DatabaseUnavailable(err))
}

pub async fn mitrate_sqlite(pool: &SqlitePool) -> DbResult {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|err| DatabaseError::MigrationFailed(err))
}
