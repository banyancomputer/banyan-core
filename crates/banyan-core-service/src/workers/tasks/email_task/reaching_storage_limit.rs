#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::message::ReachingStorageLimit;
use crate::workers::CurrentTask;
use crate::workers::TaskLike;

use super::send_email_message;
use super::should_send_email_message;
use super::EmailTaskContext;
use super::EmailTaskError;

#[derive(Deserialize, Serialize)]
pub struct ReachingStorageLimitEmailTask {
    account_id: Uuid,
    current_usage: usize,
    max_usage: usize,
}

impl ReachingStorageLimitEmailTask {
    pub fn new(account_id: Uuid, current_usage: usize, max_usage: usize) -> Self {
        Self {
            account_id,
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
        if !should_send_email_message(self.account_id, &ctx).await? {
            return Ok(());
        }
        let message = ReachingStorageLimit {
            current_usage: self.current_usage,
            max_usage: self.max_usage,
        };
        send_email_message(self.account_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workers::tasks::email_task::tests::test_setup;

    #[tokio::test]
    /// ReachingStorageLimitEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, account_id, current_task) = test_setup().await;
        let task = ReachingStorageLimitEmailTask::new(account_id, 0, 0);
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }
}
