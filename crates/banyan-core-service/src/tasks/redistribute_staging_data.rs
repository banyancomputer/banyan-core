use std::collections::HashSet;

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::auth::STAGING_SERVICE_NAME;
use crate::clients::{DistributeDataRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::{Metadata, StorageHost, StorageHostsMetadatasStorageGrants};

#[derive(Deserialize, Serialize)]
pub struct RedistributeStagingDataTask {}

impl RedistributeStagingDataTask {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl TaskLike for RedistributeStagingDataTask {
    const TASK_NAME: &'static str = "redistribute_staging_data";

    type Error = RedistributeStagingDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let staging_host = StorageHost::select_by_name(&database, STAGING_SERVICE_NAME).await?;
        let metadata = Metadata::get_by_storage_host_id(&database, &staging_host.id).await?;
        let mut undistributed_metadata: HashSet<String> = metadata
            .iter()
            .map(|metadata| metadata.id.clone())
            .collect();

        let staging_client = StagingServiceClient::new(
            ctx.secrets().service_key(),
            ctx.service_name(),
            staging_host.name.as_str(),
            Url::parse(&staging_host.url)?,
        );

        for metadata in metadata.iter() {
            tracing::info!("Redistributing blocks for metadata: {:?}", metadata.id);
            let metadata_id = &metadata.id;
            let metadata = Metadata::get_by_id(&database, metadata_id).await?;
            let grant_metadata =
                StorageHostsMetadatasStorageGrants::find_by_metadata_id(&database, metadata_id)
                    .await?;
            let total_size = metadata.metadata_size.unwrap_or_default()
                + metadata
                    .data_size
                    .unwrap_or_default()
                    .max(metadata.expected_data_size);
            let new_storage_host = StorageHost::select_for_capacity_with_exclusion(
                &database,
                total_size,
                staging_host.id.as_str(),
            )
            .await?;

            staging_client
                .distribute_data(DistributeDataRequest {
                    metadata_id: metadata_id.clone(),
                    grant_id: grant_metadata.storage_grant_id.clone(),
                    new_host_id: new_storage_host.id.clone(),
                    new_host_url: new_storage_host.url.clone(),
                })
                .await?;

            undistributed_metadata.remove(metadata_id);
        }

        if !undistributed_metadata.is_empty() {
            tracing::warn!(
                "Not all metadata have been distributed. Remaining: {:?}",
                undistributed_metadata
            );
        }
        Ok(())
    }

    fn next_time(&self) -> Option<OffsetDateTime> {
        Some(OffsetDateTime::now_utc() + time::Duration::seconds(5))
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
