use sqlx::SqlitePool;

mod error;
pub mod sqlite;

pub use error::DatabaseError;

pub type Db = SqlitePool;

pub type DbResult<T = ()> = Result<T, DatabaseError>;

pub async fn connect(db_url: &url::Url) -> DbResult<Db> {
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

    Err(DatabaseError::UnknownDbType)
}
