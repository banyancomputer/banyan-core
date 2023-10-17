#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::config::EmailConfig;
use crate::email::error::EmailError;
use crate::email::message::EmailMessage;

use crate::db::models::CreatedResource;

use crate::workers::CurrentTask;
use crate::workers::TaskLike;

#[derive(Deserialize, Serialize)]
pub struct EmailTask<M> {
    account_id: Uuid,
    message: M,
}

impl<M> EmailTask<M>
where
    M: EmailMessage,
{
    pub fn new(account_id: Uuid, message: M) -> Self {
        Self {
            account_id,
            message,
        }
    }
}

#[async_trait]
impl<M> TaskLike for EmailTask<M>
where
    M: EmailMessage,
{
    const TASK_NAME: &'static str = "email_task";

    type Error = EmailTaskError;
    type Context = sqlx::SqlitePool;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        // Get the email and transport configuration
        let config = EmailConfig::from_env()?;
        let transport = config.transport()?;

        // Filter out innapropriate emails
        let mut connection = ctx.acquire().await?;
        let account_id = self.account_id.to_string();

        // If our last email we sent to the user resulting in a status of unsubscribe we should not send the message
        let unsubscribed = sqlx::query_scalar!(
            r#"SELECT
                CASE
                    WHEN e.state = 'unsubscribed' THEN 1
                    ELSE 0
                END
            FROM emails e
            WHERE e.account_id = $1
            ORDER BY e.sent_at DESC
            LIMIT 1"#,
            account_id
        )
        .fetch_one(&mut *connection)
        .await?;
        if unsubscribed == 1 {
            tracing::info!("the user has unsubscribed from emails");
            return Ok(());
        }

        // If the last three emails we sent to a user resulted in a delivery failure we should not send the message
        let delivery_failures = sqlx::query_scalar!(
            r#"SELECT
                COUNT(*) AS delivery_failures
            FROM emails e
            WHERE e.account_id = $1
                AND e.state = 'delivery_failure'
            ORDER BY e.sent_at DESC
            LIMIT 3"#,
            account_id
        )
        .fetch_one(&mut *connection)
        .await?;
        if delivery_failures >= 3 {
            tracing::info!("the user has had too many delivery failures");
            return Ok(());
        }

        // If we have sent 3 or more email to the user which have been marked as spam we should not send the message
        let spam_reports = sqlx::query_scalar!(
            r#"SELECT
                COUNT(*) AS spam_reports
            FROM emails e
            WHERE e.account_id = $1
                AND e.state = 'spam'
            ORDER BY e.sent_at DESC
            LIMIT 3"#,
            account_id
        )
        .fetch_one(&mut *connection)
        .await?;
        if spam_reports >= 3 {
            tracing::info!("the user has had too many spam reports");
            return Ok(());
        }

        // Get the recipient address
        let recipient_address = sqlx::query_scalar!(
            r#"SELECT u.email as "email!"
            FROM users u
            JOIN accounts a ON u.id = a.userId
            WHERE a.id = $1;"#,
            account_id
        )
        .fetch_one(&mut *connection)
        .await?;

        // Create a new record for the message
        let type_name = self.message.type_name();
        let created_resource = sqlx::query_as!(
            CreatedResource,
            r#"INSERT INTO emails (account_id, type)
            VALUES ($1, $2)
            RETURNING id"#,
            account_id,
            type_name
        )
        .fetch_one(&mut *connection)
        .await?;
        let message_id = created_resource.id.parse::<Uuid>().expect("invalid uuid");

        // Send the email -- capture errors to prevent the task from being retried
        let send_result = self.message.send(
            &transport,
            config.from(),
            &recipient_address,
            message_id,
            config.test_mode(),
        );
        match send_result {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("email failed to send: {}", error);
            }
        }

        return Ok(());
    }
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EmailTaskError {
    #[error("the task is missing a message id")]
    MissingMessageId,
    #[error("the task encountered an email error: {0}")]
    EmailError(#[from] EmailError),
    #[error("the task encountered a sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}
