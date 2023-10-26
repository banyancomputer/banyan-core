use sqlx::{FromRow, SqlitePool};

pub mod sqlite;

// todo: should be a UUID but Sqlite needs a from UUID implementation
#[derive(FromRow)]
pub struct BareId {
    pub id: String,
}

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

#[derive(Debug, thiserror::Error)]
pub enum SqlxError {
    #[error("query to database contained invalid syntax")]
    BadSyntax(sqlx::Error),

    #[error("unable to load data from database, appears to be invalid")]
    CorruptData(sqlx::Error),

    #[error("unable to communicate with the database")]
    DatabaseUnavailable(sqlx::Error),

    #[error("an internal database error occurred")]
    InternalError(sqlx::Error),

    #[error("error occurred while attempting database migration")]
    MigrationFailed(sqlx::migrate::MigrateError),

    #[error("unable to create record as it would violate a uniqueness constraint")]
    RecordExists,

    #[error("unable to locate record or associated foreign key")]
    RecordNotFound,
}

pub type DbResult<T = ()> = Result<T, SqlxError>;

pub fn map_sqlx_error(err: sqlx::Error) -> SqlxError {
    match err {
        sqlx::Error::ColumnDecode { .. } => SqlxError::CorruptData(err),
        sqlx::Error::RowNotFound => SqlxError::RecordNotFound,
        err if err.to_string().contains("FOREIGN KEY constraint failed") => {
            SqlxError::RecordNotFound
        }
        err if err.to_string().contains("UNIQUE constraint failed") => SqlxError::RecordExists,
        err => SqlxError::InternalError(err),
    }
}
