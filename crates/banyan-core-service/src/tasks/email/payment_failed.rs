#![allow(dead_code)]

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{send_email_message, should_send_email_message, EmailTaskContext, EmailTaskError};
use crate::email::message::PaymentFailed;

#[derive(Deserialize, Serialize)]
pub struct PaymentFailedEmailTask {
    user_id: Uuid,
}

impl PaymentFailedEmailTask {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

#[async_trait]
impl TaskLike for PaymentFailedEmailTask {
    const TASK_NAME: &'static str = "payment_failed_email_task";

    type Error = EmailTaskError;
    type Context = EmailTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Filter out innapropriate emails
        if !should_send_email_message(self.user_id, &ctx).await? {
            return Ok(());
        }
        let message = PaymentFailed {};
        send_email_message(self.user_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::email::tests::test_setup;

    #[tokio::test]
    /// PaymentFailedEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, user_id, current_task) = test_setup().await;
        let task = PaymentFailedEmailTask::new(user_id);
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }
}
