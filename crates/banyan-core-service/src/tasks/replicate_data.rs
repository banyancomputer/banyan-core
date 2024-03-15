use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::clients::{ReplicateDataRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::{
    Blocks, Bucket, Metadata, MinimalBlockLocation, StorageHost,
};

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
        // let blocks_for_replication = get_blocks_for_replication(&mut conn).await?;
        let blocks_for_replication: Vec<BlockData> = Vec::new();
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
                tracing::error!("metadata {} has inconsistent replica count", metadata_id);
                continue;
            }
            let metadata = Metadata::find_by_id_with_conn(&mut conn, metadata_id).await?;
            let bucket = Bucket::find_by_id(&mut conn, &metadata.bucket_id).await?;
            let replication_factor = bucket.replicas;
            let replicas_diff = replication_factor - grouped_blocks[0].host_count;

            if replicas_diff <= 0 {
                continue;
            }
            let block_ids: Vec<String> = grouped_blocks
                .iter()
                .map(|block| block.block_id.clone())
                .collect::<Vec<_>>();
            let block_cids: Vec<String> = Blocks::get_cids_by_ids(&mut conn, &block_ids).await?;
            let mut selected_hosts: Vec<String> = grouped_blocks
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

            for _ in 0..replicas_diff {
                let new_storage_host = StorageHost::select_for_capacity_with_exclusion(
                    &mut conn,
                    metadata.expected_data_size,
                    &selected_hosts,
                )
                .await?;
                selected_hosts.push(new_storage_host.id.clone());

                // jumble the host that will be sending the data to avoid single host getting overloaded
                let old_host_id = existing_hosts.choose(&mut rand::thread_rng()).unwrap();
                let old_storage_host = StorageHost::find_by_id(&mut conn, &old_host_id).await?;

                staging_client
                    .replicate_data(ReplicateDataRequest {
                        metadata_id: metadata_id.clone(),
                        block_cids: block_cids.clone(),
                        new_host_id: new_storage_host.id.clone(),
                        new_host_url: new_storage_host.url.clone(),
                        old_host_id: old_storage_host.id.clone(),
                        old_host_url: old_storage_host.url.clone(),
                    })
                    .await?;

                for block_id in block_ids.iter() {
                    MinimalBlockLocation {
                        block_id: block_id.clone(),
                        metadata_id: metadata_id.clone(),
                        storage_host_id: new_storage_host.id.clone(),
                    }
                    .save(&mut conn)
                    .await?;
                }
            }

            undistributed_blocks.retain(|s| !block_ids.contains(s));
        }

        if !undistributed_blocks.is_empty() {
            tracing::warn!(
                "Not all metadata have been distributed. Remaining: {:?}",
                undistributed_blocks
            );
        }

        Ok(())
    }

    fn next_time(&self) -> Option<OffsetDateTime> {
        Some(OffsetDateTime::now_utc() + time::Duration::hours(1))
    }
}

// async fn get_blocks_for_replication(
//     conn: &mut DatabaseConnection,
// ) -> Result<Vec<BlockData>, sqlx::Error> {
//     let blocks_for_replication = sqlx::query_as!(
//         BlockData,
//         "SELECT bl.block_id, bl.metadata_id, bl.storage_host_id, COUNT(DISTINCT bl.storage_host_id) as host_count
//         FROM block_locations bl
//                  JOIN blocks b ON bl.block_id = b.id
//                  JOIN metadata m ON bl.metadata_id = m.id
//                  JOIN buckets bu ON m.bucket_id = bu.id
//                  JOIN users u ON bu.user_id = u.id
//         WHERE bl.storage_host_id != (SELECT id FROM storage_hosts WHERE staging IS TRUE)
//           AND bl.pruned_at IS NULL
//           AND bl.expired_at IS NULL
//           AND bu.deleted_at IS NULL
//         GROUP BY bl.block_id, bl.metadata_id
//         HAVING host_count < bu.host_count;",
//     )
//         .fetch_all(&mut *conn).await?;
//     Ok(blocks_for_replication)
// }

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
    NotEnoughStorageHosts,
    #[error("not all blocks were replicated")]
    NotAllBlocksReplicated,
    #[error("inconsistent replica count in the same metadata")]
    InconsistentReplicaCount,
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_replication_task() {
        // TODO: Implement tests for the replication task
    }
}
