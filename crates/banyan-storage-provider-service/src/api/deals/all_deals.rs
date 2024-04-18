use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, CoreServiceError};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DealQuery {
    pub status: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    Query(query): Query<DealQuery>,
) -> Result<Response, AllDealsError> {
    let core_service_client = CoreServiceClient::new(
        state.secrets().service_signing_key().clone(),
        state.service_name(),
        state.platform_name(),
        state.platform_hostname(),
    );

    let deals = core_service_client.get_all_deals(query).await?;

    Ok((StatusCode::OK, Json(deals)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllDealsError {
    #[error("failed to call core service: {0}")]
    CoreServiceError(#[from] CoreServiceError),
}

impl IntoResponse for AllDealsError {
    fn into_response(self) -> Response {
        match self {
            AllDealsError::CoreServiceError(err) => match err {
                CoreServiceError::RequestError(_) => {
                    tracing::error!("Internal server error on looking up all deals: {err}");
                    let err_msg = serde_json::json!({"msg": "Internal server error"});
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
                }
                CoreServiceError::BadRequest(_) => {
                    tracing::error!("Cold not retrieve all deals: {err}");
                    let err_msg = serde_json::json!({"msg": "Could not retrieve all deals"});
                    (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
                }
            },
        }
    }
}
