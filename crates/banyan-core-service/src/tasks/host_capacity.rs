use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::database::Database;

#[derive(Clone)]
pub struct HostCapacityTaskContext {
    db_pool: SqlitePool,
}

#[allow(dead_code)]
impl HostCapacityTaskContext {
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum HostCapacityTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(Deserialize, Serialize)]
pub struct HostCapacityTask {
    storage_host_id: String,
}

impl HostCapacityTask {
    pub fn new(storage_host_id: String) -> Self {
        Self { storage_host_id }
    }
}

#[async_trait]
impl TaskLike for HostCapacityTask {
    const TASK_NAME: &'static str = "used_storage_task";

    type Error = HostCapacityTaskError;
    type Context = HostCapacityTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();
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

    fn unique_key(&self) -> Option<String> {
        Some(self.storage_host_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use banyan_task::tests::default_current_task;
    use banyan_task::{CurrentTask, TaskLike};
    use uuid::Uuid;

    use super::*;
    use crate::database::models::MetadataState;
    use crate::database::test_helpers::*;

    const STORAGE_HOST_ID: &str = "00000000-0000-0000-0000-000000000000";
    const USER_EMAIL: &str = "user@user.email";
    // const STORAGE_HOST_ID: &str = "00000000-0000-1234-0000-000000000000";
    const STORAGE_HOST_URL: &str = "http://127.0.0.1:3009";

    /// Return a base context and a test account id
    pub async fn test_setup() -> (HostCapacityTaskContext, Uuid, CurrentTask) {
        (
            host_capacity_context().await,
            Uuid::parse_str(STORAGE_HOST_ID).expect("account id parse"),
            default_current_task(),
        )
    }

    #[tokio::test]
    /// ScheduledMaintenanceEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, storage_host_id, current_task) = test_setup().await;
        let task = HostCapacityTask::new(storage_host_id.to_string());
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }

    async fn host_capacity_context() -> HostCapacityTaskContext {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        // Register storage host
        let storage_host_id = create_storage_hosts(&mut conn, "host_url", "host_name")
            .await
            .expect("create storage host");
        // Create users
        let dog_user_id = create_user(&mut conn, "dog@com.example", "dog").await;
        let cat_user_id = create_user(&mut conn, "cat@com.example", "cat").await;
        // Create buckets
        let dog_bucket_id = create_hot_bucket(&mut conn, &dog_user_id, "dog files").await;
        let cat_bucket_id = create_hot_bucket(&mut conn, &cat_user_id, "cat files").await;

        // Create two metadatas per bucket
        let dog_metadata_id_1 = create_metadata(
            &mut conn,
            &dog_bucket_id,
            "dm1",
            "dr1",
            MetadataState::Outdated,
            None,
            None,
        )
        .await;
        let dog_metadata_id_2 = create_metadata(
            &mut conn,
            &dog_bucket_id,
            "dm2",
            "dm2",
            MetadataState::Current,
            None,
            None,
        )
        .await;
        let cat_metadata_id_1 = create_metadata(
            &mut conn,
            &cat_bucket_id,
            "cm1",
            "cr1",
            MetadataState::Outdated,
            None,
            None,
        )
        .await;
        let cat_metadata_id_2 = create_metadata(
            &mut conn,
            &cat_bucket_id,
            "cm2",
            "cm2",
            MetadataState::Current,
            None,
            None,
        )
        .await;

        HostCapacityTaskContext::new(db)
    }
}
