use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::keys::{requests, responses};
use crate::db::*;
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
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    }

    if let Err(errors) = new_bucket_key.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("invalid bucket key: {:?}", errors.errors()),
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
            tracing::error!("unable to create new bucket key: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("internal server error"),
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
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    }
    let response = match db::read_all_bucket_keys(&bucket_id, &mut db_conn).await {
        Ok(bucket_keys) => responses::ReadAllBucketKeys(
            bucket_keys
                .into_iter()
                .map(|bucket_key| responses::ReadBucketKey {
                    id: bucket_key.id,
                    approved: bucket_key.approved,
                    pem: bucket_key.pem,
                })
                .collect(),
        ),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("bucket keys not found: {err}"),
                )
                    .into_response();
            }
            _ => {
                tracing::error!("unable to read bucket keys: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    };
    Json(response).into_response()
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
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    }
    let response = match db::read_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => responses::ReadBucketKey {
            id: bucket_key.id,
            approved: bucket_key.approved,
            pem: bucket_key.pem,
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("bucket key not found: {err}"),
                )
                    .into_response();
            }
            _ => {
                tracing::error!("unable to read bucket key: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    };
    Json(response).into_response()
}

pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    }
    let response = match db::delete_bucket_key(&bucket_id, &bucket_key_id, &mut db_conn).await {
        Ok(bucket_key) => responses::DeleteBucketKey {
            id: bucket_key.id,
            approved: bucket_key.approved,
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("bucket key not found: {err}"),
                )
                    .into_response();
            }
            _ => {
                tracing::error!("unable to delete bucket key: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("internal server error"),
                )
                    .into_response();
            }
        },
    };
    Json(response).into_response()
}
