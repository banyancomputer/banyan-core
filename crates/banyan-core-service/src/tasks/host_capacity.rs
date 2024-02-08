use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::OffsetDateTime;

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

        #[derive(Debug, sqlx::FromRow)]
        struct StorageGrant {
            user_id: String,
            storage_host_id: String,
            redeemed_at: Option<OffsetDateTime>,
            authorized_amount: i64,
        }

        let qualifying_grants = sqlx::query_as!(
            StorageGrant,
            r#"
                SELECT a.user_id, a.storage_host_id, a.redeemed_at, a.authorized_amount 
                FROM storage_grants a
                WHERE a.redeemed_at IN (
                    SELECT b.redeemed_at
                    FROM storage_grants b
                    WHERE a.user_id = b.user_id
                    AND b.redeemed_at IS NOT NULL
                    ORDER BY redeemed_at ASC
                    LIMIT 1
                ) 
                GROUP BY user_id;
            "#
        )
        .fetch_all(&mut *db_conn)
        .await?;

        println!("qualifying grants: {:?}", qualifying_grants);

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
                            SELECT a.user_id, a.storage_host_id, a.redeemed_at, a.authorized_amount 
                            FROM storage_grants a
                            WHERE a.redeemed_at IN (
                                SELECT b.redeemed_at
                                FROM storage_grants b
                                WHERE a.user_id = b.user_id
                                AND b.redeemed_at IS NOT NULL
                                ORDER BY redeemed_at ASC
                                LIMIT 1
                            ) 
                            GROUP BY user_id
	                    ) AS sg 
	                    WHERE sg.storage_host_id = sh.id 
                        AND sh.id = $1
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
    use crate::database::models::{Metadata, MetadataState};
    use crate::database::test_helpers::*;

    const DOG_UPLOAD_1: i64 = 500;
    const DOG_UPLOAD_2: i64 = 1000;
    const CAT_UPLOAD_1: i64 = 750;
    const CAT_UPLOAD_2: i64 = 1275;

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

        // The first cat upload is never marked current
        // So the `data_size` never updates
        assert_eq!(used_storage, DOG_UPLOAD_1 + DOG_UPLOAD_2 + CAT_UPLOAD_2);
        // The reserved storage should represent the sum of authorized_amount for redeemed storage
        // grants, but only one per user.
        assert_eq!(reserved_storage, DOG_UPLOAD_2 + CAT_UPLOAD_2);
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
            create_storage_grant(&mut conn, &storage_host_id, &dog_user_id, DOG_UPLOAD_1).await;
        let dog_storage_grant_id_2 =
            create_storage_grant(&mut conn, &storage_host_id, &dog_user_id, DOG_UPLOAD_2).await;
        let cat_storage_grant_id_1 =
            create_storage_grant(&mut conn, &storage_host_id, &cat_user_id, CAT_UPLOAD_1).await;
        let cat_storage_grant_id_2 =
            create_storage_grant(&mut conn, &storage_host_id, &cat_user_id, CAT_UPLOAD_2).await;

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
        associate_upload(
            &mut conn,
            &storage_host_id,
            &cat_metadata_id_1,
            &cat_storage_grant_id_1,
        )
        .await;
        redeem_storage_grant(&mut conn, &storage_host_id, &cat_storage_grant_id_2).await;
        associate_upload(
            &mut conn,
            &storage_host_id,
            &cat_metadata_id_2,
            &cat_storage_grant_id_2,
        )
        .await;

        Metadata::mark_current(
            &mut conn,
            &dog_bucket_id,
            &dog_metadata_id_1,
            Some(DOG_UPLOAD_1),
        )
        .await
        .expect("mark current");
        Metadata::mark_current(
            &mut conn,
            &dog_bucket_id,
            &dog_metadata_id_2,
            Some(DOG_UPLOAD_2),
        )
        .await
        .expect("mark current");
        Metadata::mark_current(
            &mut conn,
            &cat_bucket_id,
            &cat_metadata_id_2,
            Some(CAT_UPLOAD_2),
        )
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
