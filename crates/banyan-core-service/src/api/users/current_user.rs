use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiUser;
use crate::app::AppState;
use crate::database::models::{MetricsTraffic, User};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, CurrentUserError> {
    let database = state.database();
    let mut conn = database.acquire().await?;
    let user_id = user_identity.id().to_string();

    let user = User::find_by_id(&mut conn, &user_id)
        .await?
        .ok_or(CurrentUserError::NotFound)?;

    let user_metrics = MetricsTraffic::find_by_user_for_the_month(&mut conn, &user_id).await?;
    let mut api_user = ApiUser::from(user);
    match user_metrics {
        Some(metrics) => api_user = api_user.with_egress(metrics.egress),
        None => api_user = api_user.with_egress(0),
    }
    Ok((StatusCode::OK, Json(api_user)).into_response())
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
        let (status_code, msg) = match self {
            CurrentUserError::DatabaseFailure(_) => {
                tracing::error!("{self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "backend service experienced an issue servicing the request",
                )
            }
            CurrentUserError::NotFound => (StatusCode::NOT_FOUND, "not found"),
        };

        let err_msg = serde_json::json!({"msg": msg});
        (status_code, Json(err_msg)).into_response()
    }
}
