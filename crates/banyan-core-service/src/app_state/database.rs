use std::str::FromStr;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous};

use crate::app_state::StateError;

static DATABASE_MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

pub(crate) fn setup_pool(db_url: &str) -> Result<SqlitePool, StateError> {
    todo!()
}
