use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::database::models::StorageHost;

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
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await.unwrap();
        let storage_host_id = self.storage_host_id.clone();

        // Update used_storage by summing the metadata entries over data_size
        let total_consumption =
            StorageHost::total_consumption(&mut db_conn, &storage_host_id).await?;
        sqlx::query!(
            r#"
                UPDATE storage_hosts
                SET used_storage = $2
                WHERE id = $1;
            "#,
            storage_host_id,
            total_consumption
        )
        .execute(&mut *db_conn)
        .await?;

        // Update reserved_storage
        // Ensure that we are only summing authorized amounts on one storage grant per user, taking
        // care to sort those grants by redemption time and ensure the redemption time is not null
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
                                SELECT MAX(b.redeemed_at)
                                FROM storage_grants b
                                WHERE a.user_id = b.user_id
                            )
                            GROUP BY a.id
	                    ) AS sg 
	                    WHERE sg.storage_host_id = sh.id 
                        AND sh.id = $1
                    ), 
                0)
                WHERE id = $1;
            "#,
            storage_host_id,
        )
        .execute(&mut *db_conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::app::mock_app_state;
    use crate::database::models::{Metadata, MetadataState};
    use crate::database::test_helpers::*;

    const STORAGE_HOST_1: &str = "00000000-0000-1234-0000-000000000000";
    const STORAGE_HOST_2: &str = "00000000-0000-5678-0000-000000000000";
    const DOG_UPLOAD_1: i64 = 500;
    const DOG_UPLOAD_2: i64 = 1000;
    const CAT_UPLOAD_1: i64 = 750;
    const CAT_UPLOAD_2: i64 = 1275;

    #[derive(Clone)]
    pub struct StorageHosts {
        storage_host_id_1: String,
        storage_host_id_2: String,
    }

    pub async fn get_stats(ctx: AppState, storage_host_id: &str) -> (i64, i64) {
        let mut db_conn = ctx.database().acquire().await.unwrap();

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
    pub async fn test_setup() -> (AppState, CurrentTask, StorageHosts) {
        let (ctx, storage_hosts) = host_capacity_context().await;
        (ctx, CurrentTask::default(), storage_hosts)
    }

    #[tokio::test]
    async fn success() {
        let (ctx, current_task, storage_hosts) = test_setup().await;
        assert!(
            HostCapacityTask::new(String::from(storage_hosts.storage_host_id_1.as_str()))
                .run(current_task, ctx.clone())
                .await
                .is_ok()
        );
        let (used_storage, reserved_storage) =
            get_stats(ctx.clone(), storage_hosts.storage_host_id_1.as_str()).await;
        println!("used: {}, reserved: {}", used_storage, reserved_storage);

        // The first cat upload is never marked current
        // So the `data_size` never updates
        assert_eq!(used_storage, DOG_UPLOAD_1 + DOG_UPLOAD_2 + CAT_UPLOAD_2);
        // The reserved storage should represent the sum of authorized_amount for redeemed storage
        // grants, but only one per user. Additionally, the 'fake grant' authorized storage is not
        // factored into this result because it was uploaded to a different storage provider.
        assert_eq!(reserved_storage, DOG_UPLOAD_2 + CAT_UPLOAD_2);

        // Do the same for the other storage host and assert it is empty
        assert!(
            HostCapacityTask::new(String::from(storage_hosts.storage_host_id_2.as_str()))
                .run(CurrentTask::default(), ctx.clone())
                .await
                .is_ok()
        );
        let (used_storage, reserved_storage) =
            get_stats(ctx.clone(), storage_hosts.storage_host_id_2.as_str()).await;
        assert_eq!(used_storage, 0);
        assert_eq!(reserved_storage, 0);
    }

    async fn host_capacity_context() -> (AppState, StorageHosts) {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.begin().await.expect("connection");

        // Register storage host
        let storage_host_1 = create_storage_hosts(&mut conn, "url1", STORAGE_HOST_1).await;
        let storage_host_2 = create_storage_hosts(&mut conn, "url2", STORAGE_HOST_2).await;

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
        let dog_storage_grant_id_1 = create_storage_grant(
            &mut conn,
            storage_host_1.as_str(),
            &dog_user_id,
            DOG_UPLOAD_1,
        )
        .await;
        let dog_storage_grant_id_2 = create_storage_grant(
            &mut conn,
            storage_host_1.as_str(),
            &dog_user_id,
            DOG_UPLOAD_2,
        )
        .await;
        let cat_storage_grant_id_1 = create_storage_grant(
            &mut conn,
            storage_host_1.as_str(),
            &cat_user_id,
            CAT_UPLOAD_1,
        )
        .await;
        let cat_fake_grant = create_storage_grant(
            &mut conn,
            storage_host_2.as_str(),
            &cat_user_id,
            CAT_UPLOAD_1,
        )
        .await;
        let cat_storage_grant_id_2 = create_storage_grant(
            &mut conn,
            storage_host_1.as_str(),
            &cat_user_id,
            CAT_UPLOAD_2,
        )
        .await;

        // Redeem both dog grants
        redeem_storage_grant(&mut conn, storage_host_1.as_str(), &dog_storage_grant_id_1).await;
        associate_upload(
            &mut conn,
            storage_host_1.as_str(),
            &dog_metadata_id_1,
            &dog_storage_grant_id_1,
        )
        .await;
        thread::sleep(Duration::from_millis(1000));
        redeem_storage_grant(&mut conn, storage_host_1.as_str(), &dog_storage_grant_id_2).await;
        associate_upload(
            &mut conn,
            storage_host_1.as_str(),
            &dog_metadata_id_2,
            &dog_storage_grant_id_2,
        )
        .await;
        // Redeem the fake and most recent cat grant
        associate_upload(
            &mut conn,
            storage_host_1.as_str(),
            &cat_metadata_id_1,
            &cat_storage_grant_id_1,
        )
        .await;
        redeem_storage_grant(&mut conn, storage_host_2.as_str(), &cat_fake_grant).await;
        associate_upload(
            &mut conn,
            storage_host_2.as_str(),
            &cat_metadata_id_1,
            &cat_fake_grant,
        )
        .await;
        thread::sleep(Duration::from_millis(1000));
        redeem_storage_grant(&mut conn, storage_host_1.as_str(), &cat_storage_grant_id_2).await;
        associate_upload(
            &mut conn,
            storage_host_1.as_str(),
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

        (
            state.0,
            StorageHosts {
                storage_host_id_1: storage_host_1,
                storage_host_id_2: storage_host_2,
            },
        )
    }
}
