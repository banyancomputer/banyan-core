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

        // We need the `authorized_amount` field in the `storage_grants` table
        // We need the `reserved_storage` field in the `storage_hosts` table
        //
        // We need to sum the metadata entries `data_size` columns associated with the storage
        // host. Ignore state. Calculate the `reserved_storage` amount by summing all the
        // `authorized_amount` columns for most recently redeemed (limit one per user.).

        let storage_host_id = self.storage_host_id.clone();
        let total_data_size = sqlx::query_as!(
            ExplicitBigInt,
            r#"
                SELECT COALESCE(SUM(m.data_size), 0) as big_int
                FROM storage_hosts_metadatas_storage_grants shms
                INNER JOIN metadata AS m ON m.id = shms.metadata_id 
                WHERE shms.storage_host_id = $1;
            "#,
            storage_host_id,
        )
        .fetch_one(&mut *db_conn)
        .await?;

        Ok(())

        /*
        // Send the request and handle the response
        let response = request
            .send()
            .await
            .map_err(UsedStorageTaskError::Reqwest)?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(UsedStorageTaskError::Http(
                response.status(),
                storage_host_url,
            ))
        }
        */
    }
}

#[derive(sqlx::FromRow)]
struct StorageHostInfo {
    pub url: String,
    pub name: String,
}
