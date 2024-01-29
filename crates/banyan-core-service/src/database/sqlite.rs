use sqlx::migrate::Migrator;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::ConnectOptions;
use url::Url;

use crate::database::DatabaseConnection;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn connect_sqlite(url: &Url) -> Result<SqlitePool, sqlx::Error> {
    let connection_options = SqliteConnectOptions::from_url(url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .statement_cache_capacity(2_500)
        .synchronous(SqliteSynchronous::Normal);

    SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .connect_with(connection_options)
        .await
}

pub async fn mitrate_sqlite(
    conn: &mut DatabaseConnection,
) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(conn).await
}
