use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use sqlx::SqliteConnection;

use crate::app::AppState;
use crate::database::models::User;
use crate::database::models::{ConsumedStorage, Snapshot};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, UsageError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();

    let mut conn = database.acquire().await?;
    let user = User::find_by_id(&mut conn, &user_id)
        .await?
        .ok_or(UsageError::NotFound)?;
    let hot_storage = user.hot_usage(&mut conn).await?;
    let archival_storage = user.archival_usage(&mut conn).await?;

    let resp = serde_json::json!({
        "hot_storage": hot_storage.data_size + hot_storage.meta_size,
        "archival_storage": archival_storage,
    });

    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    #[error("an error occurred querying the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
    #[error("associated data couldn't be found")]
    NotFound,
}

impl IntoResponse for UsageError {
    fn into_response(self) -> Response {
        match self {
            UsageError::DatabaseFailure(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("usage lookup error: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
