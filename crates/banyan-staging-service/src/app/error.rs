use crate::database::DbError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to parse program command lines")]
    BadArgument(#[from] pico_args::Error),

    #[error("axum web server experienced critical error")]
    AxumServer(#[from] hyper::Error),

    #[error("failed to initial the database")]
    DatabaseFailure(#[from] DbError),

    #[error("unable to access upload directory")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("authentication key provided could not be loaded")]
    BadAuthenticationKey(jwt_simple::Error),

    #[error("unable to write new platform auth key")]
    PlatformAuthFailedWrite(std::io::Error),

    #[error("provided session key was unable to be read")]
    UnreadableSessionKey(std::io::Error),
}
