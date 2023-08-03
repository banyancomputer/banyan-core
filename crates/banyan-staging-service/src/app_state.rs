use std::path::PathBuf;

use object_store::local::LocalFileSystem;

mod state_error;

use crate::config::Config;
pub use state_error::StateError;

#[derive(Clone)]
pub struct AppState {
    upload_directory: PathBuf,
}

impl AppState {
    pub async fn from_config(config: &Config) -> Result<Self, StateError> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        let upload_directory = config.upload_directory().clone();
        LocalFileSystem::new_with_prefix(&upload_directory)
            .map_err(StateError::inaccessible_upload_directory)?;

        Ok(Self { upload_directory })
    }

    pub fn upload_directory(&self) -> &PathBuf {
        &self.upload_directory
    }
}
