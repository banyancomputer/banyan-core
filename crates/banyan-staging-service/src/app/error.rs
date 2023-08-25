use crate::database::DatabaseSetupError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to parse program command lines")]
    ArgumentError(#[from] pico_args::Error),

    #[error("axum web server experienced critical error")]
    AxumServerError(#[from] hyper::Error),

    #[error("failed to initial the database")]
    DatabaseFailure(#[from] DatabaseSetupError),

    #[error("unable to access upload directory")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("authentication key provided could not be loaded")]
    BadAuthenticationKey(jwt_simple::Error),

    #[error("unable to write new platform auth key")]
    PlatformAuthWriteError(std::io::Error),

    #[error("provided session key was unable to be read")]
    UnreadableSessionKey(std::io::Error),
}

impl Error {
    pub fn auth_write_failed(err: std::io::Error) -> Self {
        Self::PlatformAuthWriteError(err)
    }

    pub fn invalid_key(err: jwt_simple::Error) -> Self {
        Self::BadAuthenticationKey(err)
    }

    pub fn inaccessible_upload_directory(err: object_store::Error) -> Self {
        Self::InaccessibleUploadDirectory(err)
    }

    pub fn unreadable_key(err: std::io::Error) -> Self {
        Self::UnreadableSessionKey(err)
    }
}
