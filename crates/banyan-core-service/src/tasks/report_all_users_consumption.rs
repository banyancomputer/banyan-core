use async_trait::async_trait;
use banyan_task::{CurrentTask, RecurringTask, TaskLike};
use serde::{Deserialize, Serialize};
use time::error::ComponentRange;
use time::OffsetDateTime;

use crate::app::AppState;
use crate::tasks::report_user_consumption::save_user_consumption;
use crate::utils::time::round_to_next_hour;

pub type StorageReporterTaskContext = AppState;

#[derive(Debug, thiserror::Error)]
pub enum StorageReporterTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("could not calculate end slot: {0}")]
    EndSlotParsingError(#[from] ComponentRange),
}

#[derive(Deserialize, Serialize, Default)]
pub struct ReportAllUsersConsumptionTask {}

#[async_trait]
impl TaskLike for ReportAllUsersConsumptionTask {
    const TASK_NAME: &'static str = "report_all_users_consumption_task";

    type Error = StorageReporterTaskError;
    type Context = StorageReporterTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut conn = ctx.database().acquire().await?;
        let slot_end = round_to_next_hour(OffsetDateTime::now_utc())?;

        let users = sqlx::query_scalar!("SELECT DISTINCT user_id FROM buckets")
            .fetch_all(&mut *conn)
            .await?;

        for user_id in users {
            save_user_consumption(&mut conn, slot_end, &user_id).await?;
        }

        Ok(())
    }
}

impl RecurringTask for ReportAllUsersConsumptionTask {
    fn next_schedule(&self) -> Result<Option<OffsetDateTime>, String> {
        Ok(Some(OffsetDateTime::now_utc() + time::Duration::hours(1)))
    }
}

#[cfg(test)]
mod test {
    use banyan_task::{CurrentTask, TaskLike};

    use crate::app::mock_app_state;
    use crate::database::models::{Metadata, MetadataState, UserTotalConsumption};
    use crate::database::test_helpers::{
        create_user, sample_bucket, sample_metadata, setup_database,
    };
    use crate::database::DatabaseConnection;
    use crate::tasks::report_all_users_consumption::ReportAllUsersConsumptionTask;

    pub async fn setup_user_and_data(conn: &mut DatabaseConnection, email: &str) -> String {
        let user_id = create_user(conn, email, "Test User").await;
        let bucket_id = sample_bucket(conn, &user_id).await;
        let metadata_id = sample_metadata(conn, &bucket_id, 1, MetadataState::Current).await;
        Metadata::update_size(conn, &metadata_id, 100, 30)
            .await
            .expect("metadata size update");
        user_id
    }

    #[tokio::test]
    async fn report_all_users_storage_test() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.acquire().await.expect("connection");
        setup_user_and_data(&mut conn, "test1@example.com").await;
        setup_user_and_data(&mut conn, "test2@example.com").await;

        let result = ReportAllUsersConsumptionTask::default()
            .run(CurrentTask::default(), state.0)
            .await;

        assert!(result.is_ok());
        let metrics_storage = UserTotalConsumption::find_all(&db)
            .await
            .expect("metrics_storage");
        assert_eq!(metrics_storage.len(), 2);
    }
}
