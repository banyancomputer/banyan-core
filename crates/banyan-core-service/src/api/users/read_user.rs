use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiUser;
use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, ReadUserError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();
    let user = sqlx::query_as!(
        User,
        r#"SELECT *
            FROM users WHERE id = $1;"#,
        user_id,
    )
    .fetch_one(&database)
    .await
    .map_err(ReadUserError::UnableToReadUser)?;

    let api_user = ApiUser::from(user);
    Ok((StatusCode::OK, Json(api_user)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ReadUserError {
    #[error("could not read user")]
    UnableToReadUser(sqlx::Error),
}

impl IntoResponse for ReadUserError {
    fn into_response(self) -> Response {
        match &self {
            _ => {
                tracing::error!("encountered error reading user: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
