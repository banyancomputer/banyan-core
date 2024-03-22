use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use banyan_task::{CurrentTask, RecurringTask, RecurringTaskError, TaskLike};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::clients::{DistributeDataRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::{
    Blocks, Bucket, ExistingStorageGrant, Metadata, MinimalBlockLocation, NewStorageGrant,
    StorageHost, UserStorageReport,
};
use crate::database::DatabaseConnection;
use crate::utils::rounded_storage_authorization;

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct RedistributeStagingDataTask {}

#[async_trait]
impl TaskLike for RedistributeStagingDataTask {
    const TASK_NAME: &'static str = "redistribute_staging_data_task";

    type Error = RedistributeStagingDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let staging_host = StorageHost::select_staging(&database).await?;
        let blocks_for_sync =
            Blocks::get_blocks_requiring_sync(&database, &staging_host.id).await?;

        let mut undistributed_blocks: HashSet<String> = blocks_for_sync
            .iter()
            .map(|block| block.id.clone())
            .collect();

        let staging_client = StagingServiceClient::new(
            ctx.secrets().service_key(),
            ctx.service_name(),
            &staging_host.name,
            Url::parse(&staging_host.url)?,
        );

        let mut blocks_grouped_by_metadata: HashMap<String, Vec<&Blocks>> = HashMap::new();
        for block in &blocks_for_sync {
            if !blocks_grouped_by_metadata
                .values()
                // deduplicate blocks across metadata
                .any(|blocks| blocks.iter().any(|b| b.id == block.id))
            {
                blocks_grouped_by_metadata
                    .entry(block.metadata_id.clone())
                    .or_default()
                    .push(block);
            }
        }

        for (metadata_id, grouped_blocks) in &blocks_grouped_by_metadata {
            tracing::info!("Redistributing blocks for metadata: {:?}", metadata_id);
            let metadata = Metadata::find_by_id(&database, metadata_id).await?;
            let user_id = Bucket::find_user_for_bucket(&database, &metadata.bucket_id).await?;

            let total_size = metadata
                .data_size
                .unwrap_or_default()
                .max(metadata.expected_data_size);

            let mut transaction = database.begin().await?;
            let new_storage_host = StorageHost::select_for_capacity_with_exclusion(
                &mut transaction,
                total_size,
                &[staging_host.id.clone()],
            )
            .await?;
            let authorization_grant = get_or_create_client_grant(
                &mut transaction,
                &user_id,
                total_size,
                &new_storage_host,
            )
            .await?;
            let block_cids: Vec<_> = grouped_blocks
                .iter()
                .map(|block| block.cid.clone())
                .collect();

            staging_client
                .distribute_data(DistributeDataRequest {
                    metadata_id: metadata_id.clone(),
                    storage_grant_id: authorization_grant.id.clone(),
                    storage_grant_size: authorization_grant.authorized_amount,
                    new_host_id: new_storage_host.id.clone(),
                    block_cids: block_cids.clone(),
                    new_host_url: new_storage_host.url.clone(),
                })
                .await?;

            let block_ids: Vec<_> = grouped_blocks
                .iter()
                .map(|block| block.id.clone())
                .collect();

            for block_id in block_ids.iter() {
                MinimalBlockLocation {
                    block_id: block_id.clone(),
                    metadata_id: metadata_id.clone(),
                    storage_host_id: new_storage_host.id.clone(),
                }
                .save(&mut transaction)
                .await?;
            }
            transaction.commit().await?;

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
}

impl RecurringTask for RedistributeStagingDataTask {
    fn next_schedule(&self) -> Result<Option<OffsetDateTime>, RecurringTaskError> {
        OffsetDateTime::now_utc()
            .checked_add(time::Duration::hours(1))
            .ok_or(RecurringTaskError::DateTimeAddition)
            .map(Some)
    }
}

pub async fn get_or_create_client_grant(
    conn: &mut DatabaseConnection,
    user_id: &String,
    total_size: i64,
    new_storage_host: &StorageHost,
) -> Result<ExistingStorageGrant, sqlx::Error> {
    let user_report = UserStorageReport::user_report(conn, &new_storage_host.id, user_id).await?;

    let authorization_grant = if user_report.authorization_available() < total_size
        || user_report.existing_grant().is_none()
    {
        let new_authorized_capacity = rounded_storage_authorization(&user_report, total_size);
        NewStorageGrant {
            storage_host_id: &new_storage_host.id,
            user_id,
            authorized_amount: new_authorized_capacity,
        }
        .save(conn)
        .await?
    } else {
        user_report.existing_grant().unwrap()
    };

    Ok(authorization_grant)
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
    use crate::database::models::{BlockLocations, MetadataState};
    use crate::database::test_helpers;
    use crate::tasks::redistribute_staging_data::RedistributeStagingDataTask;

    #[tokio::test]
    async fn test_block_state_changed() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let staging_host_id = test_helpers::create_storage_host(
            &mut conn,
            "staging-service",
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
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body("{}")
            .create_async()
            .await;

        let all_blocks = BlockLocations::find_all(&db).await.expect("get all blocks");
        assert_eq!(all_blocks.len(), block_ids.len());
        let res = RedistributeStagingDataTask::default()
            .run(CurrentTask::default(), mock_app_state(db.clone()).0)
            .await;

        println!("{:?}", res);

        assert!(res.is_ok());
        let all_block_locations = BlockLocations::find_all(&db).await.expect("get all blocks");
        assert_eq!(all_block_locations.len(), block_ids.len() * 2);
        for block_location in all_block_locations.iter() {
            assert_eq!(block_location.expired_at, None);
            assert_eq!(block_location.pruned_at, None);

            if block_location.storage_host_id == new_host_id {
                assert_eq!(block_location.stored_at, None);
            } else {
                assert_ne!(block_location.stored_at, None);
            }
        }
    }
}
