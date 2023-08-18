use axum::body::StreamBody;
use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use uuid::Uuid;

use crate::api::buckets::snapshots::{requests, responses};
use crate::db::models;
use crate::extractors::{ApiToken, DataStore, DbConn};

pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
    Json(new_snapshot): Json<requests::CreateSnapshotRequest>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    // Make sure the calling user owns the bucket
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
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

    // Make sure the snapshot belongs to the bucket
    let maybe_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        new_snapshot.metadata_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    match maybe_metadata {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read snapshot: {err}"),
            )
                .into_response();
        }
    };

    // Create a new snapshot
    let maybe_snapshot = sqlx::query_as!(
        models::CreatedResource,
        r#"INSERT INTO snapshots (metadata_id)
        VALUES ($1)
        RETURNING id as "id!";"#,
        new_snapshot.metadata_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let snapshot = match maybe_snapshot {
        Ok(s) => responses::CreateSnapshotResponse {
            id: s.id.to_string(),
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create snapshot: {err}"),
            )
                .into_response();
        }
    };

    Json(snapshot).into_response()
}

/// Pull the metadata for a snapshot
pub async fn pull(
    api_token: ApiToken,
    mut db_conn: DbConn,
    store: DataStore,
    Path((bucket_id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let snapshot_id = snapshot_id.to_string();
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
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

    let maybe_snapshot = sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id as "id!",
            s.metadata_id as "metadata_id!",
            s.deal_id as "deal_id!",
            m.created_at as "created_at!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            s.id = $1 AND m.bucket_id = $2;"#,
        snapshot_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let snapshot = match maybe_snapshot {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read snapshot: {err}"),
            )
                .into_response();
        }
    };

    // Try opening the file for reading
    let file_name = format!(
        "{bucket_id}/{metadata_id}.car",
        bucket_id = bucket_id,
        metadata_id = snapshot.metadata_id
    );
    let file_path = object_store::path::Path::from(file_name.as_str());
    let reader = match store.get(&file_path).await {
        Ok(r) => r,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read bucket: {err}"),
            )
                .into_response();
        }
    };
    let stream = reader.into_stream();

    // Create the headers for the response
    let mut headers = HeaderMap::new();

    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.ipld.car; version=2"),
    );
    headers.insert(
        http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(format!("attachment; filename=\"{file_name}\"").as_str()).unwrap(),
    );

    let body = StreamBody::new(stream);

    (StatusCode::OK, headers, body).into_response()
}

pub async fn read(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let snapshot_id = snapshot_id.to_string();
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
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
    // Make sure the snapshot exists -- join metadata on the bucket id
    let maybe_response = sqlx::query_as!(
        responses::ReadSnapshotResponse,
        r#"SELECT 
            s.id as "id!",
            s.metadata_id as "metadata_id!",
            s.deal_id as "deal_id!",
            m.root_cid as "root_cid!",
            m.metadata_cid as "metadata_cid!",
            m.data_size as "data_size!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            s.id = $1 AND m.bucket_id = $2;"#,
        snapshot_id,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;

    let response = match maybe_response {
        Ok(r) => r,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read snapshot: {err}"),
            )
                .into_response();
        }
    };

    Json(response).into_response()
}

/// Read all snapshots for a bucket
pub async fn read_all(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let maybe_bucket = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM buckets WHERE id = $1 AND account_id = $2;"#,
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

    // Make sure the snapshot exists -- join metadata on the bucket id
    let maybe_response = sqlx::query_as!(
        responses::ReadSnapshotResponse,
        r#"SELECT 
            s.id as "id!",
            s.metadata_id as "metadata_id!",
            s.deal_id as "deal_id!",
            m.root_cid as "root_cid!",
            m.metadata_cid as "metadata_cid!",
            m.data_size as "data_size!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            m.bucket_id = $1;"#,
        bucket_id
    )
    .fetch_all(&mut *db_conn.0)
    .await;

    let response = match maybe_response {
        Ok(r) => responses::ReadAllSnapshotsResponse(r),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read snapshots: {err}"),
            )
                .into_response();
        }
    };

    Json(response).into_response()
}
