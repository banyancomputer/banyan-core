use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::buckets::snapshots::{requests, responses};
use crate::db::models;
use crate::extractors::{ApiToken, DbConn};

pub async fn create(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path(bucket_id): Path<Uuid>,
    Json(new_snapshot): Json<requests::CreateSnapshotRequest>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let metadata_id = new_snapshot.metadata_id.to_string();
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
        metadata_id,
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
        metadata_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    match maybe_snapshot {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create snapshot: {err}"),
            )
                .into_response();
        },
    };
    // Make sure the snapshot exists -- join metadata on the bucket id
    let maybe_response = sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id as "id!",
            s.metadata_id as "metadata_id!",
            s.created_at as "created_at!"
        FROM 
            snapshots s
        INNER JOIN 
            metadata m ON m.id = s.metadata_id
        WHERE 
            m.bucket_id = $1;"#,
        bucket_id
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let response = match maybe_response {
        Ok(r) => responses::CreateSnapshotResponse {
            id: r.id,
            created_at: r.created_at.timestamp(),
        },
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
    let maybe_snapshots = sqlx::query_as!(
        models::Snapshot,
        r#"SELECT 
            s.id as "id!",
            s.metadata_id as "metadata_id!",
            s.created_at as "created_at!"
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

    let response = match maybe_snapshots {
        Ok(s) => responses::ReadAllSnapshotsResponse(
            s.into_iter()
                .map(|s| responses::ReadSnapshotResponse {
                    id: s.id,
                    metadata_id: s.metadata_id,
                    created_at: s.created_at.timestamp()
                })
                .collect(),
        ),
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

/// Restore a bucket to a specific snapshot
pub async fn restore(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, snapshot_id)): Path<(Uuid, Uuid)>
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let snapshot_id = snapshot_id.to_string();
    println!("bucket_id: {}", bucket_id);
    println!("snapshot_id: {}", snapshot_id);

    // Check that the bucket exists
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
    println!("bucket exists");
    // Check that the snapshot exists
    let maybe_snapshot = sqlx::query_as!(
        models::Snapshot,
        r#"SELECT id as "id!", metadata_id as "metadata_id!", created_at as "created_at!" FROM snapshots WHERE id = $1;"#,
        snapshot_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let snapshot = match maybe_snapshot {
        Ok(b) => b,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read snapshot: {err}"),
            )
                .into_response();
        }
    };
    println!("snapshot exists");
    // Make sure the metadata specified in the snapshot exists, and points to this bucket
    let maybe_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"SELECT id as "id!" FROM metadata WHERE id = $1 AND bucket_id = $2;"#,
        snapshot.metadata_id,
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
    println!("metadata exists");

    // TODO: This is incomplete behavior!
    // Set the metadata state to current
    let maybe_metadata_update = sqlx::query!(
        r#"UPDATE metadata SET state = 'current' WHERE id = $1;"#,
        snapshot.metadata_id,
    )
    .execute(&mut *db_conn.0)
    .await;
    let response = match maybe_metadata_update {
        Ok(_) => responses::RestoreSnapshotResponse {
            metadata_id: snapshot.metadata_id,
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to read snapshot: {err}"),
            )
                .into_response();
        }
    };
    println!("metadata updated");
    Json(response).into_response()
}
