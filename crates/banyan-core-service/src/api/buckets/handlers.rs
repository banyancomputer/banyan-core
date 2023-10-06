use axum::extract::{self, Json, Path};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;
use validify::Validate;

use crate::api::buckets::{keys, requests, responses};
use crate::database::Database;
use crate::error::CoreError;
use crate::extractors::ApiToken;
use crate::utils::db;
use crate::utils::keys::*;

/// Delete a Bucket
pub async fn delete(
    api_token: ApiToken,
    database: Database,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // todo: need to delete all the hot data stored at various storage hosts
    if let Err(err) = db::delete_bucket(&account_id, &bucket_id, &database).await {
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
    database: Database,
    Path(bucket_id): Path<Uuid>,
) -> Response {
    let bucket_id = bucket_id.to_string();

    // Observable usage is sum of data in current state for the requested bucket
    match db::read_bucket_data_usage(&bucket_id, &database).await {
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
pub async fn get_total_usage(api_token: ApiToken, database: Database) -> Response {
    match db::read_total_data_usage(&api_token.subject, &database).await {
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
