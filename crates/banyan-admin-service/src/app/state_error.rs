use std::fmt::Display;

use crate::database::DatabaseSetupError;

use super::config::ConfigError;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum StateErrorKind {
    #[error("bad configuration: {0}")]
    ConfigError(ConfigError),

    #[error("database setup failed: {0}")]
    DatabaseSetupError(DatabaseSetupError),

    #[error("unable to access configured upload directory: {0}")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("could not read platform key: {0}")]
    ReadPlatformKey(std::io::Error),

    #[error("could not load service key: {0}")]
    LoadPlatformKey(jwt_simple::Error),

    #[error("could not read service key: {0}")]
    ReadServiceKey(std::io::Error),

    #[error("unable to load service key: {0}")]
    LoadServiceKey(jwt_simple::Error),
}
