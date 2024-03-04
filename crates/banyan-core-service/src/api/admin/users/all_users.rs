use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiUsersAdmin;
use crate::app::AppState;
use crate::database::models::User;
use crate::extractors::AdminIdentity;
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AllUsersQuery {
    pub page: i64,
    pub page_size: i64,
}
pub async fn handler(
    _: AdminIdentity,
    State(state): State<AppState>,
    Query(query): Query<AllUsersQuery>,
) -> Result<Response, AllUsersError> {
    let database = state.database();
    let offset = query.page.saturating_sub(1) * query.page_size;
    let query_result = User::find_all(&database, offset, query.page_size).await?;
    let users: Vec<_> = query_result.into_iter().map(ApiUsersAdmin::from).collect();

    Ok((StatusCode::OK, Json(users)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllUsersError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for AllUsersError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all users: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
