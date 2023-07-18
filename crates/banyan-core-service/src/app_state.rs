use std::path::PathBuf;

use axum::extract::FromRef;
use object_store::local::LocalFileSystem;
use sqlx::sqlite::SqlitePool;

mod database;
mod state_error;

use crate::config::Config;
pub use state_error::StateError;

#[derive(Clone)]
pub struct AppState {
    database_pool: SqlitePool,
    pub upload_directory: PathBuf,
}

impl AppState {
    pub(crate) async fn from_config(config: &Config) -> Result<Self, StateError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        LocalFileSystem::new_with_prefix(&config.upload_directory)
            .map_err(StateError::inaccessible_upload_directory)?;

        let database_pool = database::setup_pool(&config.database_url).await?;

        Ok(Self {
            database_pool,
            upload_directory: config.upload_directory.clone(),
        })
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.database_pool.clone()
    }
}
