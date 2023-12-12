#![allow(dead_code)]

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{send_email_message, should_send_email_message, EmailTaskContext, EmailTaskError};
use crate::email::message::ReachingStorageLimit;

#[derive(Deserialize, Serialize)]
pub struct ReachingStorageLimitEmailTask {
    user_id: Uuid,
    current_usage: usize,
    max_usage: usize,
}

impl ReachingStorageLimitEmailTask {
    pub fn new(user_id: Uuid, current_usage: usize, max_usage: usize) -> Self {
        Self {
            user_id,
            current_usage,
            max_usage,
        }
    }
}

#[async_trait]
impl TaskLike for ReachingStorageLimitEmailTask {
    const TASK_NAME: &'static str = "reaching_storage_limit_email_task";

    type Error = EmailTaskError;
    type Context = EmailTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Filter out innapropriate emails
        if !should_send_email_message(self.user_id, &ctx).await? {
            return Ok(());
        }
        let message = ReachingStorageLimit {
            current_usage: self.current_usage,
            max_usage: self.max_usage,
        };
        send_email_message(self.user_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::email::tests::test_setup;

    #[tokio::test]
    /// ReachingStorageLimitEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, user_id, current_task) = test_setup().await;
        let task = ReachingStorageLimitEmailTask::new(user_id, 0, 0);
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }
}
