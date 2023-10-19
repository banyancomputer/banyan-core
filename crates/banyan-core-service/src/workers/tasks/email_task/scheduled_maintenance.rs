#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::message::ScheduledMaintenance;
use crate::workers::CurrentTask;
use crate::workers::TaskLike;

use super::send_email_message;
use super::should_send_email_message;
use super::EmailTaskContext;
use super::EmailTaskError;

#[derive(Deserialize, Serialize)]
pub struct ScheduledMaintenanceEmailTask {
    account_id: Uuid,
    start: String,
    end: String,
}

impl ScheduledMaintenanceEmailTask {
    pub fn new(account_id: Uuid, start: String, end: String) -> Self {
        Self {
            account_id,
            start,
            end,
        }
    }
}

#[async_trait]
impl TaskLike for ScheduledMaintenanceEmailTask {
    const TASK_NAME: &'static str = "scheduled_maintenance_email_task";

    type Error = EmailTaskError;
    type Context = EmailTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Filter out innapropriate emails
        if !should_send_email_message(self.account_id, &ctx).await? {
            return Ok(());
        }
        let message = ScheduledMaintenance {
            start: self.start.clone(),
            end: self.end.clone(),
        };
        send_email_message(self.account_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workers::tasks::email_task::tests::test_setup;

    #[tokio::test]
    /// ScheduledMaintenanceEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, account_id, current_task) = test_setup().await;
        let task = ScheduledMaintenanceEmailTask::new(
            account_id,
            "2021-01-01".to_string(),
            "2021-01-02".to_string(),
        );
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }
}
