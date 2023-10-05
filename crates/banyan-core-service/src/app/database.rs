use std::str::FromStr;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous};

use crate::app::StateError;

static DATABASE_MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

pub(crate) async fn setup(db_url: &str) -> Result<SqlitePool, StateError> {
    let connection_options = SqliteConnectOptions::from_str(db_url)
        .map_err(StateError::bad_database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .statement_cache_capacity(2_500)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePool::connect_with(connection_options)
        .await
        .map_err(StateError::database_unavailable)?;

    DATABASE_MIGRATIONS
        .run(&pool)
        .await
        .map_err(StateError::migrations_failed)?;

    Ok(pool)
}
