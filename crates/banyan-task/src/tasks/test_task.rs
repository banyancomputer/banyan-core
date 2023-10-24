#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CurrentTask;
use crate::TaskLike;

#[derive(Deserialize, Serialize)]
pub struct TestTask {
    user_id: Uuid,
}

impl TestTask {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

#[async_trait]
impl TaskLike for TestTask {
    const TASK_NAME: &'static str = "test_task";

    type Error = TestTaskError;
    type Context = sqlx::SqlitePool;

    async fn run(&self, task: CurrentTask, _ctx: Self::Context) -> Result<(), Self::Error> {
        // intentionally fail the task the first time it gets queued
        if task.current_attempt() == 0 {
            return Err(TestTaskError::IntentionalFailure);
        }

        tracing::info!("the test task was run for user {}", self.user_id);
        return Ok(());
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TestTaskError {
    #[error("the task failed intentionally")]
    IntentionalFailure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::current_task::tests::{default_current_task, increment_current_task_attempt_count};

    const USER_ID: &str = "00000000-0000-0000-0000-000000000000";

    #[tokio::test]
    async fn test_task() -> Result<(), TestTaskError> {
        let ctx = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("db setup");
        let current_task = default_current_task();
        let task = TestTask::new(Uuid::parse_str(USER_ID).unwrap());
        let run_result = task.run(current_task, ctx.clone()).await;
        assert!(run_result.is_err());
        let mut current_task = default_current_task();
        increment_current_task_attempt_count(&mut current_task);
        let run_result = task.run(current_task, ctx).await;
        assert!(run_result.is_ok());
        Ok(())
    }
}
