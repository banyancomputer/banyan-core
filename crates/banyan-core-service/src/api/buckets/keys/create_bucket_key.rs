use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;
use crate::utils::keys::sha1_fingerprint_publickey;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
    Json(bucket_key_req): Json<CreateBucketKeyRequest>,
) -> Result<Response, CreateBucketKeyError> {
    let public_key = ES384PublicKey::from_pem(&bucket_key_req.public_key)
        .map_err(CreateBucketKeyError::InvalidPublicKey)?;
    let fingerprint = sha1_fingerprint_publickey(&public_key);

    let database = state.database();
    let bucket_id = bucket_id.to_string();

    // we need to authorize the bucket belongs to the account before we associate the key to the
    // bucket
    let maybe_bucket_id: Option<String> = sqlx::query_scalar!(
        "SELECT id FROM buckets WHERE account_id = $1 AND id = $2;",
        api_id.account_id,
        bucket_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(CreateBucketKeyError::DatabaseFailure)?;

    let authorized_bucket_id = match maybe_bucket_id {
        Some(abi) => abi,
        None => {
            return Err(CreateBucketKeyError::NoAuthorizedBucket);
        }
    };

    let bucket_key_id: String = sqlx::query_scalar!(
        r#"INSERT INTO bucket_keys (bucket_id, pem, fingerprint, approved)
               VALUES ($1, $2, $3, 'false')
               RETURNING id;"#,
        authorized_bucket_id,
        bucket_key_req.public_key,
        fingerprint,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketKeyError::DatabaseFailure)?;

    let resp_msg = serde_json::json!({
        "id": bucket_key_id,
        "approved": false,
        "fingerprint": fingerprint,
    });

    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBucketKeyError {
    #[error("failed to call the database: {0}")]
    DatabaseFailure(sqlx::Error),

    #[error("provided public key was not valid: {0}")]
    InvalidPublicKey(jwt_simple::Error),

    #[error("bucket either doesn't exist or is unauthorized")]
    NoAuthorizedBucket,
}

impl IntoResponse for CreateBucketKeyError {
    fn into_response(self) -> Response {
        match &self {
            CreateBucketKeyError::NoAuthorizedBucket => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("failed to lookup bucket key: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct CreateBucketKeyRequest {
    #[serde(rename = "pem")]
    pub public_key: String,
}
