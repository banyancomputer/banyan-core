use axum::body::StreamBody;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;
use crate::extractors::DataStore;

pub async fn handler(
    api_id: ApiIdentity,
    store: DataStore,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Response, PullMetadataError> {
    let database = state.database();

    let db_bucket_id = bucket_id.to_string();
    let db_metadata_id = metadata_id.to_string();

    let authorized_bucket_data = sqlx::query_as!(
        PullBucketData,
        r#"SELECT m.bucket_id, m.id as metadata_id FROM metadata AS m
               JOIN buckets AS b ON m.bucket_id = b.id
               WHERE b.user_id = $1 AND b.id = $2 AND m.id = $3;"#,
        api_id.user_id,
        db_bucket_id,
        db_metadata_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(PullMetadataError::MetadataUnavailable)?
    .ok_or(PullMetadataError::NotFound)?;

    let file_name = format!(
        "{}/{}.car",
        authorized_bucket_data.bucket_id, authorized_bucket_data.metadata_id,
    );
    let file_path = object_store::path::Path::from(file_name.as_str());

    let file_reader = store
        .get(&file_path)
        .await
        .map_err(PullMetadataError::FileUnavailable)?;
    let stream = file_reader.into_stream();

    let mut headers = HeaderMap::new();

    let disposition =
        HeaderValue::from_str(format!("attachment; filename=\"{file_name}\"").as_str()).unwrap();
    headers.insert(CONTENT_DISPOSITION, disposition);
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.ipld.car; version=2"),
    );

    let body = StreamBody::new(stream);

    Ok((StatusCode::OK, headers, body).into_response())
}

#[derive(sqlx::FromRow)]
struct PullBucketData {
    bucket_id: String,
    metadata_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PullMetadataError {
    #[error("unable to get a handle on metadata file: {0}")]
    FileUnavailable(object_store::Error),

    #[error("the database reported an issue when attempting to locate metadata: {0}")]
    MetadataUnavailable(sqlx::Error),

    #[error("no matching metadata for the current account")]
    NotFound,
}

impl IntoResponse for PullMetadataError {
    fn into_response(self) -> Response {
        match &self {
            PullMetadataError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error retrieving metadata: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
