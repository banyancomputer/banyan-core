use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiUser;
use crate::app::AppState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Json(user): Json<ApiUser>,
) -> Result<Response, UpdateUserError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();
    let accepted_tos_at = user.accepted_tos_at;

    // TODO: update all api fields
    sqlx::query_scalar!(
        r#"UPDATE users SET accepted_tos_at = $1 WHERE id = $2 RETURNING id;"#,
        accepted_tos_at,
        user_id
    )
    .fetch_one(&database)
    .await
    .map_err(UpdateUserError::UnableToUpdateUser)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateUserError {
    #[error("could not read user")]
    UnableToUpdateUser(sqlx::Error),
}

impl IntoResponse for UpdateUserError {
    fn into_response(self) -> Response {
        {
            tracing::error!("encountered error reading user: {self}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
