use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike, TaskLikeExt, TaskStoreError};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::AppState;
use crate::clients::core_service::{CoreServiceClient, MoveMetadataRequest};
use crate::database::models::{Blocks, Uploads};
use crate::tasks::setup_upload_blocks::SetupUploadBlocksTask;
use crate::tasks::upload_blocks::UploadBlocksTask;

pub type ReportUploadTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RedistributeDataTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("scheduling task error: {0}")]
    SchedulingTaskError(#[from] TaskStoreError),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct RedistributeDataTask {}

impl RedistributeDataTask {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskLike for RedistributeDataTask {
    const TASK_NAME: &'static str = "redistribute_data_task";

    type Error = RedistributeDataTaskError;
    type Context = ReportUploadTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut database = ctx.database();

        let core_client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );

        let uploads_for_redistribution: Vec<Uploads> = Uploads::non_pruned_uploads(&database)
            .await
            .map_err(RedistributeDataTaskError::DatabaseError)?;

        let mut undistributed_uploads: HashSet<String> = uploads_for_redistribution
            .iter()
            .map(|upload| upload.id.clone())
            .collect();

        for upload in uploads_for_redistribution.iter() {
            tracing::info!("Redistributing blocks for upload: {:?}", upload.id);

            let upload_id = upload.id.clone();
            let task_exists = sqlx::query!(
                "SELECT * FROM background_tasks WHERE task_name = $1 AND unique_key = $2",
                SetupUploadBlocksTask::TASK_NAME,
                upload_id
            )
            .fetch_optional(&database)
            .await
            .map_err(RedistributeDataTaskError::DatabaseError)?;

            if task_exists.is_some() {
                undistributed_uploads.remove(&upload_id);
                continue;
            }

            // TODO: what's going to happen if the user deletes the file while we redistribute?
            let blocks_for_pruning = match Blocks::blocks_for_upload(&database, &upload_id).await {
                Ok(blocks) => blocks,
                Err(e) => {
                    tracing::error!("Error getting blocks for upload: {:?}", e);
                    continue;
                }
            };

            let needed_capacity = upload
                .final_size
                .unwrap_or(upload.reported_size)
                .max(upload.reported_size);
            let previous_cids: Vec<String> = blocks_for_pruning
                .iter()
                .map(|block| block.cid.clone())
                .collect();
            let metadata_move_response = match core_client
                .initiate_metadata_move(
                    &upload.metadata_id,
                    MoveMetadataRequest {
                        needed_capacity,
                        previous_cids,
                    },
                )
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!("Error initiating metadata move: {:?}", e);
                    continue;
                }
            };

            let task_result = SetupUploadBlocksTask {
                metadata_id: upload.metadata_id.clone(),
                upload_id: upload.id.clone(),
                storage_host: metadata_move_response.storage_host.clone(),
                storage_authorization: metadata_move_response.storage_authorization.clone(),
            }
            .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
            .await;

            if let Err(_) = task_result {
                tracing::error!(
                    "could not schedule: {:?} for upload {:?} to storage host {:?}",
                    UploadBlocksTask::TASK_NAME,
                    upload_id,
                    metadata_move_response.storage_host.clone()
                );
                continue;
            }
            undistributed_uploads.remove(&upload_id);
        }

        if !undistributed_uploads.is_empty() {
            tracing::warn!(
                "Not all uploads have been distributed. Remaining: {:?}",
                undistributed_uploads
            );
        }
        Ok(())
    }

    fn next_time(&self) -> Option<OffsetDateTime> {
        Some(OffsetDateTime::now_utc() + time::Duration::days(1))
    }
}
