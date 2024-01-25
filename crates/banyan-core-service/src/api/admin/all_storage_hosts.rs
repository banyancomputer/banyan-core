use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::api::models::ApiSelectedStorageHostAdmin;
use crate::app::AppState;
use crate::database::models::SelectedStorageHost;
use crate::extractors::AdminIdentity;

pub async fn handler(
    _: AdminIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllStorageHostsError> {
    let database = state.database();
    let query_result = sqlx::query_as!(
        SelectedStorageHost,
        r#"SELECT id,name,url,used_storage, available_storage,fingerprint,pem FROM storage_hosts;"#,
    )
    .fetch_all(&database)
    .await
    .map_err(AllStorageHostsError::DatabaseFailure)?;

    let hosts: Vec<_> = query_result
        .into_iter()
        .map(ApiSelectedStorageHostAdmin::from)
        .collect();

    Ok((StatusCode::OK, Json(hosts)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllStorageHostsError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllStorageHostsError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all deals: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
