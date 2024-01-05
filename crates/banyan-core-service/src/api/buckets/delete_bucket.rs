use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::QueryBuilder;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::Bucket;
use crate::database::BIND_LIMIT;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, DeleteBucketError> {
    let bucket_id = bucket_id.to_string();
    let user_id = user_identity.id().to_string();
    let database = state.database();
    let mut conn = database.acquire().await?;

    // Check that the bucket exists and is owned by the user
    // Note: this also enforces that the bucket does not have `deleted_at` set
    if !Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id).await? {
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    // TODO: This is more of a soft deletion at the moment.
    //  In the future we need to delete all the hot data stored at various storage hosts.

    // Ok, we've done our initial check, go into txn land to handle soft deletion
    conn.close().await?;
    let mut conn = database.begin().await?;

    // Note: this will give all updated entries subsecond precision tim
    // Enforce that all updates to occur have the same timestamp
    let now = OffsetDateTime::now_utc();

    // Query for all pieces of metadata that have a snapshot associated with them
    // Order by updated_at so that the most recently snapshotted metadata is first
    // if at present
    let metadata_ids_with_snapshots = sqlx::query_scalar!(
        "SELECT m.id FROM metadata as m
        JOIN snapshots as s ON s.metadata_id = m.id
        WHERE m.bucket_id = $1
        ORDER BY m.updated_at DESC;",
        bucket_id,
    )
    .fetch_all(&mut *conn)
    .await?;

    // If there are no snapshots associated with the bucket:
    if metadata_ids_with_snapshots.is_empty() {
        // Just set the bucket as soft deleted
        sqlx::query!(
            "UPDATE buckets SET deleted_at = $1, updated_at = $1 WHERE id = $2;",
            now,
            bucket_id,
        )
        .execute(&mut *conn)
        .await?;

        // And since we know there's no snapshots, we can just update the metadata to be deleted,
        // commit the txn and return
        sqlx::query!(
            "UPDATE metadata SET state = 'deleted', updated_at = $1 WHERE bucket_id = $2 AND state != 'deleted';",
            now,
            bucket_id,
        ).execute(&mut *conn).await?;

        conn.commit().await?;
        return Ok((StatusCode::NO_CONTENT, ()).into_response());
    }

    // Get the id of the most recently snapshotted metadata
    let most_recently_snapshotted_metadata_id = &metadata_ids_with_snapshots[0];

    // Otherwise, if there snapshots for this bucket
    // we should set its type it 'backup' and its storage class to 'cold'
    sqlx::query!(
        "UPDATE buckets SET type = 'backup', storage_class = 'cold', updated_at = $1 WHERE id = $2;", 
        now,
        bucket_id,
    ).execute(&mut *conn).await?;

    // Any metadata that does not have a snapshot associated with it should have its state set to deleted
    // including the current one, unless the state is already deleted. Any metadata with a snapshot should
    // be marked as outdated
    for chunk in metadata_ids_with_snapshots.chunks(BIND_LIMIT) {
        // Build a query to update metadata without a snapshot
        let mut non_shapshot_query_builder = QueryBuilder::new(
            "UPDATE metadata SET state = 'deleted', updated_at = $1
            WHERE state != 'deleted' AND bucket_id = $2 AND id NOT IN (",
        );

        // Add bind params for the timestamp and bucket_id
        non_shapshot_query_builder.push_bind(now);
        non_shapshot_query_builder.push_bind(&bucket_id);

        // And attach a list of bind params for the metadata_ids to avoid
        let mut separated_values = non_shapshot_query_builder.separated(", ");
        for metadata_id in chunk {
            separated_values.push(metadata_id);
        }

        non_shapshot_query_builder.push(");");

        // Build a query to update metadata with a snapshot
        let mut shapshot_query_builder = QueryBuilder::new(
            "UPDATE metadata SET state = 'outdated', updated_at = $1
            WHERE state != 'deleted' AND bucket_id = $2 AND id IN (",
        );

        // Add bind params for the timestamp and bucket_id
        shapshot_query_builder.push_bind(now);
        shapshot_query_builder.push_bind(&bucket_id);

        let mut separated_values = shapshot_query_builder.separated(", ");
        for metadata_id in chunk {
            separated_values.push(metadata_id);
        }

        shapshot_query_builder.push(");");

        // Build and execute the queries
        let non_shapshot_query = non_shapshot_query_builder.build();
        let shapshot_query = shapshot_query_builder.build();
        non_shapshot_query.execute(&mut *conn).await?;
        shapshot_query.execute(&mut *conn).await?;
    }

    // The most recent metadata with a snapshot should be marked as current
    // Don't worry about the timestamp here, since we just updated all the metadata
    sqlx::query!(
        "UPDATE metadata as m SET state = 'current'
        WHERE m.bucket_id = $1 AND m.id = $2;",
        bucket_id,
        most_recently_snapshotted_metadata_id,
    )
    .execute(&mut *conn)
    .await?;

    // Commit the txn and return
    conn.commit().await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteBucketError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),
}

impl IntoResponse for DeleteBucketError {
    fn into_response(self) -> Response {
        tracing::error!("internal error handling bucket usage request: {self}");
        let err_msg = serde_json::json!({"msg": "a backend service issue encountered an error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
