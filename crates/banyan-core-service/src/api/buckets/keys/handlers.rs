use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::keys::{requests, responses};
use crate::error::CoreError;
use crate::extractors::{ApiToken, DbConn};
use crate::utils::db;

/// Initialze a new bucket key for the specified bucket
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
    Json(new_bucket_key): extract::Json<requests::CreateBucketKey>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // If this Account is not allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return CoreError::sqlx_error(err, "read", "bucket").into_response();
    }
    // If the new Bucket Key is not valid
    if let Err(errors) = new_bucket_key.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("invalid bucket key: {:?}", errors.errors()),
        )
            .into_response();
    }

    // Create the Bucket Key
    match db::create_bucket_key(&bucket_id, &new_bucket_key.pem, &mut db_conn).await {
        Ok(resource) => Json(responses::CreateBucketKey {
            id: resource.id,
            approved: false,
        })
        .into_response(),
        Err(err) => CoreError::sqlx_error(err, "create", "bucket key").into_response(),
    }
}

// TODO: pagination
/// List all bucket keys for the specified bucket
pub async fn read_all(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // If this Account is not allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return CoreError::sqlx_error(err, "read", "bucket").into_response();
    }

    // Try to read all the Bucket Keys, respond based on success
    match db::read_all_bucket_keys(&bucket_id, &mut db_conn).await {
        Ok(bucket_keys) => Json(responses::ReadAllBucketKeys(
            bucket_keys
                .into_iter()
                .map(|bucket_key| responses::ReadBucketKey {
                    id: bucket_key.id,
                    approved: bucket_key.approved,
                    pem: bucket_key.pem,
                })
                .collect(),
        ))
        .into_response(),
        Err(err) => CoreError::sqlx_error(err, "read all", "bucket keys").into_response(),
    }
}

/// Read a specific bucket key for the specified bucket
pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();
    // If this Account is not allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return CoreError::sqlx_error(err, "read", "bucket").into_response();
    }
    // Try to read the Bucket Key, respond based on success
    match db::read_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => Json(responses::ReadBucketKey {
            id: bucket_key.id,
            approved: bucket_key.approved,
            pem: bucket_key.pem,
        })
        .into_response(),
        Err(err) => CoreError::sqlx_error(err, "read", "bucket key").into_response(),
    }
}

pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();
    // If this Account is not allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return CoreError::sqlx_error(err, "read", "bucket").into_response();
    }
    // Try to delete the Bucket Key, respond based on success
    match db::delete_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => Json(responses::DeleteBucketKey {
            id: bucket_key.id,
            approved: bucket_key.approved,
        })
        .into_response(),
        Err(err) => return CoreError::sqlx_error(err, "delete", "bucket key").into_response(),
    }
}

/// Reject a Bucket Key, deleting it in the process
pub async fn reject(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();
    // If this Account is not allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return CoreError::sqlx_error(err, "read", "bucket").into_response();
    }

    // If we can successfully read the key from the database
    match db::read_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => {
            // If this Bucket Key is already approved
            if bucket_key.approved {
                // Tell the user to call Delete instead
                (
                    StatusCode::BAD_REQUEST,
                    "Bucket Key is already approved. Delete it to reject it.".to_string(),
                )
                    .into_response()
            } else {
                // Delete the key
                match db::delete_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
                    Ok(bucket_key) => Json(responses::DeleteBucketKey {
                        id: bucket_key.id,
                        approved: bucket_key.approved,
                    })
                    .into_response(),
                    Err(err) => {
                        return CoreError::sqlx_error(err, "reject", "bucket key").into_response()
                    }
                }
            }
        }
        Err(err) => CoreError::sqlx_error(err, "read", "bucket key").into_response(),
    }
}
