use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::{keys, requests, responses};
use crate::error::CoreError;
use crate::extractors::{ApiToken, DbConn};
use crate::utils::db;

/// Initialze a new bucket with initial key material.
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Json(new_bucket): extract::Json<requests::CreateBucket>,
) -> impl IntoResponse {
    if let Err(errors) = new_bucket.validate() {
        (
            StatusCode::BAD_REQUEST,
            format!("invalid bucket creation request: {:?}", errors.errors()),
        )
            .into_response()
    } else {
        // Create the Bucket
        match db::create_bucket(
            &api_token.subject,
            &new_bucket.name,
            &new_bucket.r#type,
            &new_bucket.storage_class,
            &mut db_conn,
        )
        .await
        {
            // If we successfully created the resource
            Ok(bucket_resource) => {
                // Create the initial Bucket Key
                match db::create_bucket_key(
                    &bucket_resource.id,
                    &new_bucket.initial_bucket_key_pem,
                    &mut db_conn,
                )
                .await
                {
                    // If we successfully created that too
                    Ok(key_resource) => {
                        // Create a response
                        let response = responses::CreateBucket {
                            id: bucket_resource.id,
                            name: new_bucket.name,
                            r#type: new_bucket.r#type,
                            storage_class: new_bucket.storage_class,
                            initial_bucket_key: keys::responses::CreateBucketKey {
                                id: key_resource.id,
                                approved: true,
                            },
                        };

                        // Return it
                        (StatusCode::OK, Json(response)).into_response()
                    }
                    Err(err) => CoreError::sqlx_error(err, "create", "bucket key").into_response(),
                }
            }
            Err(err) => CoreError::sqlx_error(err, "create", "bucket").into_response(),
        }
    }
}

// TODO: pagination
/// Read all buckets associated with the calling account
pub async fn read_all(api_token: ApiToken, mut db_conn: DbConn) -> impl IntoResponse {
    let account_id = api_token.subject;
    match db::read_all_buckets(&account_id, &mut db_conn).await {
        Ok(buckets) => Json(responses::ReadBuckets(
            buckets
                .into_iter()
                .map(|bucket| responses::ReadBucket {
                    id: bucket.id,
                    name: bucket.name,
                    r#type: bucket.r#type,
                    storage_class: bucket.storage_class,
                })
                .collect::<Vec<_>>(),
        ))
        .into_response(),
        Err(err) => CoreError::sqlx_error(err, "read", "all buckets").into_response(),
    }
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
    match db::read_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(bucket) => Json(responses::ReadBucket {
            id: bucket.id,
            name: bucket.name,
            r#type: bucket.r#type,
            storage_class: bucket.storage_class,
        })
        .into_response(),
        Err(err) => CoreError::sqlx_error(err, "read", "bucket").into_response(),
    }
}

/// Delete a Bucket
pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    match db::delete_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(bucket) => Json(responses::DeleteBucket {
            id: bucket.id,
            name: bucket.name,
        })
        .into_response(),
        Err(err) => CoreError::sqlx_error(err, "delete", "bucket").into_response(),
    }
}

/// Return the current DATA usage for the bucket. Query metadata in the current state of the bucket
pub async fn get_usage(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let bucket_id = bucket_id.to_string();
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
