use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("failed to clean up intermediate session state")]
    CleanupFailed,

    #[error("attempt to create new user after authentication failed: {0}")]
    CreationFailed(sqlx::Error),

    #[error("an error occurred working with the database connection: {0}")]
    DatabaseConnectionFailure(sqlx::Error),

    #[error("code exchange for oauth did not validate: {0}")]
    ExchangeCodeFailure(String),

    #[error("a database error occurred while attempting to locate a user: {0}")]
    LookupFailed(sqlx::Error),

    #[error("received callback from oauth but we didn't have a matching session: {0}")]
    MissingCallbackState(sqlx::Error),

    #[error("unable to retrieve authenticated user details: {0}")]
    ProfileUnavailable(reqwest::Error),

    #[error("no credentials available for provider '{0}'")]
    ProviderNotConfigured(String),

    #[error("failed to save session in the database: {0}")]
    SessionSaveFailed(sqlx::Error),

    #[error("failed to spawn blocking task for handle oauth code exchange: {0}")]
    SpawnFailure(tokio::task::JoinError),

    #[error("failed to lookup user data: {0}")]
    UserDataLookupFailed(sqlx::Error),

    #[error("attempted to authenticate against an unknown provider")]
    UnknownProvider,

    #[error("the account used for authentication has not verified its email")]
    UnverifiedEmail,
}

impl IntoResponse for AuthenticationError {
    fn into_response(self) -> Response {
        use AuthenticationError as AE;

        match self {
            AE::MissingCallbackState(ref err) => {
                tracing::warn!("{}: {err}", &self);
                let msg = serde_json::json!({"msg": "unknown authentication callback"});
                (StatusCode::BAD_REQUEST, Json(msg)).into_response()
            }
            AE::ProviderNotConfigured(_) | AE::UnknownProvider => {
                tracing::warn!("{}", &self);
                let msg = serde_json::json!({"msg": "unknown provider or provider not configured"});
                (StatusCode::NOT_FOUND, Json(msg)).into_response()
            }
            _ => {
                tracing::error!("{}", &self);
                let msg = serde_json::json!({"msg": "authentication workflow broke down"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response()
            }
        }
    }
}
