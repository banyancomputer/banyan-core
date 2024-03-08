use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::database::models::{MetricsStorage, User};
use crate::database::DatabaseConnection;
use crate::utils::time::round_to_previous_hour;

pub type StorageReporterTaskContext = AppState;

#[derive(Debug, thiserror::Error)]
pub enum StorageReporterTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),
}

#[derive(Deserialize, Serialize)]
pub struct ReportUserStorage {
    user_id: String,
}

impl ReportUserStorage {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}

#[async_trait]
impl TaskLike for ReportUserStorage {
    const TASK_NAME: &'static str = "report_user_storage_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await?;
        let slot_end = round_to_previous_hour(OffsetDateTime::now_utc())?;

        calculate_and_store_consumed_storage(&mut db_conn, slot_end, &self.user_id).await?;

        Ok(())
    }
}

pub async fn calculate_and_store_consumed_storage(
    conn: &mut DatabaseConnection,
    slot_end: OffsetDateTime,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    let user = User::by_id(conn, user_id).await?;
    let hot_storage_bytes = user.consumed_storage(conn).await?;
    match MetricsStorage::find_by_slot_and_user(conn, slot_end, user_id).await {
        Ok(Some(existing_metrics)) => {
            let updated_hot_storage_bytes =
                std::cmp::max(existing_metrics.hot_storage_bytes, hot_storage_bytes);
            existing_metrics
                .update(
                    conn,
                    updated_hot_storage_bytes,
                    existing_metrics.archival_storage_bytes,
                )
                .await?;
        }
        Ok(None) => {
            MetricsStorage {
                user_id: user.id.clone(),
                hot_storage_bytes,
                archival_storage_bytes: 0,
                slot: slot_end,
            }
            .save(conn)
            .await?;
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use time::OffsetDateTime;

    use crate::database::models::{Metadata, MetadataState, MetricsStorage};
    use crate::database::test_helpers::{
        create_user, sample_bucket, sample_metadata, setup_database,
    };
    use crate::database::{Database, DatabaseConnection};
    use crate::tasks::report_user_storage::calculate_and_store_consumed_storage;
    use crate::utils::time::round_to_previous_hour;
    impl MetricsStorage {
        pub async fn find_all(conn: &Database) -> Result<Vec<Self>, sqlx::Error> {
            let result = sqlx::query_as!(MetricsStorage, "SELECT * FROM metrics_storage")
                .fetch_all(conn)
                .await?;
            Ok(result)
        }
    }
    impl Metadata {
        pub async fn update_size(
            conn: &mut DatabaseConnection,
            metadata_id: &str,
            data_size: i64,
            metadata_size: i64,
        ) -> Result<(), sqlx::Error> {
            sqlx::query!(
                "UPDATE metadata SET data_size = $1, metadata_size = $2 WHERE id = $3",
                data_size,
                metadata_size,
                metadata_id,
            )
            .execute(conn)
            .await?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn report_user_storage_test() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let user_id = create_user(&mut conn, "test@example.com", "Test User").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        Metadata::update_size(&mut conn, &metadata_id, 100, 30)
            .await
            .expect("metadata size update");

        let slot_end = round_to_previous_hour(OffsetDateTime::now_utc()).unwrap();
        let result = calculate_and_store_consumed_storage(&mut conn, slot_end, &user_id).await;

        assert!(result.is_ok());
        let metrics_storage = MetricsStorage::find_all(&db)
            .await
            .expect("metrics_storage");
        assert_eq!(metrics_storage.len(), 1);
        assert_eq!(metrics_storage[0].user_id, user_id);
        assert_eq!(metrics_storage[0].hot_storage_bytes, 130);
        assert_eq!(metrics_storage[0].archival_storage_bytes, 0);
        assert_eq!(metrics_storage[0].slot, slot_end);
    }
}
