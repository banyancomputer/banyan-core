use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::auth::STAGING_SERVICE_NAME;
use crate::clients::{DistributeDataRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::{
    BlockLocationState, Blocks, Metadata, MinimalBlockLocation, StorageHost,
    StorageHostsMetadatasStorageGrants,
};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct RedistributeStagingDataTask {}

#[async_trait]
impl TaskLike for RedistributeStagingDataTask {
    const TASK_NAME: &'static str = "redistribute_staging_data_task";

    type Error = RedistributeStagingDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let staging_host = StorageHost::select_by_name(&database, STAGING_SERVICE_NAME).await?;
        let blocks = Blocks::get_blocks_requiring_sync(&database, &staging_host.id).await?;

        let mut undistributed_blocks: HashSet<String> =
            blocks.iter().map(|block| block.id.clone()).collect();

        let staging_client = StagingServiceClient::new(
            ctx.secrets().service_key(),
            ctx.service_name(),
            staging_host.name.as_str(),
            Url::parse(&staging_host.url)?,
        );

        let mut blocks_grouped_by_metadata = HashMap::new();
        for block in &blocks {
            blocks_grouped_by_metadata
                .entry(block.metadata_id.clone())
                .or_insert_with(Vec::new)
                .push(block);
        }

        for (metadata_id, blocks) in &blocks_grouped_by_metadata {
            tracing::info!("Redistributing blocks for metadata: {:?}", &metadata_id);
            let metadata = Metadata::find_by_id(&database, &metadata_id.clone()).await?;
            let grant_metadata =
                StorageHostsMetadatasStorageGrants::find_by_metadata_and_storage_host(
                    &database,
                    &metadata_id.clone(),
                    &staging_host.id,
                )
                .await?;

            let total_size = metadata
                .data_size
                .unwrap_or_default()
                .max(metadata.expected_data_size);

            let new_storage_host = StorageHost::select_for_capacity_with_exclusion(
                &database,
                total_size,
                staging_host.id.as_str(),
            )
            .await?;

            let block_cids = blocks
                .iter()
                .map(|block| block.cid.clone())
                .collect::<Vec<_>>();

            staging_client
                .distribute_data(DistributeDataRequest {
                    metadata_id: metadata_id.clone(),
                    grant_id: grant_metadata.storage_grant_id.clone(),
                    new_host_id: new_storage_host.id.clone(),
                    block_cids: block_cids.clone(),
                    new_host_url: new_storage_host.url.clone(),
                })
                .await?;

            let block_ids = blocks
                .iter()
                .map(|block| block.id.clone())
                .collect::<Vec<_>>();

            MinimalBlockLocation::update_state(&database, &block_ids, BlockLocationState::Staged)
                .await
                .expect("update block location state");

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

#[derive(Debug, thiserror::Error)]
pub enum RedistributeStagingDataTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("staging host url error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("staging error: {0}")]
    StagingServiceError(#[from] StagingServiceError),
}
#[cfg(test)]
mod tests {
    use banyan_task::{CurrentTask, TaskLike};
    use mockito::{Server, ServerOpts};
    use serde_json::json;

    use crate::app::mock_app_state;
    use crate::auth::STAGING_SERVICE_NAME;
    use crate::database::models::{BlockLocationState, MetadataState, MinimalBlockLocation};
    use crate::database::test_helpers;
    use crate::tasks::redistribute_staging_data::RedistributeStagingDataTask;

    #[tokio::test]
    async fn test_block_state_changed() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let staging_host_id = test_helpers::create_storage_host(
            &mut conn,
            STAGING_SERVICE_NAME,
            "http://127.0.0.1:8001/",
            1_000_000,
        )
        .await;

        let new_host_id = test_helpers::create_storage_host(
            &mut conn,
            "Diskz",
            "http://127.0.0.1:8002/",
            1_000_000,
        )
        .await;

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;
        let storage_grant_id =
            test_helpers::create_storage_grant(&mut conn, &staging_host_id, &user_id, 1_000_000)
                .await;
        let metadata_id =
            test_helpers::sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;

        let block_ids = test_helpers::sample_blocks(
            &mut conn,
            4,
            &metadata_id,
            &staging_host_id,
            &storage_grant_id,
        )
        .await;

        let mut server = Server::new_with_opts_async(ServerOpts {
            host: "127.0.0.1",
            port: 8001,
            assert_on_drop: false,
        })
        .await;
        // do not put in a function or remove the variable, because the mock will get dropped and not match
        let _m = server
            .mock("POST", "/api/v1/hooks/distribute")
            .match_body(mockito::Matcher::PartialJsonString(
                json!({
                    "metadata_id": metadata_id,
                    "new_host_id": new_host_id,
                    "grant_id": storage_grant_id
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body("{}")
            .create_async()
            .await;

        let all_blocks = MinimalBlockLocation::get_all(&db)
            .await
            .expect("get all blocks");
        assert_eq!(all_blocks.len(), block_ids.len());
        for block_location in all_blocks.iter() {
            assert_eq!(block_location.state, BlockLocationState::SyncRequired);
        }

        let res = RedistributeStagingDataTask::default()
            .run(CurrentTask::default(), mock_app_state(db.clone()).0)
            .await;

        println!("{:?}", res);

        assert!(res.is_ok());
        let updated_block_locations = MinimalBlockLocation::get_all(&db)
            .await
            .expect("get all blocks");
        assert_eq!(updated_block_locations.len(), block_ids.len());
        for block_location in updated_block_locations.iter() {
            assert_eq!(block_location.state, BlockLocationState::Staged);
        }
    }
}
