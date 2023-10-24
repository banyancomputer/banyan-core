use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucket;
use crate::app::AppState;
use crate::database::models::Bucket;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, AuthorizationGrantError> {
    let database = state.database();

    let bucket_id = bucket_id.to_string();

    let authorized_amounts = sqlx::query_as!(
        AuthorizedAmounts,
        r#"WITH current_grants AS (
               SELECT storage_host_id, account_id, MAX(redeemed_at) AS most_recently_redeemed_at
                   FROM storage_grants
                   GROUP BY storage_host_id, account_id
           )
           SELECT sg.authorized_amount, sh.url as storage_host_url
               FROM buckets AS b
               JOIN metadata AS m ON m.bucket_id = b.id
               JOIN storage_hosts_metadatas_storage_grants AS shms ON shms.metadata_id = m.id
               JOIN storage_hosts AS sh ON sh.id = shms.storage_host_id
               JOIN storage_grants AS sg ON sg.id = shms.storage_grant_id
               JOIN current_grants AS cg ON cg.storage_host_id = sh.id
                   AND cg.account_id = sg.account_id
                   AND cg.most_recently_redeemed_at = sg.redeemed_at
               WHERE b.account_id = $1
                   AND b.id = $2
                   AND m.state NOT IN ('deleted', 'upload_failed');"#,
        api_id.account_id,
        bucket_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AuthorizationGrantError::LookupFailed)?;

    todo!();
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorizationGrantError {
    #[error("failed to locate authorization grants: {0}")]
    LookupFailed(sqlx::Error),
}

impl IntoResponse for AuthorizationGrantError {
    fn into_response(self) -> Response {
        todo!()
    }
}

#[derive(sqlx::FromRow)]
struct AuthorizedAmounts {
    authorized_amount: i64,
    storage_host_url: String,
}
