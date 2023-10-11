use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::workers::CurrentTask;
use crate::workers::TaskLike;

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
    type Context = ();

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
