use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::database::models::{User, UserTotalConsumption};
use crate::database::DatabaseConnection;
use crate::utils::time::round_to_next_hour;

pub type StorageReporterTaskContext = AppState;

#[derive(Debug, thiserror::Error)]
pub enum StorageReporterTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),
}

#[derive(Deserialize, Serialize)]
pub struct ReportUserConsumptionTask {
    user_id: String,
}

impl ReportUserConsumptionTask {
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}

#[async_trait]
impl TaskLike for ReportUserConsumptionTask {
    const TASK_NAME: &'static str = "report_user_storage_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut db_conn = ctx.database().acquire().await?;
        // round up as it makes more sense when looking backwards
        let slot_end = round_to_next_hour(OffsetDateTime::now_utc())?;

        save_user_consumption(&mut db_conn, slot_end, &self.user_id).await?;

        Ok(())
    }
}

pub async fn save_user_consumption(
    conn: &mut DatabaseConnection,
    slot_end: OffsetDateTime,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    let user = User::by_id(conn, user_id).await?;
    let hot_storage_bytes = user.hot_usage(conn).await?.total();
    match UserTotalConsumption::find_by_slot_and_user(conn, slot_end, user_id).await {
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
            UserTotalConsumption {
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

    use crate::database::models::{Metadata, MetadataState, UserTotalConsumption};
    use crate::database::test_helpers::{
        create_user, sample_bucket, sample_metadata, setup_database,
    };
    use crate::database::{Database, DatabaseConnection};
    use crate::tasks::report_user_consumption::save_user_consumption;
    use crate::utils::time::round_to_next_hour;
    impl UserTotalConsumption {
        pub async fn find_all(conn: &Database) -> Result<Vec<Self>, sqlx::Error> {
            let result =
                sqlx::query_as!(UserTotalConsumption, "SELECT * FROM user_total_consumption")
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

        let slot_end = round_to_next_hour(OffsetDateTime::now_utc()).unwrap();
        let result = save_user_consumption(&mut conn, slot_end, &user_id).await;

        assert!(result.is_ok());
        let user_total_consumption = UserTotalConsumption::find_all(&db)
            .await
            .expect("user_total_consumption");
        assert_eq!(user_total_consumption.len(), 1);
        assert_eq!(user_total_consumption[0].user_id, user_id);
        assert_eq!(user_total_consumption[0].hot_storage_bytes, 130);
        assert_eq!(user_total_consumption[0].archival_storage_bytes, 0);
        assert_eq!(user_total_consumption[0].slot, slot_end);
    }
}
