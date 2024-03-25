use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::database::models::{ConsumedStorage, Snapshot};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, CurrentTotalUsageError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();

    let mut conn = database.acquire().await?;
    let hot_storage = ConsumedStorage::total_consumption_for_user(&mut conn, &user_id).await?;
    let archival_storage = Snapshot::total_usage_for_user(&mut conn, &user_id).await?;

    let resp = serde_json::json!({
        "hot_storage": hot_storage.data_size + hot_storage.meta_size,
        "archival_storage": archival_storage,
    });
    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CurrentTotalUsageError {
    #[error("failed to calculate current total usage: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for CurrentTotalUsageError {
    fn into_response(self) -> Response {
        let (status_code, msg) = match self {
            CurrentTotalUsageError::DatabaseFailure(_) => {
                tracing::error!("{self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "backend service experienced an issue servicing the request",
                )
            }
        };

        let err_msg = serde_json::json!({"msg": msg});
        (status_code, Json(err_msg)).into_response()
    }
}
