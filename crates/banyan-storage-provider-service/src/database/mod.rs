use sqlx::{pool::PoolConnection, FromRow, Sqlite, SqlitePool};

pub mod models;
mod sqlite;
#[cfg(test)]
pub(crate) mod test_helpers;

#[derive(FromRow)]
pub struct BareId {
    pub id: String,
}

impl BareId {
    pub fn uuid(&self) -> Result<uuid::Uuid, DatabaseError> {
        uuid::Uuid::parse_str(&self.id).map_err(DatabaseError::CorruptId)
    }
}

pub type Database = SqlitePool;
pub type DatabaseConnection = sqlx::SqliteConnection;

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

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("query to database contained invalid syntax")]
    BadSyntax(sqlx::Error),

    #[error("unable to load data from database, appears to be invalid")]
    CorruptData(sqlx::Error),

    #[error("encountered corrupt database id")]
    CorruptId(uuid::Error),

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

pub type DbResult<T = ()> = Result<T, DatabaseError>;

pub fn map_sqlx_error(err: sqlx::Error) -> DatabaseError {
    match err {
        sqlx::Error::ColumnDecode { .. } => DatabaseError::CorruptData(err),
        sqlx::Error::RowNotFound => DatabaseError::RecordNotFound,
        err if err.to_string().contains("FOREIGN KEY constraint failed") => {
            DatabaseError::RecordNotFound
        }
        err if err.to_string().contains("UNIQUE constraint failed") => DatabaseError::RecordExists,
        err => DatabaseError::InternalError(err),
    }
}
