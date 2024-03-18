use sqlx::SqlitePool;

pub mod models;
pub mod sqlite;

#[cfg(test)]
pub(crate) mod test_helpers;

use crate::pricing;

/// The number of simultaneous dynamic binds any individual query is allowed to have. Technically
/// this is 32_768, but we want to ensure there are some query slots available for the non-bulk
/// parts of any particular query.
pub const BIND_LIMIT: usize = 32_000;

pub type Database = SqlitePool;

pub type DatabaseConnection = sqlx::SqliteConnection;

pub async fn connect(db_url: &url::Url) -> Result<Database, DatabaseSetupError> {
    // todo: I should figure out a way to delay the actual connection and running of migrations,
    // and reflect the service being unavailable in the readiness check until they're complete. If
    // our connection fails we should try a couple of times with a backoff before failing the
    // entire service...
    //
    // maybe a tokio task with a channel or shared state directly that can be consumed by the
    // healthcheck and database extractor... Maybe this state belongs on the database executor
    // itself...

    if db_url.scheme() == "sqlite" {
        let db = sqlite::connect_sqlite(db_url).await?;

        let mut conn = db.acquire().await?;
        sqlite::mitrate_sqlite(&mut conn).await?;
        pricing::sync_pricing_config(&mut conn, pricing::builtin_pricing_config()).await?;
        conn.close().await?;

        return Ok(db);
    }

    Err(DatabaseSetupError::UnknownDbType(
        db_url.scheme().to_string(),
    ))
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseSetupError {
    #[error("error occurred while attempting database migration: {0}")]
    MigrationFailed(#[from] sqlx::migrate::MigrateError),

    #[error("unable to perform initial connection and check of the database: {0}")]
    Unavailable(#[from] sqlx::Error),

    #[error("requested database type was not recognized: {0}")]
    UnknownDbType(String),
}
