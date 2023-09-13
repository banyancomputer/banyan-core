use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::{keys, requests, responses};
use crate::error::CoreError;
use crate::extractors::{ApiToken, DbConn};
use crate::utils::db;
use crate::utils::keys::*;

/// Initialze a new bucket with initial key material.
pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Json(new_bucket): extract::Json<requests::CreateBucket>,
) -> Response {
    if let Err(errors) = new_bucket.validate() {
        GenericError::new(
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
                    true,
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
                                fingerprint: fingerprint_public_pem(
                                    &new_bucket.initial_bucket_key_pem,
                                ),
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
pub async fn read_all(api_token: ApiToken, mut db_conn: DbConn) -> Response {
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
        Err(err) => {
            tracing::error!("failed to read all buckets: {err}");
            GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
                .into_response()
        }
    }
}

// TODO: Should this be authenticated or not?
/// Read a single bucket by id. Also search and return by account id
pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> Response {
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
        Err(err) => {
            tracing::error!("unable to delete bucket: {err}");
            GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
                .into_response()
        }
    }
}

/// Delete a Bucket
pub async fn delete(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // todo: need to delete all the hot data stored at various storage hosts
    if let Err(err) = db::delete_bucket(&account_id, &bucket_id, &mut db_conn).await {
        tracing::error!("failed to delete bucket: {err}");
        GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
            .into_response()
    } else {
        (StatusCode::NO_CONTENT, ()).into_response()
    }
}

/// Return the current DATA usage for the bucket. Query metadata in the current state of the bucket
pub async fn get_usage(
    _api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let bucket_id = bucket_id.to_string();

    // Observable usage is sum of data in current state for the requested bucket
    match db::read_bucket_data_usage(&bucket_id, &mut db_conn).await {
        Ok(usage) => Json(responses::GetUsage { size: usage }).into_response(),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                GenericError::new(StatusCode::NOT_FOUND, "bucket not found").into_response()
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
                    .into_response()
            }
        },
    }
}

/// Return the current DATA usage for the account. Query metadata in the current state of the account
pub async fn get_total_usage(api_token: ApiToken, mut db_conn: DbConn) -> Response {
    match db::read_total_data_usage(&api_token.subject, &mut db_conn).await {
        Ok(usage) => Json(responses::GetUsage { size: usage }).into_response(),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                GenericError::new(StatusCode::NOT_FOUND, "bucket not found").into_response()
            }
            _ => {
                tracing::error!("unable to read bucket: {err}");
                GenericError::new(StatusCode::INTERNAL_SERVER_ERROR, "backend service issue")
                    .into_response()
            }
        },
    }
}

pub async fn get_usage_limit(_api_token: ApiToken) -> Response {
    Json(responses::GetUsage {
        // 5 TiB
        size: 5 * 1024 * 1024 * 1024 * 1024,
    })
    .into_response()
}

pub struct GenericError {
    code: StatusCode,
    msg: String,
}

impl GenericError {
    pub fn new(code: StatusCode, msg: impl ToString) -> Self {
        Self {
            code,
            msg: msg.to_string(),
        }
    }
}

impl IntoResponse for GenericError {
    fn into_response(self) -> Response {
        (self.code, Json(serde_json::json!({"msg": self.msg}))).into_response()
    }
}
