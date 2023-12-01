use sqlx::SqlitePool;

pub mod models;
pub mod sqlite;

pub type Database = SqlitePool;

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
        sqlite::mitrate_sqlite(&db).await?;
        return Ok(db);
    }

    Err(DatabaseSetupError::UnknownDbType(
        db_url.scheme().to_string(),
    ))
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseSetupError {
    #[error("error occurred while attempting database migration: {0}")]
    MigrationFailed(sqlx::migrate::MigrateError),

    #[error("unable to perform initial connection and check of the database: {0}")]
    Unavailable(sqlx::Error),

    #[error("requested database type was not recognized: {0}")]
    UnknownDbType(String),
}

#[cfg(test)]
pub(crate) mod tests {
    use sqlx::{Pool, Sqlite};
    use sqlx::sqlite::SqlitePoolOptions;

    use crate::database::Database;

    pub async fn setup_database() -> Database {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to the database");

        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }
}
