use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use time::OffsetDateTime;

use crate::task_store::TaskStore;
use crate::{CurrentTask, TaskStoreError};

#[async_trait]
pub trait TaskLike: Serialize + DeserializeOwned + Sync + Send + 'static {
    const MAX_ATTEMPTS: i64 = 3;
    const QUEUE_NAME: &'static str = "default";
    const TASK_NAME: &'static str;

    type Error: std::error::Error;
    type Context: Clone + Send + 'static;

    async fn run(&self, task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error>;

    fn unique_key(&self) -> Option<String> {
        None
    }
}

#[async_trait]
pub trait TaskLikeExt {
    async fn enqueue<S: TaskStore>(
        self,
        conn: &mut S::Connection,
    ) -> Result<Option<String>, TaskStoreError>;
}

#[async_trait]
impl<T> TaskLikeExt for T
where
    T: TaskLike,
{
    async fn enqueue<S: TaskStore>(
        self,
        conn: &mut S::Connection,
    ) -> Result<Option<String>, TaskStoreError> {
        S::enqueue(conn, self).await
    }
}

pub trait RecurringTask: TaskLike + Default {
    fn next_schedule(&self) -> Result<Option<OffsetDateTime>, String>;
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use time::Duration;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TestTask;
    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct ScheduleTestTask;

    #[async_trait]
    impl TaskLike for TestTask {
        const TASK_NAME: &'static str = "test_task";

        type Error = TaskStoreError;
        type Context = ();

        async fn run(&self, _task: CurrentTask, _ctx: Self::Context) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[async_trait]
    impl TaskLike for ScheduleTestTask {
        const TASK_NAME: &'static str = "schedule_task";

        type Error = TaskStoreError;
        type Context = ();

        async fn run(&self, _task: CurrentTask, _ctx: Self::Context) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[async_trait]
    impl RecurringTask for ScheduleTestTask {
        fn next_schedule(&self) -> Result<Option<OffsetDateTime>, String> {
            Ok(OffsetDateTime::now_utc().checked_add(Duration::minutes(5)))
        }
    }
}
