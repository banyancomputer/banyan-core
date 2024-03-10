use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, UsageError> {
    let database = state.database();
    let mut trans = database.begin().await?;
    let user_id = user_identity.id().to_string();
    let user = User::by_id(&mut trans, &user_id).await?;
    let hot_usage = user.hot_usage(&mut trans).await?.total();
    //let remaining_tokens = user.remaining_tokens(&mut trans).await?;
    //let maximum_tokens = user.maximum_tokens(&mut trans).await?;

    let resp = serde_json::json!({
        // Let's deprecate this from future versions once clients can accept the new version
        "size": hot_usage,
        "hot_usage": hot_usage,

     //   "tokens": remaining_tokens,
    });
    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    #[error("an error occurred querying the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for UsageError {
    fn into_response(self) -> Response {
        match self {
            UsageError::DatabaseFailure(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
