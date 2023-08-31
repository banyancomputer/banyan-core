use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::{keys, requests, responses};
use crate::db::*;
use crate::extractors::{ApiToken, DbConn};
use crate::utils::db;

/// Initialze a new bucket with initial key material.
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Json(new_bucket): extract::Json<requests::CreateBucket>,
) -> impl IntoResponse {
    if let Err(errors) = new_bucket.validate() {
        return (
            StatusCode::BAD_REQUEST,
            format!("invalid bucket creation request: {:?}", errors.errors()),
        )
            .into_response();
    }

    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO buckets (account_id, name, type, storage_class) VALUES ($1, $2, $3, $4) RETURNING id;"#,
        api_token.subject,
        new_bucket.name,
        new_bucket.r#type,
        new_bucket.storage_class,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_bucket = match maybe_bucket {
        Ok(cb) => cb,
        Err(err) => match err {
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    return (
                        StatusCode::CONFLICT,
                        "bucket with that name already exists".to_string(),
                    )
                        .into_response();
                } else {
                    tracing::error!("unable to create bucket: {db_err}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal server error".to_string(),
                    )
                        .into_response();
                }
            }
            _ => {
                tracing::error!("unable to create bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        },
    };

    let maybe_key = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO bucket_keys (bucket_id, approved, pem) VALUES ($1, true, $2) RETURNING id;"#,
        created_bucket.id,
        new_bucket.initial_bucket_key_pem,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let created_key = match maybe_key {
        Ok(ck) => ck,
        Err(err) => {
            tracing::error!("unable to create bucket key: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response();
        }
    };

    let response = responses::CreateBucket {
        id: created_bucket.id,
        name: new_bucket.name,
        r#type: new_bucket.r#type,
        storage_class: new_bucket.storage_class,
        initial_bucket_key: keys::responses::CreateBucketKey {
            id: created_key.id,
            approved: true,
        },
    };

    (StatusCode::OK, axum::Json(response)).into_response()
}

// TODO: pagination
/// Read all buckets associated with the calling account
pub async fn read_all(api_token: ApiToken, mut db_conn: DbConn) -> impl IntoResponse {
    let account_id = api_token.subject;
    let response = match db::read_all_buckets(&account_id, &mut db_conn).await {
        Ok(buckets) => responses::ReadBuckets(
            buckets
                .into_iter()
                .map(|bucket| responses::ReadBucket {
                    id: bucket.id,
                    name: bucket.name,
                    r#type: bucket.r#type,
                    storage_class: bucket.storage_class,
                })
                .collect::<Vec<_>>(),
        ),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        },
    };
    Json(response).into_response()
}

// TODO: Should this be authenticated or not?
/// Read a single bucket by id. Also search and return by account id
pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let response = match db::read_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(bucket) => responses::ReadBucket {
            id: bucket.id,
            name: bucket.name,
            r#type: bucket.r#type,
            storage_class: bucket.storage_class,
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
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
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let response = match db::delete_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(bucket) => responses::DeleteBucket {
            id: bucket.id,
            name: bucket.name,
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to delete bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        },
    };

    Json(response).into_response()
}

/// Return the current DATA usage for the bucket. Query metadata in the current state of the bucket
pub async fn get_usage(
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
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        },
    }

    // Observable usage is sum of data in current state for the requested bucket
    let response = match db::read_bucket_data_usage(&bucket_id, &mut db_conn).await {
        Ok(usage) => responses::GetUsage { size: usage },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        },
    };
    Json(response).into_response()
}

/// Return the current DATA usage for the account. Query metadata in the current state of the account
pub async fn get_total_usage(api_token: ApiToken, mut db_conn: DbConn) -> impl IntoResponse {
    let account_id = api_token.subject;
    let response = match db::read_total_data_usage(&account_id, &mut db_conn).await {
        Ok(usage) => responses::GetUsage { size: usage },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        },
    };
    Json(response).into_response()
}

pub async fn get_usage_limit(_api_token: ApiToken) -> impl IntoResponse {
    Json(responses::GetUsage {
        // 5 TiB
        size: 5 * 1024 * 1024 * 1024 * 1024,
    })
    .into_response()
}
