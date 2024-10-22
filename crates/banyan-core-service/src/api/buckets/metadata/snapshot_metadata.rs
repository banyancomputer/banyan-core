use std::collections::BTreeSet;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::TaskLikeExt;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::SnapshotState;
use crate::extractors::UserIdentity;
use crate::tasks::{CreateDealsTask, BLOCK_SIZE};
use crate::utils::is_valid_cid;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
    Json(request_cids): Json<BTreeSet<String>>,
) -> Result<Response, CreateSnapshotError> {
    let database = state.database();

    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    if request_cids.is_empty() {
        return Err(CreateSnapshotError::EmptyBucket(
            "no cids provided".to_string(),
        ));
    }

    if let Some(invalid_cid) = request_cids.iter().find(|cid| !is_valid_cid(cid)) {
        tracing::error!("received invalid CID: {}", invalid_cid);
        return Err(CreateSnapshotError::InvalidRequestCid);
    }

    let mut transaction = database.begin().await?;
    let user_id = user_identity.id().to_string();
    let metadata_id = sqlx::query_scalar!(
        r#"SELECT m.id FROM metadata AS m
               JOIN buckets AS b ON m.bucket_id = b.id
               LEFT JOIN snapshots AS s ON s.metadata_id = m.id
               WHERE b.user_id = $1
                   AND b.id = $2
                   AND m.id = $3
                   AND m.state != 'deleted'
                   AND s.id IS NULL;"#,
        user_id,
        bucket_id,
        metadata_id,
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(CreateSnapshotError::NotFound)?;

    let cid_list: Vec<_> = request_cids.into_iter().collect();
    let size_estimate = cid_list.len() as i64 * BLOCK_SIZE;

    let pending_state = SnapshotState::Pending.to_string();
    let snapshot_id = sqlx::query_scalar!(
        r#"INSERT INTO snapshots (metadata_id, state, size)
               VALUES ($1, $2, $3)
               RETURNING id;"#,
        metadata_id,
        pending_state,
        size_estimate,
    )
    .fetch_one(&mut *transaction)
    .await?;

    // Create query builder that can serve as the basis for every chunk
    let mut builder = sqlx::QueryBuilder::new(format!(
        "INSERT INTO snapshot_block_locations 
            SELECT s.id as snapshot_id, bl.block_id 
            FROM blocks AS b 
            JOIN block_locations AS bl ON b.id = bl.block_id 
            JOIN metadata AS m ON bl.metadata_id = m.id 
            JOIN snapshots AS s 
            WHERE m.id = \"{metadata_id}\"
            AND s.id = \"{snapshot_id}\"
            AND b.cid IN ("
    ));

    // For every chunk of 1000 CIDs
    for cid_chunk in cid_list.chunks(1000) {
        // Reset the builder and append the CID list
        builder.reset();
        let mut separated = builder.separated(", ");
        for cid in cid_chunk {
            separated.push_bind(cid);
        }
        separated.push_unseparated(");");

        let res = builder.build().execute(&mut *transaction).await?;

        if res.rows_affected() != cid_chunk.len() as u64 {
            return Err(CreateSnapshotError::AssociationMismatch(format!(
                "expected {} got {}",
                cid_chunk.len(),
                res.rows_affected()
            )));
        }
    }

    CreateDealsTask::new(snapshot_id.clone())
        .enqueue::<banyan_task::SqliteTaskStore>(&mut *transaction)
        .await
        .map_err(CreateSnapshotError::UnableToEnqueueTask)?;

    transaction.commit().await?;

    let resp_msg = serde_json::json!({ "id": snapshot_id });
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSnapshotError {
    #[error("association mismatch: {0}")]
    AssociationMismatch(String),

    #[error("an error occurred while interacting with the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("cannot snapshot an empty bucket: {0}")]
    EmptyBucket(String),

    #[error("one or more of the provided CIDs are invalid")]
    InvalidRequestCid,

    #[error("no matching metadata for the current account")]
    NotFound,

    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(banyan_task::TaskStoreError),
}

impl IntoResponse for CreateSnapshotError {
    fn into_response(self) -> Response {
        match &self {
            CreateSnapshotError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            CreateSnapshotError::EmptyBucket(_) | CreateSnapshotError::InvalidRequestCid => {
                let err_msg = serde_json::json!({"msg": self.to_string()});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            CreateSnapshotError::AssociationMismatch(e) => {
                let err_msg = serde_json::json!({"msg": e.to_string()});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error creating snapshot: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use axum::extract::{Json, Path};
    use http::StatusCode;
    use uuid::Uuid;

    use crate::api::buckets::metadata::snapshot_metadata::{handler, CreateSnapshotError};
    use crate::app::mock_app_state;
    use crate::database::models::{MetadataState, Snapshot};
    use crate::database::test_helpers::{
        associate_blocks, create_blocks, create_storage_host, data_generator, generate_cids,
        get_or_create_session, sample_bucket, sample_metadata, sample_user, setup_database,
    };
    use crate::database::Database;
    use crate::extractors::UserIdentity;

    #[allow(dead_code)]
    #[derive(Debug, sqlx::FromRow)]
    pub struct SnapshotBlockLocation {
        pub snapshot_id: String,
        pub block_id: String,
    }
    impl SnapshotBlockLocation {
        pub(crate) async fn get_all(conn: &Database) -> Vec<SnapshotBlockLocation> {
            sqlx::query_as!(
                SnapshotBlockLocation,
                "SELECT * FROM snapshot_block_locations;"
            )
            .fetch_all(conn)
            .await
            .expect("snapshot block locations")
        }
    }

    #[tokio::test]
    async fn test_create_snapshot_no_cids_returns_error() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = sample_user(&mut conn, "test@example.com").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;

        let res = handler(
            UserIdentity::Session(get_or_create_session(&mut conn, &user_id).await),
            mock_app_state(db.clone()),
            Path((
                Uuid::parse_str(&bucket_id).expect("bucket id as uuid"),
                Uuid::parse_str(&metadata_id).expect("bucket id as uuid"),
            )),
            Json(BTreeSet::new()), // No CIDs provided
        )
        .await;

        assert!(res.is_err());
        assert!(matches!(res, Err(CreateSnapshotError::EmptyBucket(_))));
        assert_eq!(SnapshotBlockLocation::get_all(&db).await.len(), 0);
        assert_eq!(Snapshot::get_all(&db).await.len(), 0);
    }

    #[tokio::test]
    async fn test_create_snapshot_works() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = sample_user(&mut conn, "test@example.com").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let prim_storage_host_id =
            create_storage_host(&mut conn, "Diskz", "https://127.0.0.1:8001/", 1_000_000).await;
        let cids_set: BTreeSet<String> = generate_cids(data_generator(0..3)).collect();
        let cids_string: Vec<String> = cids_set.iter().cloned().collect();
        let initial_blocks = create_blocks(&mut conn, cids_string.iter().map(String::as_str)).await;
        associate_blocks(
            &mut conn,
            &metadata_id,
            &prim_storage_host_id,
            initial_blocks.iter().map(String::as_str),
        )
        .await;

        let res = handler(
            UserIdentity::Session(get_or_create_session(&mut conn, &user_id).await),
            mock_app_state(db.clone()),
            Path((
                Uuid::parse_str(&bucket_id).expect("bucket id as uuid"),
                Uuid::parse_str(&metadata_id).expect("metadata id as uuid"),
            )),
            Json(cids_set.clone()),
        )
        .await;

        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(Snapshot::get_all(&db).await.len(), 1);
        assert_eq!(
            SnapshotBlockLocation::get_all(&db).await.len(),
            cids_set.len()
        );
    }
}
