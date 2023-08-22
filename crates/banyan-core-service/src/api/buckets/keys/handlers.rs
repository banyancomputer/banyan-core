use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::keys::{requests, responses};
use crate::db::*;
use crate::extractors::{ApiToken, DbConn};

/// Initialze a new bucket key for the specified bucket
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
    Json(new_bucket_key): extract::Json<requests::CreateBucketKey>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();

    // TODO: Does this need to be authorized?
    // Make sure the calling user owns the bucket
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };

    if let Err(errors) = new_bucket_key.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("errors: {:?}", errors.errors()),
        )
            .into_response();
    };

    let maybe_bucket_key = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem) VALUES ($1, false, $2) RETURNING id;"#,
        bucket_id,
        new_bucket_key.pem,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_bucket_key = match maybe_bucket_key {
        Ok(cbk) => responses::CreateBucketKey {
            id: cbk.id,
            approved: false,
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create new bucket key: {err}"),
            )
                .into_response();
        }
    };

    Json(created_bucket_key).into_response()
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

    // TODO: Does this need to be authorized?
    // Make sure the calling user owns the bucket
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };

    let maybe_bucket_keys = sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem FROM bucket_keys WHERE bucket_id = $1;"#,
        bucket_id,
    )
    .fetch_all(&mut *db_conn.0)
    .await;

    let bucket_keys = match maybe_bucket_keys {
        Ok(bks) => responses::ReadAllBucketKeys(
            bks.into_iter()
                .map(|bk| responses::ReadBucketKey {
                    id: bk.id,
                    approved: bk.approved,
                    pem: bk.pem,
                })
                .collect(),
        ),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket keys: {err}"),
            )
                .into_response();
        }
    };

    Json(bucket_keys).into_response()
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

    // TODO: Does this need to be authorized?
    // Make sure the calling user owns the bucket
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };

    let maybe_bucket_key = sqlx::query_as!(
        models::BucketKey,
        r#"SELECT id, bucket_id, approved, pem FROM bucket_keys WHERE id = $1 AND bucket_id = $2;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let bucket_key = match maybe_bucket_key {
        Ok(bk) => responses::ReadBucketKey {
            id: bk.id,
            approved: bk.approved,
            pem: bk.pem,
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket key: {err}"),
            )
                .into_response();
        }
    };

    Json(bucket_key).into_response()
}

pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();

    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id FROM buckets WHERE id = $1 AND account_id = $2;"#,
        bucket_id,
        account_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_bucket {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };

    let maybe_bucket_key = sqlx::query_as!(
        models::BucketKey,
        r#"DELETE FROM bucket_keys WHERE id = $1 AND bucket_id = $2 RETURNING id, bucket_id, approved, pem;"#,
        bucket_key_id,
        bucket_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let bucket_key = match maybe_bucket_key {
        Ok(bk) => responses::DeleteBucketKey {
            id: bk.id,
            approved: bk.approved,
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to delete bucket key: {err}"),
            )
                .into_response();
        }
    };

    Json(bucket_key).into_response()
}
