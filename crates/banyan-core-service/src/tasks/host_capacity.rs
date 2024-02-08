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

        println!(
            "storage_host.id: {} | used_storage updated",
            storage_host_id
        );

        // Update reserved_storage
        sqlx::query!(
            r#"
                UPDATE storage_hosts
                SET reserved_storage = 
                COALESCE(
	                (
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
                    ), 
                0);
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
    use crate::database::models::{Metadata, MetadataState, NewStorageGrant};
    use crate::database::test_helpers::*;

    pub async fn get_stats(ctx: HostCapacityTaskContext, storage_host_id: &str) -> (i64, i64) {
        let mut db_conn = ctx.db_pool().acquire().await.unwrap();

        let used_storage = sqlx::query_scalar!(
            r#"
                SELECT used_storage
                FROM storage_hosts
                WHERE id = $1;
            "#,
            storage_host_id
        )
        .fetch_one(&mut *db_conn)
        .await
        .expect("get used_storage");

        let reserved_storage = sqlx::query_scalar!(
            r#"
                SELECT reserved_storage
                FROM storage_hosts
                WHERE id = $1;
            "#,
            storage_host_id
        )
        .fetch_one(&mut *db_conn)
        .await
        .expect("get used_storage");

        (used_storage, reserved_storage)
    }

    /// Return a base context and a test account id
    pub async fn test_setup() -> (HostCapacityTaskContext, Uuid, CurrentTask) {
        let (ctx, storage_host_id) = host_capacity_context().await;
        (ctx, storage_host_id, default_current_task())
    }

    #[tokio::test]
    async fn success() {
        let (ctx, storage_host_id, current_task) = test_setup().await;
        let task = HostCapacityTask::new(storage_host_id.to_string());
        let result = task.run(current_task, ctx.clone()).await;
        println!("result: {:?}", result);
        assert!(result.is_ok());

        let (used_storage, reserved_storage) = get_stats(ctx, &storage_host_id.to_string()).await;
        println!("used: {}, reserved: {}", used_storage, reserved_storage);
    }

    async fn host_capacity_context() -> (HostCapacityTaskContext, Uuid) {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        // Register storage host
        let storage_host_id = create_storage_hosts(
            &mut conn,
            "host_url",
            "00000000-0000-1234-0000-000000000000",
        )
        .await
        .expect("create storage host");
        // Create users
        let dog_user_id = create_user(&mut conn, "dog@com.example", "dog").await;
        let cat_user_id = create_user(&mut conn, "cat@com.example", "cat").await;
        // Create buckets
        let dog_bucket_id = create_hot_bucket(&mut conn, &dog_user_id, "dog files").await;
        let cat_bucket_id = create_hot_bucket(&mut conn, &cat_user_id, "cat files").await;

        // Create metadatas
        let dog_metadata_id_1 = create_metadata(
            &mut conn,
            &dog_bucket_id,
            "dm1",
            "dr1",
            MetadataState::Pending,
            None,
            None,
        )
        .await;
        let dog_metadata_id_2 = create_metadata(
            &mut conn,
            &dog_bucket_id,
            "dm2",
            "dm2",
            MetadataState::Pending,
            None,
            None,
        )
        .await;
        let cat_metadata_id_1 = create_metadata(
            &mut conn,
            &cat_bucket_id,
            "cm1",
            "cr1",
            MetadataState::Pending,
            None,
            None,
        )
        .await;
        let cat_metadata_id_2 = create_metadata(
            &mut conn,
            &cat_bucket_id,
            "cm2",
            "cr2",
            MetadataState::Pending,
            None,
            None,
        )
        .await;

        // Create storage grants for uploading
        let dog_storage_grant_id_1 =
            create_storage_grant(&mut conn, &storage_host_id, &dog_user_id, 1000).await;
        let dog_storage_grant_id_2 =
            create_storage_grant(&mut conn, &storage_host_id, &dog_user_id, 2000).await;
        let cat_storage_grant_id_1 =
            create_storage_grant(&mut conn, &storage_host_id, &cat_user_id, 1000).await;
        let cat_storage_grant_id_2 =
            create_storage_grant(&mut conn, &storage_host_id, &cat_user_id, 2000).await;

        // Redeem both dog grants
        redeem_storage_grant(&mut conn, &storage_host_id, &dog_storage_grant_id_1).await;
        associate_upload(
            &mut conn,
            &storage_host_id,
            &dog_metadata_id_1,
            &dog_storage_grant_id_1,
        )
        .await;
        redeem_storage_grant(&mut conn, &storage_host_id, &dog_storage_grant_id_2).await;
        associate_upload(
            &mut conn,
            &storage_host_id,
            &dog_metadata_id_2,
            &dog_storage_grant_id_2,
        )
        .await;
        // Redeem only the most recent cat grant
        redeem_storage_grant(&mut conn, &storage_host_id, &cat_storage_grant_id_2).await;
        associate_upload(
            &mut conn,
            &storage_host_id,
            &cat_metadata_id_2,
            &cat_storage_grant_id_2,
        )
        .await;

        Metadata::mark_current(&mut conn, &dog_bucket_id, &dog_metadata_id_1, Some(1000))
            .await
            .expect("mark current");
        Metadata::mark_current(&mut conn, &dog_bucket_id, &dog_metadata_id_2, Some(2000))
            .await
            .expect("mark current");
        Metadata::mark_current(&mut conn, &cat_bucket_id, &cat_metadata_id_1, Some(1000))
            .await
            .expect("mark current");
        Metadata::mark_current(&mut conn, &cat_bucket_id, &cat_metadata_id_2, Some(2000))
            .await
            .expect("mark current");

        conn.commit().await.expect("failed to commit transaction");

        println!("shid: {}", storage_host_id);
        (
            HostCapacityTaskContext::new(db),
            Uuid::parse_str(&storage_host_id).expect("uuid parse"),
        )
    }
}
