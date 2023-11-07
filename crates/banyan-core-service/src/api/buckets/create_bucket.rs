use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;
use validify::{Validate, Validify};

use crate::app::AppState;
use crate::database::models::{Bucket, BucketKey, BucketType, StorageClass};
use crate::extractors::{Identity, UserIdentity};
use crate::utils::keys::sha1_fingerprint_publickey;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Json(request): Json<CreateBucketRequest>,
) -> Result<Response, CreateBucketError> {
    request.validate()?;

    // todo: should probably move this validation into the validate() call...
    let public_key = ES384PublicKey::from_pem(&request.initial_bucket_key_pem)
        .map_err(CreateBucketError::InvalidPublicKey)?;
    let fingerprint = sha1_fingerprint_publickey(&public_key);

    let database = state.database();

    let user_id = user_id.user_id();
    let bucket_id = sqlx::query_scalar!(
        r#"INSERT INTO buckets (user_id, name, type, storage_class)
               VALUES ($1, $2, $3, $4)
               RETURNING id;"#,
        user_id,
        request.name,
        request.bucket_type,
        request.storage_class,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketError::BucketCreationFailed)?;

    // todo: when the extra returns have been removed this can turn into an execute query, for now
    // we need to keep a handle on the id

    let bucket_key_id = sqlx::query_scalar!(
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem, fingerprint)
               VALUES ($1, true, $2, $3)
               RETURNING id;"#,
        bucket_id,
        request.initial_bucket_key_pem,
        fingerprint,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketError::BucketKeyCreationFailed)?;

    let bucket = sqlx::query_as!(Bucket, "SELECT * FROM buckets WHERE id = $1;", bucket_id)
        .fetch_one(&database)
        .await
        .expect("(temp query, no custom error, just needs refactor)");

    let bucket_key = sqlx::query_as!(
        BucketKey,
        "SELECT * FROM bucket_keys WHERE id = $1;",
        bucket_key_id
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketError::AdditionalDetailsUnavailable)?;

    let resp = serde_json::json!({
        "id": bucket.id,
        "name": bucket.name,
        "type": bucket.r#type,
        "storage_class": bucket.storage_class,
        "initial_bucket_key": {
            "id": bucket_key.id,
            "approved": bucket_key.approved,
            "fingerprint": bucket_key.fingerprint,
        },
    });

    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateBucketRequest {
    #[validate(length(min = 3, max = 32))]
    name: String,

    #[serde(rename = "type")]
    bucket_type: BucketType,
    storage_class: StorageClass,

    initial_bucket_key_pem: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBucketError {
    #[error("retrieving additional bucket details failed: {0}")]
    AdditionalDetailsUnavailable(sqlx::Error),

    #[error("failed to insert bucket into database: {0}")]
    BucketCreationFailed(sqlx::Error),

    #[error("failed to insert bucket key into database: {0}")]
    BucketKeyCreationFailed(sqlx::Error),

    #[error("invalid bucket creation request received: {0}")]
    InvalidBucket(#[from] validify::ValidationErrors),

    #[error("provided public key was not valid: {0}")]
    InvalidPublicKey(jwt_simple::Error),
}

impl IntoResponse for CreateBucketError {
    fn into_response(self) -> Response {
        use CreateBucketError as CBE;

        match self {
            CBE::InvalidBucket(_) | CBE::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": "{self}"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
