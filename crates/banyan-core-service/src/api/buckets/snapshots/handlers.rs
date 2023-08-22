use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::buckets::snapshots::{requests, responses};
use crate::extractors::{ApiToken, DbConn};
use crate::utils::db;

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
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unable to read bucket: {err}"),
                )
                    .into_response()
            }
        },
    }
    match db::authorize_metadata(&bucket_id, &metadata_id, &mut db_conn).await {
        Ok(_) => (),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("metadata not found: {err}"))
                    .into_response();
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unable to read metadata: {err}"),
                )
                    .into_response()
            }
        },
    }
    // Create a new snapshot
    let response = match db::create_snapshot(&metadata_id, &mut db_conn).await {
        Ok(snapshot) => responses::CreateSnapshotResponse {
            id: snapshot.id,
            created_at: snapshot.created_at.timestamp(),
        },
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unable to create snapshot: {err}"),
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
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unable to read bucket: {err}"),
                )
                    .into_response()
            }
        },
    }
    let response = match db::read_all_snapshots(&bucket_id, &mut db_conn).await {
        Ok(snapshots) => responses::ReadAllSnapshotsResponse(
            snapshots
                .into_iter()
                .map(|s| responses::ReadSnapshotResponse {
                    id: s.id,
                    metadata_id: s.metadata_id,
                    created_at: s.created_at.timestamp(),
                })
                .collect(),
        ),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("snapshot not found: {err}"))
                    .into_response();
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unable to read snapshot: {err}"),
                )
                    .into_response()
            }
        },
    };
    Json(response).into_response()
}

/// Restore a bucket to a specific snapshot
pub async fn restore(
    api_token: ApiToken,
    mut db_conn: DbConn,
    Path((bucket_id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let account_id = api_token.subject;
    let bucket_id = bucket_id.to_string();
    let snapshot_id = snapshot_id.to_string();

    // Check that the bucket exists
    match db::authorize_bucket(&account_id, &bucket_id, &mut db_conn).await {
        Ok(_) => {}
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("bucket not found: {err}")).into_response();
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unable to read bucket: {err}"),
                )
                    .into_response()
            }
        },
    }
    // Check that the snapshot exists
    let snapshot = match db::read_snapshot(&bucket_id, &snapshot_id, &mut db_conn).await {
        Ok(s) => s,
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return (StatusCode::NOT_FOUND, format!("snapshot not found: {err}"))
                    .into_response();
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("unable to read snapshot: {err}"),
                )
                    .into_response()
            }
        },
    };
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
                format!("unable to restore snapshot: {err}"),
            )
                .into_response();
        }
    };
    Json(response).into_response()
}
