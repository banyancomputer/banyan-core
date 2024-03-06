use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, CurrentUserError> {
    let database = state.database();
    let mut conn = database.acquire().await?;
    let user_id = user_identity.id().to_string();

    match User::find_by_id(&mut conn, &user_id).await? {
        Some(u) => Ok((StatusCode::OK, Json(u.as_api_user(&mut conn).await?)).into_response()),
        None => Err(CurrentUserError::NotFound),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurrentUserError {
    #[error("an error occurred querying the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("current user doesn't exist?")]
    NotFound,
}

impl IntoResponse for CurrentUserError {
    fn into_response(self) -> Response {
        match self {
            CurrentUserError::DatabaseFailure(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            CurrentUserError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}
