use std::collections::HashSet;

use crate::database::models::ExplicitBigInt;
use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;

pub type UsedStorageTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum UsedStorageTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),

    #[error("http error: {0} response from {1}")]
    Http(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct UsedStorageTask {
    storage_host_id: String,
}

impl UsedStorageTask {
    pub fn new(storage_host_id: String) -> Self {
        Self { storage_host_id }
    }
}

#[async_trait]
impl TaskLike for UsedStorageTask {
    const TASK_NAME: &'static str = "used_storage_task";

    type Error = UsedStorageTaskError;
    type Context = UsedStorageTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await?;
        let storage_host_id = self.storage_host_id.clone();

        // Update used_storage by summing the metadata entries over data_size
        sqlx::query!(
            r#"
                UPDATE storage_hosts
                SET used_storage = (
                    SELECT COALESCE(SUM(m.data_size), 0) as big_int
                    FROM storage_hosts_metadatas_storage_grants shms
                    INNER JOIN metadata AS m ON m.id = shms.metadata_id 
                    WHERE shms.storage_host_id = $1
                )
                WHERE id = $1;
            "#,
            storage_host_id,
        )
        .execute(&mut *db_conn)
        .await?;

        tracing::info!(
            "storage_host.id: {} | used_storage updated",
            storage_host_id
        );

        // Update reserved_storage
        sqlx::query!(
            r#"
                UPDATE storage_hosts
                SET reserved_storage = (
	                SELECT SUM(sg.authorized_amount)
	                FROM storage_hosts sh
	                INNER JOIN (
                        SELECT user_id, storage_host_id, MAX(redeemed_at) as redeemed_at, authorized_amount 
                        FROM storage_grants
                        GROUP BY user_id
	                ) AS sg 
	                WHERE sg.storage_host_id = sh.id 
                    AND sh.id = $1
	                AND sg.redeemed_at <> NULL
	                ORDER BY sg.redeemed_at
                );
            "#,
            storage_host_id,
        )
        .execute(&mut *db_conn)
        .await?;

        tracing::info!(
            "storage_host.id: {} | reserved_storage updated",
            storage_host_id
        );

        Ok(())
    }
}
