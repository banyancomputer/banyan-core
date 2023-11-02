#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::message::GaRelease;
use banyan_task::CurrentTask;
use banyan_task::TaskLike;

use super::send_email_message;
use super::should_send_email_message;
use super::EmailTaskContext;
use super::EmailTaskError;

#[derive(Deserialize, Serialize)]
pub struct GaReleaseEmailTask {
    user_id: Uuid,
}

impl GaReleaseEmailTask {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

#[async_trait]
impl TaskLike for GaReleaseEmailTask {
    const TASK_NAME: &'static str = "ga_release_email_task";

    type Error = EmailTaskError;
    type Context = EmailTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Filter out innapropriate emails
        if !should_send_email_message(self.user_id, &ctx).await? {
            return Ok(());
        }
        let message = GaRelease {};
        send_email_message(self.user_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::email::tests::test_setup;

    #[tokio::test]
    /// GaReleaseEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, user_id, current_task) = test_setup().await;
        let task = GaReleaseEmailTask::new(user_id);
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }
}
