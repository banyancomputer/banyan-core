use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use validify::{Validate, Validify};

use crate::app::AppState;
use crate::database::models::{BucketType, StorageClass};
use crate::extractors::ApiToken;

/// Initialze a new bucket
pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Json(request): Json<CreateBucketRequest>,
) -> Result<Response, CreateBucketError> {
    request.validate()?;

    let database = state.database();

    let bucket_id = sqlx::query_scalar!(
        r#"INSERT INTO buckets (account_id, name, type, storage_class)
               VALUES ($1, $2, $3, $4);"#,
        api_token.subject,
        request.name,
        request.bucket_type,
        request.storage_class,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateBucketError::BucketCreationFailed)?;

    //            match db::create_bucket_key(
    //                &bucket_resource.id,
    //                true,
    //                &new_bucket.initial_bucket_key_pem,
    //                &database,
    //            )
    //            .await
    //            {
    //                // If we successfully created that too
    //                Ok(key_resource) => {
    //                    // Create a response
    //                    let response = responses::CreateBucket {
    //                        id: bucket_resource.id,
    //                        name: new_bucket.name,
    //                        r#type: new_bucket.r#type,
    //                        storage_class: new_bucket.storage_class,
    //                        initial_bucket_key: keys::responses::CreateBucketKey {
    //                            id: key_resource.id,
    //                            approved: true,
    //                            fingerprint: fingerprint_public_pem(
    //                                &new_bucket.initial_bucket_key_pem,
    //                            ),
    //                        },
    //                    };

    //                    // Return it
    //                    (StatusCode::OK, Json(response)).into_response()
    //                }
    //                Err(err) => CoreError::sqlx_error(err, "create", "bucket key").into_response(),
    //            }

    todo!()
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
    #[error("failed to insert bucket into database: {0}")]
    BucketCreationFailed(sqlx::Error),

    #[error("invalid bucket creation request received: {0}")]
    InvalidBucket(#[from] validify::ValidationErrors),
}

impl IntoResponse for CreateBucketError {
    fn into_response(self) -> Response {
        match self {
            CreateBucketError::InvalidBucket(_) => {
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
