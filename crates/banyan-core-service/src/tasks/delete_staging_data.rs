use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::{
    DeleteBlocksRequest, GrantResetRequest, StagingServiceClient, StagingServiceError,
};
use crate::database::models::{
    ExistingStorageGrant, NewStorageGrant, StorageHost, StorageHostsMetadatasStorageGrants,
};
use crate::utils::minimal_grant_amount;

#[derive(Deserialize, Serialize)]
pub struct DeleteStagingDataTask {
    normalized_cids: Vec<String>,
    metadata_id: String,
}

impl DeleteStagingDataTask {
    pub fn new(metadata_id: String, normalized_cids: Vec<String>) -> Self {
        Self {
            metadata_id,
            normalized_cids,
        }
    }
}
#[async_trait]
impl TaskLike for DeleteStagingDataTask {
    const TASK_NAME: &'static str = "delete_staging_data_task";

    type Error = DeleteStagingDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let staging_host = StorageHost::select_staging(&database).await?;
        let grant_metadata = StorageHostsMetadatasStorageGrants::find_by_metadata_and_storage_host(
            &database,
            &self.metadata_id,
            &staging_host.id,
        )
        .await?;
        let latest_grant_for_staging =
            ExistingStorageGrant::latest_grant_for_host(&database, &staging_host.id).await?;
        let mut reset_storage_grant_request = None;
        // It means that the user did not request a new storage_grant by the time the block redistribution has completed.
        // Consequently, we need to manually lower the allocated storage for the staging service for that user,
        // since the blocks are no longer stored on the staging service
        if latest_grant_for_staging.id == grant_metadata.storage_grant_id {
            let mut conn = database.begin().await?;
            let reset_storage_grant = NewStorageGrant {
                storage_host_id: &staging_host.id,
                user_id: &latest_grant_for_staging.user_id,
                authorized_amount: minimal_grant_amount(),
            }
            .save(&mut conn)
            .await?;
            conn.commit().await?;

            reset_storage_grant_request = Some(GrantResetRequest {
                old_grant_id: latest_grant_for_staging.id,
                new_grant_id: reset_storage_grant.id.clone(),
                new_grant_size: reset_storage_grant.authorized_amount,
            });
        }
        let staging_client = StagingServiceClient::new(
            ctx.secrets().service_key(),
            ctx.service_name(),
            staging_host.name.as_str(),
            Url::parse(&staging_host.url)?,
        );

        staging_client
            .delete_blocks(DeleteBlocksRequest {
                metadata_id: self.metadata_id.clone(),
                normalized_cids: self.normalized_cids.clone(),
                reset_storage_grant: reset_storage_grant_request,
            })
            .await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteStagingDataTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("staging host url error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("staging error: {0}")]
    StagingServiceError(#[from] StagingServiceError),
}
