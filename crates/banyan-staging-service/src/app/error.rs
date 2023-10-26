#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to parse program command lines")]
    BadArgument(#[from] pico_args::Error),

    #[error("axum web server experienced critical error")]
    AxumServer(#[from] hyper::Error),

    #[error("unable to setup database")]
    DatabaseSetupError(#[from] crate::database::DatabaseSetupError),

    #[error("failed to initial the database")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("unable to access upload directory")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("invalid db url")]
    InvalidDatabaseUrl(url::ParseError),

    #[error("authentication key provided could not be loaded")]
    BadAuthenticationKey(jwt_simple::Error),

    #[error("unable to write new platform auth key")]
    PlatformAuthFailedWrite(std::io::Error),

    #[error("provided session key was unable to be read")]
    UnreadableSessionKey(std::io::Error),
}
