use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::algorithms::ECDSAP384KeyPairLike;
use serde::Serialize;

use crate::app::AppState;
use crate::auth::storage_ticket::StorageTicketBuilder;
use crate::auth::STAGING_SERVICE_NAME;
use crate::database::models::StorageHost;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(storage_host_id): Path<String>,
) -> Result<Response, ProviderGrantError> {
    let database = state.database();
    let service_key = state.secrets().service_key();

    if storage_provider.name != STAGING_SERVICE_NAME {
        return Err(ProviderGrantError::Unauthorized);
    }
    let request_host = StorageHost::get_by_id(&database, storage_host_id.as_str())
        .await
        .map_err(ProviderGrantError::LookupFailed)?;

    let mut ticket_builder = StorageTicketBuilder::new(STAGING_SERVICE_NAME.to_string());
    ticket_builder.add_audience(request_host.name);
    ticket_builder.add_authorization(request_host.id, request_host.url.clone(), 0);
    let claim = ticket_builder.build();
    let bearer_token = service_key.sign(claim)?;

    let response = ProviderGrantResponse {
        token: bearer_token,
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}
#[derive(Serialize)]
struct ProviderGrantResponse {
    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderGrantError {
    #[error("not authorized to move metadata")]
    Unauthorized,

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("database lookup failed: {0}")]
    LookupFailed(sqlx::Error),
}

impl IntoResponse for ProviderGrantError {
    fn into_response(self) -> Response {
        match self {
            ProviderGrantError::Unauthorized => {
                let err_msg = serde_json::json!({"msg": "not authorized"});
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
            ProviderGrantError::LookupFailed(_) => {
                let err_msg = serde_json::json!({"msg": "metadata not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            ProviderGrantError::JwtError(_) => {
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
