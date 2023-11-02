#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to parse program command lines: {0}")]
    BadArgument(#[from] pico_args::Error),

    #[error("axum web server experienced critical error: {0}")]
    AxumServer(#[from] hyper::Error),

    #[error("unable to setup database: {0}")]
    DatabaseSetup(#[from] crate::database::DatabaseSetupError),

    #[error("failed to initial the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("failed to write out fingerprint for auth key: {0}")]
    FingerprintWriteFailed(std::io::Error),

    #[error("unable to access upload directory: {0}")]
    InaccessibleUploadDirectory(object_store::Error),

    #[error("invalid db url: {0}")]
    InvalidDatabaseUrl(url::ParseError),

    #[error("authentication key provided could not be loaded: {0}")]
    BadAuthenticationKey(jwt_simple::Error),

    #[error("unable to write new platform auth key: {0}")]
    PlatformAuthFailedWrite(std::io::Error),

    #[error("failed to write public key for auth key")]
    PublicKeyWriteFailed(std::io::Error),

    #[error("unreadable platform verification key: {0}")]
    UnreadablePlatformVerificationKey(std::io::Error),

    #[error("provided signing key was unable to be read: {0}")]
    UnreadableSigningKey(std::io::Error),
}
