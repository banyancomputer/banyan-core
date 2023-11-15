use sqlx::SqlitePool;

mod sqlite;

pub type Database = SqlitePool;

pub async fn connect(db_url: &url::Url) -> Result<Database, DatabaseSetupError> {
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
    #[error("requested database type was not recognized: {0}")]
    UnknownDbType(String),
    
    #[error("error occurred while attempting database migration: {0}")]
    MigrationFailed(sqlx::migrate::MigrateError),

    #[error("unable to perform initial connection and check of the database: {0}")]
    SetupFailed(sqlx::Error),
}
