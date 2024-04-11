use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use banyan_task::{CurrentTask, RecurringTask, RecurringTaskError, TaskLike};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::clients::{ReplicateDataRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::{Blocks, Bucket, Metadata, MinimalBlockLocation, StorageHost};
use crate::database::DatabaseConnection;
use crate::tasks::redistribute_staging_data::get_or_create_client_grant;
use crate::tasks::replicate_data::ReplicateDataTaskError::NotEnoughStorageHosts;

#[derive(sqlx::FromRow)]
pub struct BlockData {
    pub block_id: String,
    pub metadata_id: String,
    pub storage_host_id: String,
    pub host_count: i64,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct ReplicateDataTask {}

#[async_trait]
impl TaskLike for ReplicateDataTask {
    const TASK_NAME: &'static str = "replicate_data_task";

    type Error = ReplicateDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let db = ctx.database();
        let staging_host = StorageHost::select_staging(&db).await?;

        let staging_client = StagingServiceClient::new(
            ctx.secrets().service_key(),
            ctx.service_name(),
            &staging_host.name,
            Url::parse(&staging_host.url)?,
        );
        let mut conn = db.acquire().await?;
        let blocks_for_replication = get_blocks_for_replication(&mut conn).await?;
        let mut undistributed_blocks: HashSet<String> = blocks_for_replication
            .iter()
            .map(|block| block.block_id.clone())
            .collect();

        let mut blocks_grouped_by_metadata: HashMap<String, Vec<&BlockData>> = HashMap::new();
        for block in &blocks_for_replication {
            if !blocks_grouped_by_metadata
                .values()
                // deduplicate blocks across metadata
                .any(|blocks| blocks.iter().any(|b| b.block_id == block.block_id))
            {
                blocks_grouped_by_metadata
                    .entry(block.metadata_id.clone())
                    .or_default()
                    .push(block);
            }
        }

        for (metadata_id, grouped_blocks) in &blocks_grouped_by_metadata {
            let block_replicas = grouped_blocks
                .iter()
                .map(|block| block.host_count)
                .collect::<HashSet<_>>();
            if block_replicas.len() > 1 {
                tracing::error!(
                    "metadata {} has inconsistent replica count across blocks",
                    metadata_id
                );
                continue;
            }
            let metadata = Metadata::find_by_id_with_conn(&mut conn, metadata_id).await?;
            let bucket = Bucket::find_by_id(&mut conn, &metadata.bucket_id).await?;
            let replication_factor = bucket.replicas;
            let replicas_diff = replication_factor - grouped_blocks[0].host_count;

            // nothing to replicate
            if replicas_diff <= 0 {
                continue;
            }

            let block_ids: Vec<String> = grouped_blocks
                .iter()
                .map(|block| block.block_id.clone())
                .collect::<Vec<_>>();

            let block_cids: Vec<String> = Blocks::get_cids_by_ids(&mut conn, &block_ids).await?;

            let mut already_selected_hosts: Vec<String> = grouped_blocks
                .iter()
                .map(|block| block.storage_host_id.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let existing_hosts: Vec<String> = grouped_blocks
                .iter()
                .map(|block| block.storage_host_id.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let total_size = metadata
                .data_size
                .unwrap_or_default()
                .max(metadata.expected_data_size);

            for _ in 0..replicas_diff {
                let new_storage_host = match StorageHost::select_for_capacity_with_exclusion(
                    &mut conn,
                    total_size,
                    &already_selected_hosts,
                )
                .await
                {
                    Ok(host) => host,
                    _ => {
                        tracing::error!("not enough storage hosts for metadata {}", metadata_id);
                        return Err(NotEnoughStorageHosts(Error::RowNotFound));
                    }
                };
                already_selected_hosts.push(new_storage_host.id.clone());
                let authorization_grant = get_or_create_client_grant(
                    &mut conn,
                    &bucket.user_id,
                    total_size,
                    &new_storage_host,
                )
                .await?;

                // jumble the host that will be sending the data to avoid single host getting overloaded
                let old_host_id = existing_hosts.choose(&mut rand::thread_rng()).unwrap();
                let old_storage_host = StorageHost::find_by_id(&mut conn, old_host_id).await?;

                staging_client
                    .replicate_data(ReplicateDataRequest {
                        metadata_id: metadata_id.clone(),
                        block_cids: block_cids.clone(),
                        new_storage_grant_id: authorization_grant.id.clone(),
                        new_storage_grant_size: authorization_grant.authorized_amount,
                        new_host_id: new_storage_host.id.clone(),
                        new_host_url: new_storage_host.url.clone(),
                        old_host_id: old_storage_host.id.clone(),
                        old_host_url: old_storage_host.url.clone(),
                    })
                    .await?;

                for block_id in block_ids.iter() {
                    MinimalBlockLocation {
                        block_id: (*block_id).clone(),
                        metadata_id: metadata_id.clone(),
                        storage_host_id: new_storage_host.id.clone(),
                    }
                    .save(&mut conn)
                    .await?;
                }
            }

            for id in block_ids {
                undistributed_blocks.remove(&id);
            }
        }

        if !undistributed_blocks.is_empty() {
            tracing::warn!(
                "Not all metadata have been distributed. Remaining: {:?}",
                undistributed_blocks
            );
        }

        Ok(())
    }
}

impl RecurringTask for ReplicateDataTask {
    fn next_schedule(&self) -> Result<Option<OffsetDateTime>, RecurringTaskError> {
        OffsetDateTime::now_utc()
            .checked_add(time::Duration::hours(1))
            .ok_or(RecurringTaskError::DateTimeAddition)
            .map(Some)
    }
}

async fn get_blocks_for_replication(
    conn: &mut DatabaseConnection,
) -> Result<Vec<BlockData>, sqlx::Error> {
    // The below query will skip:
    // 1. Metadatas (and their associated blocks)  that have a block on the staging host. Those need to handled first by the redistribute_staging_data task.
    // 2. Blocks that have a block on a storage host that is not marked as pruned or expired.
    // 3. Blocks that have been kicked off for replication but not yet completed.
    let rows = sqlx::query(
        "SELECT bl.block_id, bl.metadata_id, bl.storage_host_id, COUNT(DISTINCT bl.storage_host_id) as host_count
            FROM block_locations bl
                 JOIN blocks b ON bl.block_id = b.id
                 JOIN metadata m ON bl.metadata_id = m.id
                 JOIN buckets bu ON m.bucket_id = bu.id
             LEFT JOIN block_locations bl2 ON bl.block_id = bl2.block_id AND bl2.stored_at IS NULL
            WHERE bl2.block_id IS NULL
              AND bl.pruned_at IS NULL
              AND bl.expired_at IS NULL
              AND bu.deleted_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM block_locations bl2
                  WHERE bl2.metadata_id = bl.metadata_id
                  AND bl2.storage_host_id = (SELECT id FROM storage_hosts WHERE staging IS TRUE)
              )
            GROUP BY bl.block_id
            HAVING host_count < bu.replicas;",
    )
    .fetch_all(&mut *conn).await?;

    // explicit conversion to BlockData because of weird  unsupported type NULL of column #4 ("host_count")
    // regardless of the COALESCE, NOT NULL, Option on the struct, etc.
    let blocks_for_replication: Vec<BlockData> = rows
        .into_iter()
        .map(|row| BlockData {
            block_id: row.get(0),
            metadata_id: row.get(1),
            storage_host_id: row.get(2),
            host_count: row.get(3),
        })
        .collect();

    let blocks_for_replication: Vec<BlockData> = blocks_for_replication
        .into_iter()
        .map(|row| BlockData {
            block_id: row.block_id,
            metadata_id: row.metadata_id,
            storage_host_id: row.storage_host_id,
            host_count: row.host_count,
        })
        .collect();

    let staging_host_id = sqlx::query!("SELECT id FROM storage_hosts WHERE staging IS TRUE")
        .fetch_one(&mut *conn)
        .await?
        .id;

    let mut staging_metadata_ids = HashSet::new();
    for block in &blocks_for_replication {
        if block.storage_host_id == staging_host_id {
            staging_metadata_ids.insert(block.metadata_id.clone());
        }
    }

    let final_blocks_for_replication: Vec<BlockData> = blocks_for_replication
        .into_iter()
        .filter(|block| !staging_metadata_ids.contains(&block.metadata_id))
        .collect();

    Ok(final_blocks_for_replication)
}

#[derive(Debug, thiserror::Error)]
pub enum ReplicateDataTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("staging host url error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("staging error: {0}")]
    StagingServiceError(#[from] StagingServiceError),
    #[error("not enough storage hosts")]
    NotEnoughStorageHosts(sqlx::Error),
}
#[cfg(test)]
mod tests {
    use crate::database::models::MetadataState;
    use crate::database::test_helpers::associate_blocks;
    use crate::database::{test_helpers, DatabaseConnection};
    use crate::tasks::replicate_data::get_blocks_for_replication;

    async fn sample_block_for_host(
        conn: &mut DatabaseConnection,
        user_id: &str,
        storage_host_id: &str,
        bucket_id: &str,
    ) -> Vec<String> {
        let storage_grant_id =
            test_helpers::create_storage_grant(conn, storage_host_id, user_id, 1_000_000).await;
        let metadata_id =
            test_helpers::sample_metadata(conn, bucket_id, 1, MetadataState::Current).await;

        test_helpers::sample_blocks(conn, 4, &metadata_id, storage_host_id, &storage_grant_id).await
    }

    async fn setup_test_environment(
        conn: &mut DatabaseConnection,
        user_email: &str,
        host_name: &str,
        host_url: &str,
    ) -> (String, String, String) {
        let user_id = test_helpers::sample_user(conn, user_email).await;
        let bucket_id = test_helpers::sample_bucket(conn, &user_id).await;
        let host_id = test_helpers::create_storage_host(conn, host_name, host_url, 1_000_000).await;
        (user_id, bucket_id, host_id)
    }

    #[tokio::test]
    async fn test_all_blocks_on_staging_host() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, staging_host_id) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sample_block_for_host(&mut conn, &user_id, &staging_host_id, &bucket_id).await;
        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert!(blocks_for_replication.is_empty());
    }

    #[tokio::test]
    async fn test_no_blocks_returned_on_full_replication() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, _) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sqlx::query!("UPDATE buckets SET replicas = 2 WHERE id = $1", bucket_id)
            .execute(&mut *conn)
            .await
            .unwrap();
        let storage_host_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        let block_ids =
            sample_block_for_host(&mut conn, &user_id, &storage_host_id, &bucket_id).await;

        let storage_host_two_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service-two",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        let metadata_id = sqlx::query_scalar!(
            "SELECT id FROM metadata AS m JOIN block_locations as bl ON bl.metadata_id = m.id WHERE block_id = $1;",
            block_ids[0]
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap();

        associate_blocks(
            &mut conn,
            &metadata_id,
            &storage_host_two_id,
            block_ids.iter().map(String::as_str),
        )
        .await;

        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert!(blocks_for_replication.is_empty());
    }

    #[tokio::test]
    async fn test_scheduled_and_completed_returns_no_blocks() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, _) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sqlx::query!("UPDATE buckets SET replicas = 2 WHERE id = $1", bucket_id)
            .execute(&mut *conn)
            .await
            .unwrap();
        let storage_host_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        let block_ids =
            sample_block_for_host(&mut conn, &user_id, &storage_host_id, &bucket_id).await;

        let storage_host_two_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service-two",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        let metadata_id = sqlx::query_scalar!(
            "SELECT id FROM metadata AS m JOIN block_locations as bl ON bl.metadata_id = m.id WHERE block_id = $1;",
            block_ids[0]
        )
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        associate_blocks(
            &mut conn,
            &metadata_id,
            &storage_host_two_id,
            block_ids.iter().map(String::as_str),
        )
        .await;

        for block_id in &block_ids {
            sqlx::query!(
                "UPDATE block_locations SET stored_at = NULL WHERE block_id = $1 AND storage_host_id != $2;",
                block_id,
                storage_host_two_id
            )
                .execute(&mut *conn)
                .await
                .unwrap();
        }

        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert!(blocks_for_replication.is_empty());
    }

    #[tokio::test]
    async fn test_staging_blocks_are_skipped() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, staging_host_id) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sqlx::query!("UPDATE buckets SET replicas = 2 WHERE id = $1", bucket_id)
            .execute(&mut *conn)
            .await
            .unwrap();
        let new_host_id = test_helpers::create_storage_host(
            &mut conn,
            "Diskz",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;

        let staging_blocks =
            sample_block_for_host(&mut conn, &user_id, &staging_host_id, &bucket_id).await;
        let new_host_blocks =
            sample_block_for_host(&mut conn, &user_id, &new_host_id, &bucket_id).await;
        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert_eq!(blocks_for_replication.len(), new_host_blocks.len());
        for block in blocks_for_replication {
            assert!(!staging_blocks.contains(&block.block_id));
        }
    }

    #[tokio::test]
    async fn test_returns_no_blocks_on_pruned_or_expired_host() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, _) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sqlx::query!("UPDATE buckets SET replicas = 2 WHERE id = $1", bucket_id)
            .execute(&mut *conn)
            .await
            .unwrap();
        let storage_host_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        sample_block_for_host(&mut conn, &user_id, &storage_host_id, &bucket_id).await;

        sqlx::query!(
            "UPDATE block_locations SET pruned_at = DATETIME('now') WHERE storage_host_id = $1",
            storage_host_id
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert!(blocks_for_replication.is_empty());

        sample_block_for_host(&mut conn, &user_id, &storage_host_id, &bucket_id).await;
        sqlx::query!(
            "UPDATE block_locations SET expired_at = DATETIME('now') WHERE storage_host_id = $1",
            storage_host_id
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert!(blocks_for_replication.is_empty());
    }

    #[tokio::test]
    async fn test_schedules_blocks_for_replication() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, _) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sqlx::query!("UPDATE buckets SET replicas = 2 WHERE id = $1", bucket_id)
            .execute(&mut *conn)
            .await
            .unwrap();
        let storage_host_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        sample_block_for_host(&mut conn, &user_id, &storage_host_id, &bucket_id).await;

        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();

        assert_eq!(blocks_for_replication.len(), 4);
    }

    #[tokio::test]
    async fn test_new_blocks_not_scheduled_on_replication_not_complete() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let (user_id, bucket_id, _) = setup_test_environment(
            &mut conn,
            "user@domain.tld",
            "staging-service",
            "http://127.0.0.1:8001/",
        )
        .await;
        sqlx::query!("UPDATE buckets SET replicas = 2 WHERE id = $1", bucket_id)
            .execute(&mut *conn)
            .await
            .unwrap();
        let storage_host_id = test_helpers::create_storage_host(
            &mut conn,
            "storage-service",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;
        let block_ids =
            sample_block_for_host(&mut conn, &user_id, &storage_host_id, &bucket_id).await;

        for block_id in &block_ids {
            sqlx::query!(
                "UPDATE block_locations SET stored_at = NULL WHERE block_id = $1",
                block_id
            )
            .execute(&mut *conn)
            .await
            .unwrap();
        }

        let blocks_for_replication = get_blocks_for_replication(&mut conn).await.unwrap();
        assert!(blocks_for_replication.is_empty());
    }
}
