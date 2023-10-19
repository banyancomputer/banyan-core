use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiMetadata;
use crate::app::AppState;
use crate::extractors::DataStore;
use crate::database::models::PartialMetadataWithSnapshot;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    store: DataStore,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Response {
    let database = state.database();

    let db_bucket_id = bucket_id.to_string();
    let db_metadata_id = metadata_id.to_string();

    let authorized_bucket_data = sqlx::query_as!(
        PullBucketData,
        r#"SELECT m.bucket_id, m.id as metadata_id FROM metadata AS m
               JOIN buckets AS b ON m.bucket_id = b.id
               WHERE b.account_id = $1 AND b.id = $2 AND m.id = $3;"#,
        api_id.account_id,
        db_bucket_id,
        db_metadata_id,
    )
    .fetch_optional(&database)
    .await
    .unwrap();

    let file_path = object_store::path::Path::from(format!("{bucket_id}/{metadata_id}.car").as_str());

    todo!()
}

#[derive(sqlx::FromRow)]
struct PullBucketData {
    bucket_id: String,
    metadata_id: String,
}
