use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::clients::{ReplicateDataRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::{BlockLocations, Blocks, Bucket, Metadata, MinimalBlockLocation, NewStorageGrant, StorageHost, StorageHostsMetadatasStorageGrants, UserStorageReport};
use crate::utils::rounded_storage_authorization;

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct ReplicateDataTask {}

#[async_trait]
impl TaskLike for ReplicateDataTask {
    const TASK_NAME: &'static str = "replicate_data_task";

    type Error = ReplicateDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let mut conn = database.begin().await?;
        let buckets = Bucket::find_all_non_deleted(&mut conn).await?;
        //
        // for bucket in buckets {
        //     let replication_factor = bucket.replicas;
        //     let metadatas = Metadata::find_by_bucket_id(&database, &bucket.id).await?;
        //     let block_locations = BlockLocations::find_by_locations_for_metadata_id( &database, &metadta    ).await?;
        //
        //     for metadata in metadatas {
        //         let blocks = Blocks::get_by_metadata_id(&database, &metadata.id).await?;
        //         // we preserve the invariant that blocks for the same metadata are collocated on the same host
        //         let unique_storage_hosts = blocks.iter().map(|block| block.storage_host_id.clone()).collect::<HashSet<_>>();
        //         let storage_replicas = unique_storage_hosts.len();
        //
        //         let replicas_diff = replication_factor as usize - storage_replicas;
        //
        //         if replicas_diff > 0 {
        //             if storage_hosts.len() < replicas_diff {
        //                 tracing::error!("Not enough storage hosts to replicate the blocks to");
        //                 return Err(ReplicateDataTaskError::NotEnoughStorageHosts);
        //             }
        //
        //             for _ in 0..replicas_diff {
        //                 let new_host = storage_hosts.choose(&mut rand::thread_rng()).unwrap();
        //                 let block_cids: Vec<_> = blocks.iter().map(|block| block.cid.clone()).collect();
        //
        //                 let distribute_data = ReplicateDataRequest {
        //                     metadata_id: metadata.id.clone(),
        //                     block_cids: block_cids.clone(),
        //                     new_host_id: new_host.id.clone(),
        //                     new_host_url: new_host.url.clone(),
        //                     old_host_id: blocks[0].storage_host_id.clone(),
        //                     old_host_url: blocks[0].storage_host_url.clone(),
        //                 };
        //
        //                 let staging_client = StagingServiceClient::new(
        //                     ctx.secrets().service_key(),
        //                     ctx.service_name(),
        //                     &new_host.name,
        //                     Url::parse(&new_host.url)?,
        //                 );
        //
        //                 staging_client
        //                     .replicate_data(distribute_data)
        //                     .await?;
        //             }
        //         }
        //     }
        // }

        Ok(())
    }

    fn next_time(&self) -> Option<OffsetDateTime> {
        Some(OffsetDateTime::now_utc() + time::Duration::hours(1))
    }
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
    NotEnoughStorageHosts,
}

#[cfg(test)]
mod tests {
    use banyan_task::{CurrentTask, TaskLike};
    use mockito::{Server, ServerOpts};
    use serde_json::json;

    use crate::app::mock_app_state;
    use crate::database::models::{BlockLocations, MetadataState};
    use crate::database::test_helpers;
    use crate::tasks::replicate_data::ReplicateDataTask;

    #[tokio::test]
    async fn test_replication_task() {
        // TODO: Implement tests for the replication task
    }
}
