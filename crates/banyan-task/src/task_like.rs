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

    fn next_time(&self) -> Option<OffsetDateTime> {
        None
    }
}

#[async_trait]
pub trait TaskLikeExt {
    async fn enqueue<S: TaskStore>(
        self,
        pool: &mut S::Pool,
    ) -> Result<Option<String>, TaskStoreError>;

    async fn enqueue_with_connection<S: TaskStore>(
        self,
        pool: &mut S::Connection,
    ) -> Result<Option<String>, TaskStoreError>;
}

#[async_trait]
impl<T> TaskLikeExt for T
where
    T: TaskLike,
{
    async fn enqueue<S: TaskStore>(
        self,
        pool: &mut S::Pool,
    ) -> Result<Option<String>, TaskStoreError> {
        S::enqueue(pool, self).await
    }

    async fn enqueue_with_connection<S: TaskStore>(
        self,
        connection: &mut S::Connection,
    ) -> Result<Option<String>, TaskStoreError> {
        S::enqueue_with_connection(connection, self).await
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod tests {
    use std::time::Duration;

    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TestTask;
    #[derive(Debug, Serialize, Deserialize)]
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

        fn next_time(&self) -> Option<OffsetDateTime> {
            Some(OffsetDateTime::now_utc() + Duration::from_secs(60))
        }
    }
}
