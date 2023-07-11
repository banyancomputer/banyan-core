use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use object_store::local::LocalFileSystem;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub upload_directory: PathBuf,
}

impl TryFrom<Config> for AppState {
    type Error = StateError;

    fn try_from(cfg: Config) -> Result<Self, Self::Error> {
        // Do a test setup to make sure the upload directory exists and is writable as an early
        // sanity check
        LocalFileSystem::new_with_prefix(&cfg.upload_directory)
            .map_err(StateError::inaccessible_upload_directory)?;

        Ok(Self { upload_directory: cfg.upload_directory.clone() })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct StateError {
    kind: StateErrorKind,
}

impl StateError {
    fn inaccessible_upload_directory(err: object_store::Error) -> Self {
        Self {
            kind: StateErrorKind::InaccessibleUploadDirectory(err),
        }
    }
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use StateErrorKind::*;

        let msg = match self.kind {
            InaccessibleUploadDirectory(_) => "service upload directory isn't available",
        };

        f.write_str(msg)
    }
}

impl std::error::Error for StateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use StateErrorKind::*;

        match &self.kind {
            InaccessibleUploadDirectory(err) => Some(err),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum StateErrorKind {
    InaccessibleUploadDirectory(object_store::Error),
}
