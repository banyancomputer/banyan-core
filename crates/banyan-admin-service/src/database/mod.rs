use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use url::Url;

pub mod models;
pub mod sqlite;

/// The number of simultaneous dynamic binds any individual query is allowed to have. Technically
/// this is 32_768, but we want to ensure there are some query slots available for the non-bulk
/// parts of any particular query.
pub type Database = SqlitePool;

pub async fn connect(db_url: &url::Url) -> Result<Database, DatabaseSetupError> {
    if db_url.scheme() == "sqlite" {
        let db = connect_sqlite(db_url).await?;
        mitrate_sqlite(&db).await?;
        return Ok(db);
    }

    Err(DatabaseSetupError::UnknownDbType(
        db_url.scheme().to_string(),
    ))
}

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn connect_sqlite(url: &Url) -> Result<SqlitePool, DatabaseSetupError> {
    let connection_options = SqliteConnectOptions::from_url(url)
        .map_err(DatabaseSetupError::SetupFailed)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .statement_cache_capacity(2_500)
        .synchronous(SqliteSynchronous::Normal);

    SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .connect_with(connection_options)
        .await
        .map_err(DatabaseSetupError::SetupFailed)
}

pub async fn mitrate_sqlite(pool: &SqlitePool) -> Result<(), DatabaseSetupError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(DatabaseSetupError::MigrationFailed)
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseSetupError {
    #[error("requested database type was not recognized: {0}")]
    UnknownDbType(String),

    #[error("error occurred while attempting database migration: {0}")]
    MigrationFailed(sqlx::migrate::MigrateError),

    #[error("unable to perform initial connection and check of the database: {0}")]
    SetupFailed(sqlx::Error),
}
