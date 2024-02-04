use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiDealsAdmin;
use crate::app::AppState;
use crate::database::models::Deal;
use crate::extractors::AdminIdentity;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DealQuery {
    pub status: Option<String>,
}

pub async fn handler(
    _: AdminIdentity,
    State(state): State<AppState>,
    Query(_payload): Query<DealQuery>,
) -> Result<Response, AllDealsError> {
    let database = state.database();
    let query_result = sqlx::query_as!(
        Deal,
        r#"SELECT d.id, d.state, SUM(ss.size) AS size, accepted_by, accepted_at
        FROM deals d
            JOIN snapshot_segments ss ON d.id = ss.deal_id
        GROUP BY d.id;"#,
    )
    .fetch_all(&database)
    .await
    .map_err(AllDealsError::DatabaseFailure)?;

    let deals: Vec<_> = query_result.into_iter().map(ApiDealsAdmin::from).collect();

    Ok((StatusCode::OK, Json(deals)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllDealsError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllDealsError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all deals: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
