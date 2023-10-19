#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use url::Url;

use crate::email::message::ProductInvoice;
use crate::workers::CurrentTask;
use crate::workers::TaskLike;

use super::send_email_message;
use super::should_send_email_message;
use super::EmailTaskContext;
use super::EmailTaskError;

#[derive(Deserialize, Serialize)]
pub struct ProductInvoiceEmailTask {
    account_id: Uuid,
    url: Url,
}

impl ProductInvoiceEmailTask {
    pub fn new(account_id: Uuid, url: Url) -> Self {
        Self { account_id, url }
    }
}

#[async_trait]
impl TaskLike for ProductInvoiceEmailTask {
    const TASK_NAME: &'static str = "product_invoice_email_task";

    type Error = EmailTaskError;
    type Context = EmailTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Filter out innapropriate emails
        if !should_send_email_message(self.account_id, &ctx).await? {
            return Ok(());
        }
        let message = ProductInvoice { url: self.url.clone() };
        send_email_message(self.account_id, &message, &ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workers::tasks::email_task::tests::test_setup;

    #[tokio::test]
    /// ProductInvoiceEmailTask should succeed in a valid context
    async fn success() {
        let (ctx, account_id, current_task) = test_setup().await;
        let task = ProductInvoiceEmailTask::new(account_id, Url::parse("https://example.com").unwrap());
        let result = task.run(current_task, ctx).await;
        assert!(result.is_ok());
    }
}

