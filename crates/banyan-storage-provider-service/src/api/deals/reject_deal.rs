use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use uuid::Uuid;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, CoreServiceError};

pub async fn handler(
    State(state): State<AppState>,
    Path(deal_id): Path<Uuid>,
) -> Result<Response, CancelDealError> {
    let deal_id = deal_id.to_string();

    let core_service_client = CoreServiceClient::new(
        state.secrets().service_signing_key().clone(),
        state.service_name(),
        state.platform_name(),
        state.platform_hostname(),
    );

    core_service_client.reject_deal(&deal_id).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CancelDealError {
    #[error("failed to call core service: {0}")]
    CoreServiceError(#[from] CoreServiceError),
}
impl IntoResponse for CancelDealError {
    fn into_response(self) -> Response {
        match self {
            CancelDealError::CoreServiceError(err) => match err {
                CoreServiceError::RequestError(_) => {
                    tracing::error!("Internal server error on reject deal: {err}");
                    let err_msg = serde_json::json!({"msg": "Internal server error"});
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
                }
                CoreServiceError::BadRequest(_) => {
                    tracing::error!("Could not reject deal: {err}");
                    let err_msg = serde_json::json!({"msg": "Could not reject deal"});
                    (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
                }
            },
        }
    }
}
