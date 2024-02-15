use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::storage_ticket::StorageTicketBuilder;
use crate::database::models::AuthorizedAmounts;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, AuthorizationGrantError> {
    let database = state.database();
    let service_key = state.secrets().service_key();
    let bucket_id = bucket_id.to_string();
    let user_id = api_id.user_id().to_string();

    let mut conn = database.begin().await?;

    let authorized_amounts = AuthorizedAmounts::lookup(&mut conn, &user_id, &bucket_id).await?;

    if authorized_amounts.is_empty() {
        return Err(AuthorizationGrantError::NotFound);
    }

    let claims =
        StorageTicketBuilder::from_authorized_amounts(api_id.ticket_subject(), authorized_amounts)
            .build();

    let bearer_token = service_key
        .sign(claims)
        .map_err(AuthorizationGrantError::SigningFailed)?;

    let resp_msg = serde_json::json!({"authorization_token": bearer_token});
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationGrantError {
    #[error("failed to locate authorization grants: {0}")]
    LookupFailed(#[from] sqlx::Error),

    #[error("no grants found associated with bucket and account")]
    NotFound,

    #[error("failed to sign new grant: {0}")]
    SigningFailed(jwt_simple::Error),
}

impl IntoResponse for AuthorizationGrantError {
    fn into_response(self) -> Response {
        match &self {
            AuthorizationGrantError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
