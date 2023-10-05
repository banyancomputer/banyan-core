#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
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

    #[error("requested database type was not recognized")]
    UnknownDbType,
}

