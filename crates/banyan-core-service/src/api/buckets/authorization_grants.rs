use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::storage_ticket::StorageTicketBuilder;
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
    let authorized_amounts = sqlx::query_as!(
        AuthorizedAmounts,
        r#"WITH current_grants AS (
            SELECT id, storage_host_id, user_id, MAX(redeemed_at) AS most_recently_redeemed_at
            FROM storage_grants
            WHERE redeemed_at IS NOT NULL AND user_id = $1
            GROUP BY storage_host_id, user_id
        )
            SELECT sg.id AS storage_grant_id, sg.authorized_amount, sh.name AS storage_host_name, sh.url AS storage_host_url
                FROM current_grants AS cg
                JOIN storage_hosts_metadatas_storage_grants AS shms ON shms.storage_grant_id = cg.id
                JOIN storage_hosts AS sh ON sh.id = shms.storage_host_id
                JOIN metadata AS m ON m.id = shms.metadata_id
                JOIN buckets AS b ON b.id = m.bucket_id
                JOIN storage_grants AS sg ON sg.id = cg.id
                WHERE b.user_id = $1
                    AND b.id = $2
                    AND b.deleted_at IS NULL
                    AND m.state NOT IN ('deleted', 'upload_failed');"#,
        user_id,
        bucket_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AuthorizationGrantError::LookupFailed)?;

    if authorized_amounts.is_empty() {
        return Err(AuthorizationGrantError::NotFound);
    }

    let mut ticket_builder = StorageTicketBuilder::new(api_id.ticket_subject());

    for auth_details in authorized_amounts.into_iter() {
        ticket_builder.add_audience(auth_details.storage_host_name);
        ticket_builder.add_authorization(
            auth_details.storage_grant_id,
            auth_details.storage_host_url,
            auth_details.authorized_amount,
        );
    }

    let claims = ticket_builder.build();

    let bearer_token = service_key
        .sign(claims)
        .map_err(AuthorizationGrantError::SigningFailed)?;

    let resp_msg = serde_json::json!({"authorization_token": bearer_token});
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(sqlx::FromRow, Debug)]
struct AuthorizedAmounts {
    authorized_amount: i64,
    storage_grant_id: String,
    storage_host_name: String,
    storage_host_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationGrantError {
    #[error("failed to locate authorization grants: {0}")]
    LookupFailed(sqlx::Error),

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
