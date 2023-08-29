use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::keys::{requests, responses};
use crate::db::*;
use crate::extractors::{ApiToken, DbConn};
use crate::utils::db::{self, sqlx_error_to_response};

/// Initialze a new bucket key for the specified bucket
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
    Json(new_bucket_key): extract::Json<requests::CreateBucketKey>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // Ensure this Account is allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return sqlx_error_to_response(err, "read", "bucket");
    }

    if let Err(errors) = new_bucket_key.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("invalid bucket key: {:?}", errors.errors()),
        )
            .into_response();
    };

    // Try to insert the new bucket key
    let maybe_bucket_key = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem) VALUES ($1, false, $2) RETURNING id;"#,
        bucket_id,
        new_bucket_key.pem,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_bucket_key {
        Ok(cbk) => Json(responses::CreateBucketKey {
            id: cbk.id,
            approved: false,
        })
        .into_response(),
        Err(err) => sqlx_error_to_response(err, "create", "new bucket key"),
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
    // Ensure this Account is allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return sqlx_error_to_response(err, "read", "bucket");
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
        Err(err) => sqlx_error_to_response(err, "read_all", "bucket keys"),
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
    // Ensure this Account is allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return sqlx_error_to_response(err, "read", "bucket");
    }

    // Try to read the Bucket Key, respond based on success
    match db::read_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => Json(responses::ReadBucketKey {
            id: bucket_key.id,
            approved: bucket_key.approved,
            pem: bucket_key.pem,
        })
        .into_response(),
        Err(err) => sqlx_error_to_response(err, "read", "bucket key"),
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
    // Ensure this Account is allowed to read this Bucket
    if let Err(err) = db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        // Return error response if not
        return sqlx_error_to_response(err, "read", "bucket");
    }
    // Try to delete the Bucket Key, respond based on success
    match db::delete_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => Json(responses::DeleteBucketKey {
            id: bucket_key.id,
            approved: bucket_key.approved,
        })
        .into_response(),
        Err(err) => sqlx_error_to_response(err, "delete", "bucket key"),
    }
}
