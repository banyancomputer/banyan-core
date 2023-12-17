use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiDeal;
use crate::app::AppState;
use crate::database::models::Deal;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    _storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllDealsError> {
    let database = state.database();
    let query_result = sqlx::query_as!(Deal, "SELECT * FROM deals where state='active';")
        .fetch_all(&database)
        .await
        .map_err(AllDealsError::DatabaseFailure)?;

    let deals: Vec<_> = query_result.into_iter().map(ApiDeal::from).collect();

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
